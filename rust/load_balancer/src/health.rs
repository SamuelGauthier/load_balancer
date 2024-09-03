/// Servers are defined as either healthy or unhealthy. In the case of unhealthy servers, the load
/// balancer will not forward requests to them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Health {
    Healthy,
    Unhealthy,
}
