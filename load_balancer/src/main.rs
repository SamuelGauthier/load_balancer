/*
 * A simple load balancer listening on port 8080 and forwarding requests to a backend server
 *
 * Author: Samuel Gauthier
 */
use log::{error, info};
use ntex::{
    http::{client::Client, StatusCode},
    web,
    web::error::InternalError,
};
use simple_logger;

#[web::get("/")]
async fn index(request: web::HttpRequest) -> Result<String, web::Error> {
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

    // make call to backend server and return the answer
    let backend_address = "http://localhost:8081/";
    let response = Client::new().get(backend_address).send().await;

    match response {
        Ok(mut r) => {
            info!("{:?}", r);
            let body = r.body().await?;
            let body_string = String::from_utf8(body.to_vec()).unwrap();
            Ok(body_string)
        }
        Err(e) => {
            error!("Failed to send request to backend server: {:?}", e);
            Err(InternalError::new(
                "Failed to send request to backend server",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into())
        }
    }
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    web::HttpServer::new(|| web::App::new().service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
