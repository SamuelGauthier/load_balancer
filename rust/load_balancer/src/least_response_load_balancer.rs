use crate::backend::Backend;
use crate::health::Health;
use crate::internal_error::InternalError;
use crate::load_balancer::LoadBalancer;
use crate::min_heap_item::MinHeapItem;

use async_trait::async_trait;
use log::{error, info, warn};
use std::collections::BinaryHeap;
use tokio::sync::RwLock as TokioRwLock;

/// Represents a very basic load balancer. Sends the requests to healthy backend servers in a round
/// robin fashion.
#[derive(Debug)]
pub struct LeastResponseLoadBalancer {
    /// List of unhealthy backends servers
    unhealthy_backends: TokioRwLock<Vec<Box<dyn Backend>>>,

    /// Min heap of healthy backend servers. The heap is ordered by the response time of the
    /// backends
    healthy_backends: TokioRwLock<BinaryHeap<MinHeapItem<Box<dyn Backend>>>>,
}

impl LeastResponseLoadBalancer {
    /// Creates a new load balancer with the given list of backend servers to route the requests
    /// to.
    pub fn new(backends: Vec<Box<dyn Backend>>) -> Self {
        let mut healthy_backends = BinaryHeap::new();
        for backend in backends.into_iter() {
            healthy_backends.push(MinHeapItem {
                priority: 0.0,
                element: backend,
            });
        }
        Self {
            unhealthy_backends: TokioRwLock::new(Vec::new()),
            healthy_backends: TokioRwLock::new(healthy_backends),
        }
    }
}

#[async_trait]
impl LoadBalancer for LeastResponseLoadBalancer {
    // Returns the next available backend server to which the request can be sent. If none are
    // available, an error is returned.
    async fn next_available_backend(&self) -> Result<Box<dyn Backend>, String> {
        let r_healthy_backends = self.healthy_backends.read().await;
        if r_healthy_backends.is_empty() {
            return Err("No backend server available".to_string());
        }

        let MinHeapItem { element, .. } = r_healthy_backends.peek().unwrap();

        Ok(element.clone())
    }

    async fn send_request(&self) -> Result<String, InternalError> {
        let mut w_healthy_backends = self.healthy_backends.write().await;
        if w_healthy_backends.is_empty() {
            return Err(InternalError::NoBackendAvailable);
        }

        let MinHeapItem {
            element: backend, ..
        } = w_healthy_backends.pop().unwrap();

        // Send the request to the backend server
        let response = backend.send_request().await;
        match response {
            Ok(r) => {
                info!("{:?}", r);
                w_healthy_backends.push(MinHeapItem {
                    priority: backend.response_time_ms().await,
                    element: backend,
                });
                drop(w_healthy_backends);
                let body = r.text_with_charset("utf-8").await.unwrap();
                Ok(body)
            }
            Err(e) => {
                error!(
                    "Failed to send request to backend server: {:?}, trying next one",
                    e
                );
                let mut w_unhealthy_backends = self.unhealthy_backends.write().await;
                w_unhealthy_backends.push(backend);
                drop(w_unhealthy_backends);
                drop(w_healthy_backends);

                self.send_request().await
            }
        }
    }

    /// Checks and update the health status of all backend servers.
    async fn check_backends_healths(&self) {
        // This is used for profiling only
        let start_time = std::time::Instant::now();

        let mut new_healthy_backends = BinaryHeap::new();
        let mut new_unhealthy_backends: Vec<Box<dyn Backend>> = Vec::new();

        let mut w_healthy_backends = self.healthy_backends.write().await;
        // check healthy backends
        while let Some(MinHeapItem {
            element: backend, ..
        }) = w_healthy_backends.pop()
        {
            backend.check_health().await;
            if backend.health().await == Health::Healthy {
                let response_time = backend.response_time_ms().await;
                info!(
                    "Backend {:?} is healthy with response time {}ms",
                    backend, response_time
                );
                new_healthy_backends.push(MinHeapItem {
                    priority: response_time,
                    element: backend,
                });
            } else {
                warn!("Backend {:?} is unhealthy", backend);
                new_unhealthy_backends.push(backend);
            }
        }

        // check unhealthy backends
        let mut w_unhealthy_backends = self.unhealthy_backends.write().await;
        while let Some(backend) = w_unhealthy_backends.pop() {
            backend.check_health().await;
            if backend.health().await == Health::Healthy {
                info!("Backend {:?} is now healthy", backend);
                new_healthy_backends.push(MinHeapItem {
                    priority: backend.response_time_ms().await,
                    element: backend,
                });
            } else {
                info!("Backend {:?} is still unhealthy", backend);
                new_unhealthy_backends.push(backend);
            }
        }

        *w_healthy_backends = new_healthy_backends;
        *w_unhealthy_backends = new_unhealthy_backends;
        let healthy_backends_count = w_healthy_backends.len();
        let unhealthy_backends_count = w_unhealthy_backends.len();

        let best_backend = w_healthy_backends.peek().unwrap();
        let best_backend_priority = best_backend.priority;
        let best_backend_address: String = best_backend.element.address().into();

        drop(w_healthy_backends);
        drop(w_unhealthy_backends);

        // For profiling only, measures how much time it took to check all backends health
        let end_time = std::time::Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_millis();
        info!("checking all backends health took {}ms", elapsed_time);
        info!(
            "Best backend: {}ms, {}",
            best_backend_priority, best_backend_address
        );
        info!(
            "Healthy backends: {}, Unhealthy backends: {}",
            healthy_backends_count, unhealthy_backends_count
        );
    }
}
