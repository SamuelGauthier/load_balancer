use crate::backend::Backend;
use crate::health::Health;
use async_trait::async_trait;
use reqwest::{Client, Error, Response, StatusCode};
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

use log::{debug, error, info, warn};

/// Represents a backend server resource to which the load balancer can forward the requests.
#[derive(Debug)]
pub struct SimpleBackend {
    /// Address of the backend server, contains the protocol, hostname and port. For example:
    /// http://localhost:8081
    address: String,

    /// Response time of the backend server in milliseconds.
    response_time_ms: Arc<TokioRwLock<f32>>,

    /// Health status of the backend server.
    health: Arc<TokioRwLock<Health>>,
}

impl SimpleBackend {
    pub fn new(address: String, health: Health) -> Self {
        Self {
            address,
            response_time_ms: Arc::new(TokioRwLock::new(0.0)),
            health: Arc::new(TokioRwLock::new(health)),
        }
    }
}

impl Clone for SimpleBackend {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
            response_time_ms: Arc::clone(&self.response_time_ms),
            health: Arc::clone(&self.health),
        }
    }
}

#[async_trait]
impl Backend for SimpleBackend {
    /// Checks the health of the backend server by sending a request to the health check endpoint.
    /// If the server is healthy, the health status is set to Healthy, otherwise it is set to
    /// Unhealthy.
    async fn check_health(&self) {
        let start_time = std::time::Instant::now();

        // Sends a health check
        let health_check_address = self.address.clone() + "health";
        let client = Client::new();
        let response = client.get(&health_check_address).send().await;

        let end_time = std::time::Instant::now();
        let elapsed_time_ms = end_time.duration_since(start_time).as_millis();
        info!("checking backend health took {}ms", elapsed_time_ms);

        debug!(
            "[{}] trying to acquire write lock for response time",
            self.address
        );
        let mut response_time = self.response_time_ms.write().await;
        debug!("[{}] acquired write lock for response time", self.address);

        *response_time = elapsed_time_ms as f32;
        drop(response_time);

        debug!("[{}] trying to acquire write lock for health", self.address);
        let mut health = self.health.write().await;
        debug!("[{}] acquired write lock for health", self.address);

        match response {
            // The server is considered healthy if the health enpoint returns anything.
            Ok(r) => {
                info!("Response: {:?}", r);

                if r.status() != StatusCode::OK {
                    warn!(
                        "SimpleBackend server {} does not support health checks on address {}",
                        self.address, health_check_address
                    );
                }

                info!("SimpleBackend server {} is healthy", self.address);
                *health = Health::Healthy;
            }
            Err(e) => {
                error!("Failed to send request to backend server: {:?}", e);
                info!("SimpleBackend server {} is unhealthy", self.address);
                *health = Health::Unhealthy;
            }
        }
    }

    /// Returns the health status of the backend server.
    async fn health(&self) -> Health {
        let h = self.health.read().await;
        *h
    }

    /// Sends a request to the backend server and returns the response in case of success. If the
    /// request succeeds, the health status is updated to healthy. If the request fails, the health
    /// status of the backend server is set to Unhealthy.
    ///
    /// TODO: You should add arguments to this function to pass the request method, headers, body, etc.
    async fn send_request(&self) -> Result<Response, Error> {
        info!("Sending request to backend server {}", self.address);
        let start_time = std::time::Instant::now();

        let client = Client::new();
        let response = client.get(&self.address).send().await;

        let end_time = std::time::Instant::now();
        let elapsed_time_ms = end_time.duration_since(start_time).as_millis();
        info!("sending request to backend took {}ms", elapsed_time_ms);

        debug!(
            "[{}] trying to acquire write lock for response time",
            self.address
        );
        let mut response_time = self.response_time_ms.write().await;
        debug!("[{}] acquired write lock for response time", self.address);

        *response_time = elapsed_time_ms as f32;

        let r_health = self.health.read().await;

        match response {
            Ok(r) => {
                if *r_health != Health::Healthy {
                    debug!("[{}] trying to acquire write lock for health", self.address);
                    let mut health = self.health.write().await;
                    debug!("[{}] acquired write lock for health", self.address);
                    *health = Health::Healthy;
                }
                Ok(r)
            }
            Err(e) => {
                error!("Failed to send request to backend server: {:?}", e);
                if *r_health != Health::Unhealthy {
                    debug!("[{}] trying to acquire write lock for health", self.address);
                    let mut health = self.health.write().await;
                    debug!("[{}] acquired write lock for health", self.address);
                    *health = Health::Unhealthy;
                }
                Err(e)
            }
        }
    }

    /// Returns the response time in milliseconds of the last request sent to the backend server.
    async fn response_time_ms(&self) -> f32 {
        let response_time = self.response_time_ms.read().await;
        *response_time
    }

    /// Returns the name of the backend server.
    fn address(&self) -> &str {
        self.address.as_str()
    }
}
