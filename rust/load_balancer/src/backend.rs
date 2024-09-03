use crate::health::Health;
use async_trait::async_trait;
use core::f32;
use reqwest::{Error, Response};
use std::fmt::Debug;

/// Represents a backend server resource to which the load balancer can forward the requests.
#[async_trait]
pub trait Backend: Send + Debug + BackendClone {
    /// Checks the health of the backend server by sending a request to the health check endpoint.
    /// If the server is healthy, the health status is set to Healthy, otherwise it is set to
    /// Unhealthy.
    async fn check_health(&mut self);

    /// Returns the health status of the backend server.
    fn health(&self) -> Health;

    /// Sends a request to the backend server and returns the response in case of success. If the
    /// request succeeds, the health status is updated to healthy. If the request fails, the health
    /// status of the backend server is set to Unhealthy.
    ///
    /// You should add arguments to this function to pass the request method, headers, body, etc.
    async fn send_request(&mut self) -> Result<Response, Error>;

    /// Returns the response time in milliseconds of the last request sent to the backend server.
    fn response_time_ms(&self) -> f32;
}

pub trait BackendClone {
    fn clone_box(&self) -> Box<dyn Backend>;
}

impl<T> BackendClone for T
where
    T: 'static + Backend + Clone,
{
    fn clone_box(&self) -> Box<dyn Backend> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Backend> {
    fn clone(&self) -> Box<dyn Backend> {
        self.clone_box()
    }
}
