#include <drogon/drogon.h>
#include <spdlog/spdlog.h>

#include <CLI/CLI.hpp>
#include <chrono>
#include <future>
#include <iostream>
#include <string>
#include <vector>

enum class Health { Healthy, Unhealthy };

class Backend {
public:
  Backend(std::string address, int weight, Health health)
      : backend_address{address},
        backend_weight{weight},
        backend_health{health},
        client{drogon::HttpClient::newHttpClient(address)} {}

  void check_health() {
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

  Health health() { return this->backend_health; }

  drogon::HttpResponsePtr send_request() {
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
    auto resp = co_await client->sendRequest(request, callback);

    auto end = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    spdlog::info("Sending request to {} took {}ms", this->backend_address, duration.count());

    return response_future.get();
  }
  std::string address() { return this->backend_address; }
  int weight() { return this->backend_weight; }

private:
  std::string backend_address;
  int backend_weight;
  Health backend_health;
  drogon::HttpClientPtr client;
};

class LoadBalancer {
public:
  LoadBalancer(std::vector<std::shared_ptr<Backend>> backends, int health_check_interval)
      : backends{backends}, health_check_interval{health_check_interval}, current_backend_index{0} {
    spdlog::info("Load balancer started with {} backends", backends.size());
    for (auto &backend : backends) {
      spdlog::info("Backend at {} with weight {}", backend->address(), backend->weight());
    }
  }

  std::shared_ptr<Backend> next_available_backend() {
    int tried_backends = 0;
    while (this->backends[this->current_backend_index]->health() == Health::Unhealthy) {
      this->current_backend_index = (this->current_backend_index + 1) % this->backends.size();
      tried_backends++;
      if (tried_backends >= this->backends.size()) {
        spdlog::error("No healthy backends out of {} available", this->backends.size());
        throw std::runtime_error("No healthy backends available");
      }
    }

    auto index_to_return = this->current_backend_index;
    spdlog::info("Returning backend at {}", this->backends[index_to_return]->address());
    this->current_backend_index = (this->current_backend_index + 1) % this->backends.size();
    return this->backends[index_to_return];
  }

  void check_backend_healths() {
    auto start = std::chrono::high_resolution_clock::now();
    for (auto &backend : backends) {
      backend->check_health();
    }
    auto end = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    spdlog::info("Health check of all backends took {}ms", duration.count());
  }

private:
  std::vector<std::shared_ptr<Backend>> backends;
  int health_check_interval;
  unsigned int current_backend_index;
};

int main(int argc, char *argv[]) {
  CLI::App app{
      "Load balancer listening on port 8080 and forwarding requests to a list of backend servers"};
  argv = app.ensure_utf8(argv);

  std::vector<std::string> backend_addresses{};
  app.add_option("-b,--backends", backend_addresses, "List of backend server addresses");
  int interval_health_check{10};
  app.add_option("-c,--health-check", interval_health_check,
                 "Time interval in miliseconds between health checks, defaults to 10ms");
  CLI11_PARSE(app, argc, argv);

  drogon::app().addListener("0.0.0.0", 8080);
  drogon::app().setNumThreads(16);

  std::vector<std::shared_ptr<Backend>> backends{};
  std::transform(backend_addresses.begin(), backend_addresses.end(), std::back_inserter(backends),
                 [](const std::string &address) {
                   return std::make_shared<Backend>(address, 1, Health::Healthy);
                 });

  auto load_balancer = std::make_shared<LoadBalancer>(backends, interval_health_check);

  using Callback = std::function<void(const drogon::HttpResponsePtr &)>;

  drogon::app().registerHandler("/", [&](const drogon::HttpRequestPtr &req, Callback &&callback) {
    spdlog::info("Received request from {}", req->getPeerAddr().toIpPort());
    spdlog::info("{} {} {}", req->methodString(), req->getPath(), req->versionString());
    spdlog::info("Host: {}", req->getHeader("host"));
    spdlog::info("User-Agent: {}", req->getHeader("user-agent"));
    spdlog::info("Accept: {}", req->getHeader("accept"));

    try {
      auto backend = load_balancer->next_available_backend();
      auto response = backend->send_request();
      callback(response);
    } catch (std::runtime_error &e) {
      auto response = drogon::HttpResponse::newHttpResponse();
      response->setStatusCode(drogon::HttpStatusCode::k503ServiceUnavailable);
      response->setBody("No healthy backends available");
      callback(response);
      return;
    }
  });

  // Run HTTP framework,the method will block in the internal event loop
  drogon::app().run();
  return 0;
}
