use crate::backend::Backend;
use crate::health::Health;
use async_trait::async_trait;
use reqwest::{Client, Error, Response, StatusCode};

use log::{error, info, warn};

/// Represents a backend server resource to which the load balancer can forward the requests.
#[derive(Clone, Debug)]
pub struct SimpleBackend {
    /// Address of the backend server, contains the protocol, hostname and port. For example:
    /// http://localhost:8081
    address: String,

    /// Response time of the backend server in milliseconds.
    response_time_ms: f32,

    /// Health status of the backend server.
    health: Health,
}

impl SimpleBackend {
    pub fn new(address: String, health: Health) -> Self {
        Self {
            address,
            response_time_ms: 0.0,
            health,
        }
    }
}

#[async_trait]
impl Backend for SimpleBackend {
    /// Checks the health of the backend server by sending a request to the health check endpoint.
    /// If the server is healthy, the health status is set to Healthy, otherwise it is set to
    /// Unhealthy.
    async fn check_health(&mut self) {
        let start_time = std::time::Instant::now();

        // Sends a health check
        let health_check_address = self.address.clone() + "health";
        let client = Client::new();
        let response = client.get(&health_check_address).send().await;

        let end_time = std::time::Instant::now();
        let elapsed_time_ms = end_time.duration_since(start_time).as_millis();
        info!("checking backend health took {}ms", elapsed_time_ms);
        self.response_time_ms = elapsed_time_ms as f32;

        match response {
            // The server is considered healthy if the health enpoint returns anything.
            Ok(r) => {
                if r.status() != StatusCode::OK {
                    warn!(
                        "SimpleBackend server {} does not support health checks on address {}",
                        self.address, health_check_address
                    );
                }

                info!("Response: {:?}", r);
                info!("SimpleBackend server {} is healthy", self.address);
                self.health = Health::Healthy;
            }
            Err(e) => {
                error!("Failed to send request to backend server: {:?}", e);
                info!("SimpleBackend server {} is unhealthy", self.address);
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
        let start_time = std::time::Instant::now();

        let client = Client::new();
        let response = client.get(&self.address).send().await;

        let end_time = std::time::Instant::now();
        let elapsed_time_ms = end_time.duration_since(start_time).as_millis();
        info!("sending request to backend took {}ms", elapsed_time_ms);
        self.response_time_ms = elapsed_time_ms as f32;

        match response {
            Ok(r) => {
                if self.health != Health::Healthy {
                    self.health = Health::Healthy;
                }
                Ok(r)
            }
            Err(e) => {
                error!("Failed to send request to backend server: {:?}", e);
                if self.health != Health::Unhealthy {
                    self.health = Health::Unhealthy;
                }
                Err(e)
            }
        }
    }

    /// Returns the response time in milliseconds of the last request sent to the backend server.
    fn response_time_ms(&self) -> f32 {
        self.response_time_ms
    }
}

