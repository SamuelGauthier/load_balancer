use crate::backend::Backend;
use crate::health::Health;
use crate::internal_error::InternalError;
use crate::load_balancer::LoadBalancer;

use async_trait::async_trait;
use log::{debug, info};
use tokio::sync::RwLock as TokioRwLock;

/// Represents a very basic load balancer. Sends the requests to healthy backend servers in a round
/// robin fashion.
#[derive(Debug)]
pub struct RoundRobinLoadBalancer {
    /// List of backend servers
    backends: Vec<Box<dyn Backend>>,

    /// Index of the current backend server to which the next request will be sent.
    current_backend_index: TokioRwLock<usize>,
}

impl RoundRobinLoadBalancer {
    /// Creates a new load balancer with the given list of backend servers to route the requests
    /// to. The health check interval is the time in seconds between each health check sent to the
    /// backends.
    pub fn new(backends: Vec<Box<dyn Backend>>) -> Self {
        Self {
            backends,
            current_backend_index: 0.into(),
        }
    }
}

#[async_trait]
impl LoadBalancer for RoundRobinLoadBalancer {
    /// Returns the next available backend server to which the request can be sent. If none are
    /// available, an error is returned.
    async fn next_available_backend(&self) -> Result<Box<dyn Backend>, String> {
        debug!("trying to acquire current_backend_index write lock");
        let mut current_backend_index = self.current_backend_index.write().await;
        debug!("acquired current_backend_index write lock");

        let mut tried_backends = 0;

        let mut backend_index = *current_backend_index;
        *current_backend_index = (*current_backend_index + 1) % self.backends.len();

        self.backends[backend_index].check_health().await;
        let mut backend_health = self.backends[backend_index].health().await;

        while tried_backends < self.backends.len() {
            if backend_health == Health::Healthy {
                debug!("selected healthy backend {:?}", backend_index);
                return Ok(self.backends[backend_index].clone());
            }

            backend_index = *current_backend_index;
            *current_backend_index = (*current_backend_index + 1) % self.backends.len();

            self.backends[backend_index].check_health().await;
            backend_health = self.backends[backend_index].health().await;

            tried_backends += 1;
        }

        return Err("No backend server available".to_string());
    }

    /// Sends a request to the next available backend server. Returns an error if no backend server
    /// is reachable.
    async fn send_request(&self) -> Result<String, InternalError> {
        debug!("trying to get next available backend");
        let backend = self.next_available_backend().await;
        match backend {
            Ok(backend) => {
                info!("Sending request to backend {:?}", backend);
                let response = backend.send_request().await;
                match response {
                    Ok(response) => {
                        info!("{:?}", response);
                        let body = response.text_with_charset("utf-8").await.unwrap();
                        Ok(body)
                    }
                    Err(_) => Err(InternalError::BackendUnreachable),
                }
            }
            Err(_) => Err(InternalError::NoBackendAvailable),
        }
    }

    /// Checks and update the health status of all backend servers.
    async fn check_backends_healths(&self) {
        // This is used for profiling only
        let start_time = std::time::Instant::now();

        for backend in &self.backends {
            backend.check_health().await;
        }

        // For profiling only, measures how much time it took to check all backends health
        let end_time = std::time::Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_millis();
        info!("checking all backends health took {}ms", elapsed_time);
    }
}
