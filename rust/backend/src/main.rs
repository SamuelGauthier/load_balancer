/*
 * Backend server that listens on a given port and returns a hello message
 *
 * Author: Samuel Gauthier
 */
use clap::Parser;
use log::info;
use ntex::time::sleep;
use ntex::web;
use simple_logger;
use std::sync::{Arc, Mutex};

/// State of the backend server. Contains the name of the server and the number of times it has
/// been called
#[derive(Debug, Clone)]
struct State {
    /// Name of the backend server
    name: String,

    /// Number of times the backend server has been called, for profiling and debugging purposes
    times_called: u64,

    /// Delay in seconds before responding to a request
    delay_ms: u64,
}

impl State {
    /// Creates a new state with the given name
    fn new(name: String, delay_ms: u64) -> Self {
        State {
            name,
            times_called: 0,
            delay_ms,
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

    /// Delay in seconds before responding to a request
    #[arg(short, long, default_value = "0")]
    delay_ms: u64,
}

/// Prints information about the incoming request
fn print_request_info(request: &web::HttpRequest) {
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
}

/// Index endpoint that returns a hello message containing the name of the backend server
async fn index(
    state: web::types::State<Arc<Mutex<State>>>,
    request: web::HttpRequest,
) -> Result<String, web::Error> {
    print_request_info(&request);
    let mut state = state.lock().unwrap();

    if state.delay_ms > 0 {
        info!("Sleeping for {} milliseconds", state.delay_ms);
        sleep(std::time::Duration::from_millis(state.delay_ms)).await;
    }

    info!("Replied with a hello message from {}", state.name);
    state.times_called += 1;
    info!(
        "Backend server has been called {} times",
        state.times_called
    );

    Ok(format!("Hello from backend server: {}", state.name))
}

/// Health check endpoint that returns an empty string
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
    let state = Arc::new(Mutex::new(State::new(args.name.clone(), args.delay_ms)));

    web::HttpServer::new(move || {
        web::App::new()
            .state(state.clone())
            .service(health_check)
            .default_service(web::to(index))
    })
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}
