#include <backend.h>

namespace load_balancer {

SimpleBackend::SimpleBackend(std::string address, Health health)
    : backend_address{address},
      backend_health{health},
      client{drogon::HttpClient::newHttpClient(address)},
      response_time_ms{std::chrono::milliseconds(0)} {}

drogon::Task<void> SimpleBackend::check_health() {
  spdlog::info("Checking health of backend at {}", this->backend_address);

  auto start = std::chrono::high_resolution_clock::now();
  auto request = drogon::HttpRequest::newHttpRequest();
  request->setPath("/health");
  try {
    auto response = co_await client->sendRequestCoro(request);
    if (response->getStatusCode() < drogon::HttpStatusCode::k200OK
        || response->getStatusCode() > drogon::HttpStatusCode::k206PartialContent) {
      spdlog::error("Health check of {} failed", this->backend_address);
    }
    this->update_health_from_status_code(response);
  } catch (const drogon::HttpException& e) {
    spdlog::error("Health check of {} failed: {}", this->backend_address, e.what());
    this->backend_health = Health::Unhealthy;
  }

  auto end = std::chrono::high_resolution_clock::now();
  auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
  this->response_time_ms = duration;
  spdlog::info("Health check of {} took {}ms", this->backend_address, duration.count());
}

Health SimpleBackend::health() { return this->backend_health; }

drogon::Task<drogon::HttpResponsePtr> SimpleBackend::send_request(drogon::HttpRequestPtr request) {
  spdlog::info("Sending request to backend at {}", this->backend_address);

  auto http_response = drogon::HttpResponse::newHttpResponse(
      drogon::HttpStatusCode::k503ServiceUnavailable, drogon::ContentType::CT_TEXT_HTML);

  auto start = std::chrono::high_resolution_clock::now();

  try {
    auto response = co_await client->sendRequestCoro(request);
    if (response->getStatusCode() >= drogon::HttpStatusCode::k200OK
        && response->getStatusCode() <= drogon::HttpStatusCode::k206PartialContent) {
      http_response = response;
    } else {
      spdlog::error("Request to {} failed", this->backend_address);
    }
    this->update_health_from_status_code(response);
  } catch (const drogon::HttpException& e) {
    spdlog::error("Request to {} failed: {}", this->backend_address, e.what());
    this->backend_health = Health::Unhealthy;
  }

  auto end = std::chrono::high_resolution_clock::now();
  auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
  this->response_time_ms = duration;
  spdlog::info("Sending request to {} took {}ms", this->backend_address, duration.count());

  co_return http_response;
}
std::string SimpleBackend::address() { return this->backend_address; }
std::chrono::milliseconds SimpleBackend::response_time() { return this->response_time_ms; }

void SimpleBackend::update_health_from_status_code(drogon::HttpResponsePtr response) {
  if (response->getStatusCode() >= drogon::HttpStatusCode::k200OK
      && response->getStatusCode() <= drogon::HttpStatusCode::k206PartialContent) {
    if (this->backend_health == Health::Unhealthy) {
      spdlog::info("Backend at {} is now healthy", this->backend_address);
      this->backend_health = Health::Healthy;
    }
    spdlog::info("Health check of {} was successful", this->backend_address);
  } else {
    if (this->backend_health == Health::Healthy) {
      spdlog::info("Backend at {} is now unhealthy", this->backend_address);
      this->backend_health = Health::Unhealthy;
    }
    spdlog::error("Health check of {} failed", this->backend_address);
  }
}

} /* namespace load_balancer */
