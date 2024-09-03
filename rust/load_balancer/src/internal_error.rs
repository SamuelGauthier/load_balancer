use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum InternalError {
    NoBackendAvailable,
    BackendUnreachable,
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternalError::NoBackendAvailable => {
                write!(f, "No load balancer available")
            }
            InternalError::BackendUnreachable => {
                write!(f, "Backend server unreachable")
            }
        }
    }
}

impl Error for InternalError {}
