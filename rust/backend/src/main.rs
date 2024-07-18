/*
 * Backend server that listens on a given port and returns a hello message
 *
 * Author: Samuel Gauthier
 */
use clap::Parser;
use log::info;
use ntex::web;
use simple_logger;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct State {
    name: String,
    times_called: u64,
}

impl State {
    fn new(name: String) -> Self {
        State {
            name,
            times_called: 0,
        }
    }
}

/// Backend server that listens on a given port and returns a hello message
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port on which to run the backend server
    #[arg(short, long, default_value = "8081")]
    port: u16,

    /// Name of the backend server
    #[arg(short, long, default_value = "backend-server")]
    name: String,
}

#[web::get("/")]
async fn index(
    state: web::types::State<Arc<Mutex<State>>>,
    request: web::HttpRequest,
) -> Result<String, web::Error> {
    info!(
        "Received request from {}",
        request.connection_info().remote().unwrap()
    );
    info!(
        "{} {} {:?}",
        request.head().method,
        request.head().uri,
        request.head().version
    );
    for (key, value) in request.headers().iter() {
        info!("{}: {}", key, value.to_str().unwrap());
    }
    let mut state = state.lock().unwrap();

    info!("Replied with a hello message from {}", state.name);
    state.times_called += 1;
    info!(
        "Backend server has been called {} times",
        state.times_called
    );

    Ok(format!("Hello from backend server: {}", state.name))
}

#[web::get("/health")]
async fn health_check(request: web::HttpRequest) -> Result<String, web::Error> {
    info!(
        "Received health check request from {}",
        request.connection_info().remote().unwrap()
    );

    Ok("".to_string())
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let args = Args::parse();
    let state = Arc::new(Mutex::new(State::new(args.name.clone())));

    web::HttpServer::new(move || {
        web::App::new()
            .state(state.clone())
            .service(index)
            .service(health_check)
    })
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}
