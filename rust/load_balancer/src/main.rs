/*
 * A simple load balancer listening on port 8080 and forwarding requests to a backend server
 *
 * Author: Samuel Gauthier
 */
mod backend;
mod geo_load_balancer;
mod health;
mod internal_error;
mod least_response_load_balancer;
mod load_balancer;
mod simple_backend;
mod simple_load_balancer;

use crate::load_balancer::LoadBalancer;

use backend::Backend;
use health::Health;
use least_response_load_balancer::LeastResponseLoadBalancer;
use simple_backend::SimpleBackend;
use simple_load_balancer::SimpleLoadBalancer;

use actix_web;
use actix_web::error::InternalError;
use actix_web::http::StatusCode;
use clap::Parser;
use log::{error, info};
use simple_logger;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// Prints the request information to the log. Used for debugging purposes only.
async fn print_request_info(request: actix_web::HttpRequest) {
    info!(
        "Received request from {}",
        request.connection_info().peer_addr().unwrap()
    );
    info!(
        "{} {} {:?}",
        request.head().method,
        request.head().uri,
        request.head().version,
    );
    for (key, value) in request.headers().iter() {
        info!("{}: {}", key, value.to_str().unwrap());
    }
}

/// Index route of the load balancer. Forwards the request to the next available backend server.
async fn index(
    load_balancer: actix_web::web::Data<Arc<TokioMutex<Box<dyn LoadBalancer>>>>,
    request: actix_web::HttpRequest,
) -> Result<String, actix_web::Error> {
    print_request_info(request).await;

    // Extract the load balancer from the state and get the next available backend server
    let mut lb = load_balancer.get_ref().lock().await;
    let request_response = lb.send_request().await;
    match request_response {
        Ok(r) => Ok(r),
        Err(e) => {
            error!("Failed to send request to backend server: {:?}", e);
            return Err(InternalError::new(
                "Failed to send request to backend server",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into());
        }
    }
}

/// Load balancer listening on port 8080 and forwarding requests to a list of backend servers
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Time interval in miliseconds between health checks
    #[arg(short, long, default_value = "10")]
    interval_health_check: u64,

    /// List of backend servers
    backend_adresses: Vec<String>,

    /// Dynamic load balancer
    #[arg(short, long, default_value = "false")]
    dynamic: bool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let args = Args::parse();

    let backends = args
        .backend_adresses
        .iter()
        .map(|address| {
            Box::new(SimpleBackend::new(address.clone(), Health::Healthy)) as Box<dyn Backend>
        })
        .collect();

    let load_balancer: Arc<TokioMutex<Box<dyn LoadBalancer>>> = if args.dynamic {
        LeastResponseLoadBalancer::new(backends, args.interval_health_check)
    } else {
        SimpleLoadBalancer::new(backends, args.interval_health_check)
    };

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(load_balancer.clone()))
            .default_service(actix_web::web::to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
