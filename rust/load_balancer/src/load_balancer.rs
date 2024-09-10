use crate::backend::Backend;
use crate::internal_error::InternalError;
use async_trait::async_trait;

/// Load balancer interface
#[async_trait]
pub trait LoadBalancer: Send + Sync {
    /// Returns the next available backend server to which the request can be sent. If none are
    /// available, an error is returned.
    async fn next_available_backend(&self) -> Result<Box<dyn Backend>, String>;

    async fn send_request(&self) -> Result<String, InternalError>;

    async fn check_backends_healths(&self);
}
