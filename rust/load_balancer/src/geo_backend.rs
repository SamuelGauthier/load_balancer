use crate::backend::Backend;
use crate::health::Health;
use async_trait::async_trait;
use reqwest::{Client, Error, Response, StatusCode};

use log::{error, info, warn};

/// Represents a backend server resource to which the load balancer can forward the requests.
#[derive(Clone, Debug)]
pub struct GeoBackend {
    /// Address of the backend server, contains the protocol, hostname and port. For example:
    /// http://localhost:8081
    address: String,

    /// [Unused for now] Weight of the backend server. The higher the weight, the more requests
    /// will be forwarded to.
    weight: u32,

    /// Health status of the backend server.
    health: Health,
}

impl GeoBackend {
    /// Creates a new backend server with the given address, weight and health status.
    pub fn new(address: String, weight: u32, health: Health) -> Self {
        GeoBackend {
            address,
            weight,
            health,
        }
    }
}

#[async_trait]
impl Backend for GeoBackend {
    /// Checks the health of the backend server by sending a request to the health check endpoint.
    /// If the server is healthy, the health status is set to Healthy, otherwise it is set to
    /// Unhealthy.
    async fn check_health(&mut self) {
        // This is used for profiling only
        let start_time = std::time::Instant::now();

        // Sends a health check
        let health_check_address = self.address.clone() + "health";
        let client = Client::new();
        let response = client.get(&health_check_address).send().await;

        // For profiling only, measures the time it took to send the request
        let end_time = std::time::Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_millis();
        info!("checking backend health took {}ms", elapsed_time);

        match response {
            // The server is considered healthy if the health enpoint returns anything.
            Ok(r) => {
                if r.status() != StatusCode::OK {
                    warn!(
                        "GeoBackend server {} does not support health checks on address {}",
                        self.address, health_check_address
                    );
                }

                info!("Response: {:?}", r);
                info!("GeoBackend server {} is healthy", self.address);
                self.health = Health::Healthy;
            }
            // The server cannot be reached, it is considered unhealthy.
            Err(e) => {
                error!("Failed to send request to backend server: {:?}", e);
                info!("GeoBackend server {} is unhealthy", self.address);
                self.health = Health::Unhealthy;
            }
        }
    }

    /// Returns the health status of the backend server.
    fn health(&self) -> Health {
        self.health.clone()
    }

    /// Sends a request to the backend server and returns the response in case of success. If the
    /// request succeeds, the health status is updated to healthy. If the request fails, the health
    /// status of the backend server is set to Unhealthy.
    ///
    /// You should add arguments to this function to pass the request method, headers, body, etc.
    async fn send_request(&mut self) -> Result<Response, Error> {
        info!("Sending request to backend server {}", self.address);
        // This is used for profiling only
        let start_time = std::time::Instant::now();

        // Sends a request to the backend server
        let client = Client::new();
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
