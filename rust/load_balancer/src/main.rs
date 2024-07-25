/*
 * A simple load balancer listening on port 8080 and forwarding requests to a backend server
 *
 * Author: Samuel Gauthier
 */
use actix_web;
use actix_web::error::InternalError;
use actix_web::http::StatusCode;
use clap::Parser;
use log::{error, info, warn};
use reqwest;
use simple_logger;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::spawn;
use tokio::time::{interval, Duration};

/// Servers are defined as either healthy or unhealthy. In the case of unhealthy servers, the load
/// balancer will not forward requests to them.
#[derive(Clone, Debug, PartialEq)]
enum Health {
    Healthy,
    Unhealthy,
}

/// Represents a backend server resource to which the load balancer can forward the requests.
#[derive(Clone, Debug)]
struct Backend {
    /// Address of the backend server, contains the protocol, hostname and port. For example:
    /// http://localhost:8081
    address: String,

    /// [Unused for now] Weight of the backend server. The higher the weight, the more requests
    /// will be forwarded to.
    weight: u32,

    /// Health status of the backend server.
    health: Health,
}

impl Backend {
    /// Creates a new backend server with the given address, weight and health status.
    pub fn new(address: String, weight: u32, health: Health) -> Self {
        Backend {
            address,
            weight,
            health,
        }
    }

    /// Checks the health of the backend server by sending a request to the health check endpoint.
    /// If the server is healthy, the health status is set to Healthy, otherwise it is set to
    /// Unhealthy.
    pub async fn check_health(&mut self) {
        // This is used for profiling only
        let start_time = std::time::Instant::now();

        // Sends a health check
        let health_check_address = self.address.clone() + "health";
        let client = reqwest::Client::new();
        let response = client.get(&health_check_address).send().await;

        // For profiling only, measures the time it took to send the request
        let end_time = std::time::Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_millis();
        info!("checking backend health took {}ms", elapsed_time);

        match response {
            // The server is considered healthy if the health enpoint returns anything.
            Ok(r) => {
                if r.status() != reqwest::StatusCode::OK {
                    warn!(
                        "Backend server {} does not support health checks on address {}",
                        self.address, health_check_address
                    );
                }

                info!("Response: {:?}", r);
                info!("Backend server {} is healthy", self.address);
                self.health = Health::Healthy;
            }
            // The server cannot be reached, it is considered unhealthy.
            Err(e) => {
                error!("Failed to send request to backend server: {:?}", e);
                info!("Backend server {} is unhealthy", self.address);
                self.health = Health::Unhealthy;
            }
        }
    }

    /// Returns the health status of the backend server.
    pub fn health(&self) -> Health {
        self.health.clone()
    }

    /// Sends a request to the backend server and returns the response in case of success. If the
    /// request succeeds, the health status is updated to healthy. If the request fails, the health
    /// status of the backend server is set to Unhealthy.
    ///
    /// You should add arguments to this function to pass the request method, headers, body, etc.
    pub async fn send_request(&mut self) -> Result<reqwest::Response, reqwest::Error> {
        info!("Sending request to backend server {}", self.address);
        // This is used for profiling only
        let start_time = std::time::Instant::now();

        // Sends a request to the backend server
        let client = reqwest::Client::new();
        let response = client.get(&self.address).send().await;

        // For profiling only, measures the time it took to send the request
        let end_time = std::time::Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_millis();
        info!("sending request to backend took {}ms", elapsed_time);

        match response {
            // The backend server responded, so it is considered healthy.
            Ok(r) => {
                if self.health != Health::Healthy {
                    self.health = Health::Healthy;
                }
                Ok(r)
            }
            // The server cannot be reached, it is considered unhealthy.
            Err(e) => {
                error!("Failed to send request to backend server: {:?}", e);
                if self.health != Health::Unhealthy {
                    self.health = Health::Unhealthy;
                }
                Err(e)
            }
        }
    }
}

/// Represents a very basic load balancer. Sends the requests to healthy backend servers in a round
/// robin fashion.
#[derive(Clone, Debug)]
struct LoadBalancer {
    /// List of backend servers
    backends: Vec<Backend>,

    /// Index of the current backend server to which the next request will be sent.
    current_backend_index: usize,

    /// Number of backend servers that have been tried. If all backend servers have been tried, the
    /// load balancer will return an error.
    tried_backends: usize,
}

impl LoadBalancer {
    /// Creates a new load balancer with the given list of backend servers to route the requests
    /// to. The health check interval is the time in seconds between each health check sent to the
    /// backends.
    pub fn new(backends: Vec<Backend>, health_check_interval: u64) -> Arc<TokioMutex<Self>> {
        // We need to create a mutex from the Tokio lib in order to be able to pass it to the new
        // created task
        let load_balancer = Arc::new(TokioMutex::new(Self {
            backends,
            current_backend_index: 0,
            tried_backends: 0,
        }));

        let load_balancer_clone = Arc::clone(&load_balancer);

        // Start a background task that checks the health of the backend servers at regular
        // intervals. The interval can be specified in the command line arguments.
        spawn(async move {
            let mut interval = interval(Duration::from_secs(health_check_interval));
            // The loop will run indefinitely
            loop {
                interval.tick().await;
                let mut lb = load_balancer_clone.lock().await;
                lb.check_backends_healths().await;
            }
        });

        load_balancer
    }

    // Returns the next available backend server to which the request can be sent. If none are
    // available, an error is returned.
    pub async fn next_available_backend(&mut self) -> Result<Backend, String> {
        // We have tried all the backend servers, none are available
        if self.tried_backends == self.backends.len() {
            return Err("No backend server available".to_string());
        }

        let backend_index = self.current_backend_index;
        // We increment the index to point to the next backend server
        self.current_backend_index = (self.current_backend_index + 1) % self.backends.len();

        // We check the health of the backend server
        self.backends[backend_index].check_health().await;
        if self.backends[backend_index].health() == Health::Healthy {
            // It is healthy, we can return it and reset the number of tried backends
            self.tried_backends = 0;
            return Ok(self.backends[backend_index].clone());
        }

        // It is unhealthy, we try the next one
        self.tried_backends += 1;
        // As this function is recursive, we need to watch out for stack overflow, so we use the
        // heap and guarantee that the value remains at the same place
        Box::pin(self.next_available_backend()).await
    }

    /// Checks and update the health status of all backend servers.
    async fn check_backends_healths(&mut self) {
        // This is used for profiling only
        let start_time = std::time::Instant::now();

        for backend in &mut self.backends {
            backend.check_health().await;
        }

        // For profiling only, measures how much time it took to check all backends health
        let end_time = std::time::Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_millis();
        info!("checking all backends health took {}ms", elapsed_time);
    }
}

/// Prints the request information to the log. Used for debugging purposes only.
async fn print_request_info(request: actix_web::HttpRequest) {
    info!(
        "Received request from {}",
        request.connection_info().peer_addr().unwrap()
    );
    info!(
        "{} {} {:?}",
        request.head().method,
        request.head().uri,
        request.head().version,
    );
    for (key, value) in request.headers().iter() {
        info!("{}: {}", key, value.to_str().unwrap());
    }
}

/// Index route of the load balancer. Forwards the request to the next available backend server.
async fn index(
    load_balancer: actix_web::web::Data<Arc<TokioMutex<LoadBalancer>>>,
    request: actix_web::HttpRequest,
) -> Result<String, actix_web::Error> {
    print_request_info(request).await;

    // Extract the load balancer from the state and get the next available backend server
    let backend = load_balancer
        .get_ref()
        .lock()
        .await
        .next_available_backend()
        .await;

    let mut backend = match backend {
        Ok(b) => {
            info!("Next available backend server: {:?}", b);
            b
        }
        Err(e) => {
            error!("Failed to get next available backend server: {:?}", e);
            return Err(InternalError::new(
                "Failed to get next available backend server",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into());
        }
    };

    // Send the request to the backend server
    let response = backend.send_request().await;
    match response {
        Ok(r) => {
            info!("{:?}", r);
            let body = r.text_with_charset("utf-8").await.unwrap();
            Ok(body)
        }
        Err(e) => {
            error!("Failed to send request to backend server: {:?}", e);
            Err(InternalError::new(
                "Failed to send request to backend server",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into())
        }
    }
}

/// Load balancer listening on port 8080 and forwarding requests to a list of backend servers
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Time interval in miliseconds between health checks
    #[arg(short, long, default_value = "10")]
    interval_health_check: u64,

    /// List of backend servers
    backend_adresses: Vec<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let args = Args::parse();
    let backends = args
        .backend_adresses
        .iter()
        .map(|address| Backend::new(address.clone(), 1, Health::Healthy))
        .collect();

    let load_balancer = LoadBalancer::new(backends, args.interval_health_check);

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(load_balancer.clone()))
            .default_service(actix_web::web::to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
