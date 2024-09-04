use crate::backend::Backend;
use crate::health::Health;
use crate::internal_error::InternalError;
use crate::load_balancer::LoadBalancer;

use async_trait::async_trait;
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::spawn;
use tokio::time::{interval, Duration};

/// Represents a very basic load balancer. Sends the requests to healthy backend servers in a round
/// robin fashion.
#[derive(Clone, Debug)]
pub struct RoundRobinLoadBalancer {
    /// List of backend servers
    backends: Vec<Box<dyn Backend>>,

    /// Index of the current backend server to which the next request will be sent.
    current_backend_index: usize,

    /// Number of backend servers that have been tried. If all backend servers have been tried, the
    /// load balancer will return an error.
    tried_backends: usize,
}

impl RoundRobinLoadBalancer {
    /// Creates a new load balancer with the given list of backend servers to route the requests
    /// to. The health check interval is the time in seconds between each health check sent to the
    /// backends.
    pub fn new(
        backends: Vec<Box<dyn Backend>>,
        health_check_interval: u64,
    ) -> Arc<TokioMutex<Box<dyn LoadBalancer>>> {
        // We need to create a mutex from the Tokio lib in order to be able to pass it to the new
        // created task
        let load_balancer: Box<dyn LoadBalancer> = Box::new(RoundRobinLoadBalancer {
            backends,
            current_backend_index: 0,
            tried_backends: 0,
        });

        let lb_arc = Arc::new(TokioMutex::new(load_balancer));
        let load_balancer_clone = Arc::clone(&lb_arc);

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

        lb_arc
    }
}

#[async_trait]
impl LoadBalancer for RoundRobinLoadBalancer {
    // Returns the next available backend server to which the request can be sent. If none are
    // available, an error is returned.
    async fn next_available_backend(&mut self) -> Result<Box<dyn Backend>, String> {
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

    async fn send_request(&mut self) -> Result<String, InternalError> {
        let backend = self.next_available_backend().await;
        match backend {
            Ok(mut backend) => {
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
