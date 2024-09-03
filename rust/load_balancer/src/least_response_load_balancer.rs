use crate::backend::Backend;
use crate::health::Health;
use crate::internal_error::InternalError;
use crate::load_balancer::LoadBalancer;

use async_trait::async_trait;
use log::{error, info, warn};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::spawn;
use tokio::time::{interval, Duration};

// To extract into own file
#[derive(Debug, Clone)]
struct MinHeapItem<T> {
    priority: f32,
    backend: T,
}

impl<T> Ord for MinHeapItem<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .priority
            .partial_cmp(&self.priority)
            .unwrap_or(Ordering::Equal)
    }
}

impl<T> PartialOrd for MinHeapItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for MinHeapItem<T> {}

impl<T> PartialEq for MinHeapItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}
// <----

/// Represents a very basic load balancer. Sends the requests to healthy backend servers in a round
/// robin fashion.
#[derive(Clone, Debug)]
pub struct LeastResponseLoadBalancer {
    /// List of unhealthy backends servers
    unhealthy_backends: Vec<Box<dyn Backend>>,

    /// Min heap of healthy backend servers. The heap is ordered by the response time of the
    /// backends
    healthy_backends: BinaryHeap<MinHeapItem<Box<dyn Backend>>>,
}

impl LeastResponseLoadBalancer {
    /// Creates a new load balancer with the given list of backend servers to route the requests
    /// to. The health check interval is the time in seconds between each health check sent to the
    /// backends.
    pub fn new(
        backends: Vec<Box<dyn Backend>>,
        health_check_interval: u64,
    ) -> Arc<TokioMutex<Box<dyn LoadBalancer>>> {
        // We need to create a mutex from the Tokio lib in order to be able to pass it to the new
        // created task
        let mut healthy_backends = BinaryHeap::new();
        for backend in backends.into_iter() {
            healthy_backends.push(MinHeapItem {
                priority: 0.0,
                backend,
            });
        }
        let load_balancer: Box<dyn LoadBalancer> = Box::new(Self {
            unhealthy_backends: Vec::new(),
            healthy_backends,
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
impl LoadBalancer for LeastResponseLoadBalancer {
    // Returns the next available backend server to which the request can be sent. If none are
    // available, an error is returned.
    async fn next_available_backend(&mut self) -> Result<Box<dyn Backend>, String> {
        if self.healthy_backends.is_empty() {
            return Err("No backend server available".to_string());
        }

        let MinHeapItem { backend, .. } = self.healthy_backends.peek().unwrap();

        Ok(backend.clone())
    }

    async fn send_request(&mut self) -> Result<String, InternalError> {
        if self.healthy_backends.is_empty() {
            return Err(InternalError::NoBackendAvailable);
        }

        let MinHeapItem { mut backend, .. } = self.healthy_backends.pop().unwrap();

        // Send the request to the backend server
        let response = backend.send_request().await;
        match response {
            Ok(r) => {
                info!("{:?}", r);
                self.healthy_backends.push(MinHeapItem {
                    priority: backend.response_time_ms(),
                    backend,
                });
                let body = r.text_with_charset("utf-8").await.unwrap();
                Ok(body)
            }
            Err(e) => {
                error!(
                    "Failed to send request to backend server: {:?}, trying next one",
                    e
                );
                self.unhealthy_backends.push(backend);
                // Err(InternalError::BackendUnreachable)
                self.send_request().await
            }
        }
    }

    /// Checks and update the health status of all backend servers.
    async fn check_backends_healths(&mut self) {
        // This is used for profiling only
        let start_time = std::time::Instant::now();

        let mut new_healthy_backends = BinaryHeap::new();
        let mut new_unhealthy_backends: Vec<Box<dyn Backend>> = Vec::new();

        // check healthy backends
        while let Some(mut b) = self.healthy_backends.pop() {
            b.backend.check_health().await;
            if b.backend.health() == Health::Healthy {
                let response_time = b.backend.response_time_ms();
                info!(
                    "Backend {:?} is healthy with response time {}ms",
                    b.backend, response_time
                );
                new_healthy_backends.push(MinHeapItem {
                    priority: response_time,
                    backend: b.backend,
                });
            } else {
                warn!("Backend {:?} is unhealthy", b.backend);
                new_unhealthy_backends.push(b.backend);
            }
        }

        // check unhealthy backends
        while let Some(mut backend) = self.unhealthy_backends.pop() {
            backend.check_health().await;
            if backend.health() == Health::Healthy {
                info!("Backend {:?} is now healthy", backend);
                new_healthy_backends.push(MinHeapItem {
                    priority: backend.response_time_ms(),
                    backend,
                });
            } else {
                info!("Backend {:?} is still unhealthy", backend);
                new_unhealthy_backends.push(backend);
            }
        }

        self.healthy_backends = new_healthy_backends;
        self.unhealthy_backends = new_unhealthy_backends;

        // For profiling only, measures how much time it took to check all backends health
        let end_time = std::time::Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_millis();
        info!("checking all backends health took {}ms", elapsed_time);
        info!(
            "Healthy backends: {}, Unhealthy backends: {}",
            self.healthy_backends.len(),
            self.unhealthy_backends.len()
        );
    }
}
