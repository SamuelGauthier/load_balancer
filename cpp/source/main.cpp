#include <backend.h>
#include <drogon/drogon.h>
#include <health.h>
#include <least_response_load_balancer.h>
#include <round_robin_load_balancer.h>
#include <spdlog/spdlog.h>

#include <CLI/CLI.hpp>
#include <chrono>
#include <future>
#include <iostream>
#include <string>
#include <vector>

using namespace load_balancer;

int main(int argc, char *argv[]) {
  CLI::App app{
      "Load balancer listening on port 8080 and forwarding requests to a list of backend servers"};
  argv = app.ensure_utf8(argv);

  std::vector<std::string> backend_addresses{};
  app.add_option("-b,--backends", backend_addresses, "List of backend server addresses");
  int interval_health_check_s{10};
  app.add_option("-c,--health-check", interval_health_check_s,
                 "Time interval in seconds between health checks, defaults to 10s");
  bool dynamic_algorithm{false};
  app.add_flag("-d,--dynamic-algo", dynamic_algorithm,
               "Use dynamic algorithm (least response) to select the backend server");
  CLI11_PARSE(app, argc, argv);

  drogon::app().addListener("0.0.0.0", 8080);
  drogon::app().setThreadNum(4);

  std::vector<std::shared_ptr<Backend>> backends{};
  std::transform(backend_addresses.begin(), backend_addresses.end(), std::back_inserter(backends),
                 [](const std::string &address) {
                   return std::make_shared<SimpleBackend>(address, Health::Healthy);
                 });

  std::shared_ptr<LoadBalancer> load_balancer;
  if (dynamic_algorithm) {
    load_balancer = std::make_shared<LeastResponseLoadBalancer>(backends, interval_health_check_s);
  } else {
    load_balancer = std::make_shared<RoundRobinLoadBalancer>(backends, interval_health_check_s);
  }
  load_balancer->start_health_checks();

  drogon::app().registerHandlerViaRegex(
      "/.*",
      [&](drogon::HttpRequestPtr req,
          std::function<void(const drogon::HttpResponsePtr &)> callback) -> drogon::Task<> {
        spdlog::info("Received request from {}", req->getPeerAddr().toIpPort());
        spdlog::info("{} {} {}", req->methodString(), req->getPath(), req->versionString());
        spdlog::info("Host: {}", req->getHeader("host"));
        spdlog::info("User-Agent: {}", req->getHeader("user-agent"));
        spdlog::info("Accept: {}", req->getHeader("accept"));

        try {
          auto response = co_await load_balancer->send_request(req);
          callback(response);
        } catch (std::runtime_error &e) {
          auto response = drogon::HttpResponse::newHttpResponse();
          response->setStatusCode(drogon::HttpStatusCode::k503ServiceUnavailable);
          response->setBody("No healthy backends available");
          callback(response);
          /* co_return; */
        }
      });

  // Run HTTP framework,the method will block in the internal event loop
  drogon::app().run();
  load_balancer->stop_health_checks();
  return 0;
}
