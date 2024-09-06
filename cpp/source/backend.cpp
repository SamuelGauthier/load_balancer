#include <backend.h>
#include <spdlog/spdlog.h>

#include <string>

namespace load_balancer {

SimpleBackend::SimpleBackend(std::string address, int weight, Health health)
    : backend_address{address},
      backend_weight{weight},
      backend_health{health},
      client{drogon::HttpClient::newHttpClient(address)} {}

void SimpleBackend::check_health() {
  spdlog::info("Checking health of backend at {}", this->backend_address);

  auto start = std::chrono::high_resolution_clock::now();

  auto callback = [&, this](drogon::ReqResult result, const drogon::HttpResponsePtr &response) {
    if (result == drogon::ReqResult::Ok) {
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
  };
  auto request = drogon::HttpRequest::newHttpRequest();
  request->setPath("/health");
  client->sendRequest(request, callback);

  auto end = std::chrono::high_resolution_clock::now();
  auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
  spdlog::info("Health check of {} took {}ms", this->backend_address, duration.count());
}

Health SimpleBackend::health() { return this->backend_health; }

drogon::Task<drogon::HttpResponsePtr> SimpleBackend::send_request() {
  spdlog::info("Sending request to backend at {}", this->backend_address);

  auto start = std::chrono::high_resolution_clock::now();

  std::promise<drogon::HttpResponsePtr> response_promise;
  std::future<drogon::HttpResponsePtr> response_future = response_promise.get_future();

  auto callback = [&, this](drogon::ReqResult result, const drogon::HttpResponsePtr &response) {
    if (result == drogon::ReqResult::Ok) {
      if (this->backend_health == Health::Unhealthy) {
        spdlog::info("Backend at {} is now healthy", this->backend_address);
        this->backend_health = Health::Healthy;
      }
      spdlog::info("Request to {} was successful", this->backend_address);
      response_promise.set_value(response);
    } else {
      if (this->backend_health == Health::Healthy) {
        spdlog::info("Backend at {} is now unhealthy", this->backend_address);
        this->backend_health = Health::Unhealthy;
      }
      spdlog::error("Request to {} failed", this->backend_address);
      response_promise.set_value(drogon::HttpResponse::newHttpResponse(
          drogon::HttpStatusCode::k503ServiceUnavailable, drogon::ContentType::CT_TEXT_HTML));
    }
  };
  auto request = drogon::HttpRequest::newHttpRequest();
  request->setMethod(drogon::Get);
  request->setPath("/");

  client->sendRequest(request, callback);

  auto end = std::chrono::high_resolution_clock::now();
  auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
  spdlog::info("Sending request to {} took {}ms", this->backend_address, duration.count());

  co_return response_future.get();
}
std::string SimpleBackend::address() { return this->backend_address; }
int SimpleBackend::weight() { return this->backend_weight; }

} /* namespace load_balancer */
