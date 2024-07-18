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

#[derive(Clone, Debug, PartialEq)]
enum Health {
    Healthy,
    Unhealthy,
}

#[derive(Clone, Debug)]
struct Backend {
    address: String,
    weight: u32,
    health: Health,
}

impl Backend {
    pub fn new(address: String, weight: u32, health: Health) -> Self {
        Backend {
            address,
            weight,
            health,
        }
    }

    pub async fn check_health(&mut self) {
        let start_time = std::time::Instant::now();

        let health_check_address = self.address.clone() + "health";
        let client = reqwest::Client::new();
        let response = client.get(&health_check_address).send().await;

        let end_time = std::time::Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_millis();
        info!("checking backend health took {}ms", elapsed_time);

        match response {
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
            Err(e) => {
                error!("Failed to send request to backend server: {:?}", e);
                info!("Backend server {} is unhealthy", self.address);
                self.health = Health::Unhealthy;
            }
        }
    }

    pub fn health(&self) -> Health {
        self.health.clone()
    }

    pub async fn send_request(&mut self) -> Result<reqwest::Response, reqwest::Error> {
        info!("Sending request to backend server {}", self.address);
        let start_time = std::time::Instant::now();

        let client = reqwest::Client::new();
        let response = client.get(&self.address).send().await;

        let end_time = std::time::Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_millis();
        info!("sending request to backend took {}ms", elapsed_time);

        match response {
            Ok(r) => {
                self.health = Health::Healthy;
                Ok(r)
            }
            Err(e) => {
                error!("Failed to send request to backend server: {:?}", e);
                self.health = Health::Unhealthy;
                Err(e)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct LoadBalancer {
    backends: Vec<Backend>,
    current_backend_index: usize,
    tried_backends: usize,
}

impl LoadBalancer {
    pub fn new(backends: Vec<Backend>, health_check_interval: u64) -> Arc<TokioMutex<Self>> {
        let load_balancer = Arc::new(TokioMutex::new(Self {
            backends,
            current_backend_index: 0,
            tried_backends: 0,
        }));

        let load_balancer_clone = Arc::clone(&load_balancer);

        // This runs periodically on the same thread as the load balancer. We cannot spawn it on
        // another thread because ntex::http::client::Client is not Sync nor Send.
        spawn(async move {
            // spawn_local(async move {
            let mut interval = interval(Duration::from_secs(health_check_interval));
            loop {
                interval.tick().await;
                let mut lb = load_balancer_clone.lock().await;
                lb.check_backends_healths().await;
            }
        });

        load_balancer
    }

    pub async fn next_available_backend(&mut self) -> Result<Backend, String> {
        if self.tried_backends == self.backends.len() {
            return Err("No backend server available".to_string());
        }

        let backend_index = self.current_backend_index;
        self.current_backend_index = (self.current_backend_index + 1) % self.backends.len();

        self.backends[backend_index].check_health().await;
        if self.backends[backend_index].health() == Health::Healthy {
            self.tried_backends = 0;
            return Ok(self.backends[backend_index].clone());
        }

        self.tried_backends += 1;
        Box::pin(self.next_available_backend()).await
    }

    async fn check_backends_healths(&mut self) {
        let start_time = std::time::Instant::now();

        for backend in &mut self.backends {
            backend.check_health().await;
        }

        let end_time = std::time::Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_millis();
        info!("checking all backends health took {}ms", elapsed_time);
    }
}

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

#[actix_web::get("/")]
async fn index(
    load_balancer: actix_web::web::Data<Arc<TokioMutex<LoadBalancer>>>,
    request: actix_web::HttpRequest,
) -> Result<String, actix_web::Error> {
    print_request_info(request).await;

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
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
