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

    /// Number of backend servers that have been tried. If all backend servers have been tried, the
    /// load balancer will return an error.
    tried_backends: TokioRwLock<usize>,
}

impl RoundRobinLoadBalancer {
    /// Creates a new load balancer with the given list of backend servers to route the requests
    /// to. The health check interval is the time in seconds between each health check sent to the
    /// backends.
    pub fn new(backends: Vec<Box<dyn Backend>>) -> Self {
        Self {
            backends,
            current_backend_index: 0.into(),
            tried_backends: 0.into(),
        }
    }
}

#[async_trait]
impl LoadBalancer for RoundRobinLoadBalancer {
    /// Returns the next available backend server to which the request can be sent. If none are
    /// available, an error is returned.
    async fn next_available_backend(&self) -> Result<Box<dyn Backend>, String> {
        // We have tried all the backend servers, none are available
        let r_tried_backends = self.tried_backends.read().await;
        if *r_tried_backends == self.backends.len() {
            return Err("No backend server available".to_string());
        }
        drop(r_tried_backends);

        debug!("trying to acquire current_backend_index write lock");
        let mut current_backend_index = self.current_backend_index.write().await;
        debug!("acquired current_backend_index write lock");

        let backend_index = *current_backend_index;
        // We increment the index to point to the next backend server
        *current_backend_index = (*current_backend_index + 1) % self.backends.len();

        // We check the health of the backend server
        self.backends[backend_index].check_health().await;
        let backend_health = self.backends[backend_index].health().await;

        debug!("trying to acquire tried_backends write lock");
        let mut w_tried_backends = self.tried_backends.write().await;
        debug!("acquired tried_backends write lock");

        if backend_health == Health::Healthy {
            debug!("selected healthy backend {:?}", backend_index);
            // It is healthy, we can return it and reset the number of tried backends
            *w_tried_backends = 0;
            return Ok(self.backends[backend_index].clone());
        }

        // It is unhealthy, we try the next one
        *w_tried_backends += 1;
        // As this function is recursive, we need to watch out for stack overflow, so we use the
        // heap and guarantee that the value remains at the same place
        Box::pin(self.next_available_backend()).await
    }

    /// Sends a request to the next available backend server. Returns an error if no backend server
    /// is reachable.
    async fn send_request(&self) -> Result<String, InternalError> {
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
