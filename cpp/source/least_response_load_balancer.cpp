#include <least_response_load_balancer.h>

namespace load_balancer {

LeastResponseLoadBalancer::LeastResponseLoadBalancer(std::vector<std::shared_ptr<Backend>> backends,
                                                     int health_check_interval_s)
    : health_check_interval_s{health_check_interval_s},
      current_backend_index{0},
      health_check_thread_running{false},
      backend_semaphore{1} {
  spdlog::info("Load balancer started with {} backends", backends.size());
  for (auto &backend : backends) {
    spdlog::info("Backend at {} with response time {}", backend->address(),
                 backend->response_time().count());
    this->healthy_backends.push(backend);
  }
}

drogon::Task<void> LeastResponseLoadBalancer::check_backend_healths() {
  spdlog::info("Checking health of all backends");

  auto start = std::chrono::high_resolution_clock::now();

  std::priority_queue<std::shared_ptr<Backend>, std::vector<std::shared_ptr<Backend>>,
                      MinHeapBackendComparator>
      new_healthy_backends;
  std::vector<std::shared_ptr<Backend>> new_unhealthy_backends;

  this->backend_semaphore.acquire();

  spdlog::info("Checking health of {} healthy backends", this->healthy_backends.size());
  while (!this->healthy_backends.empty()) {
    auto backend = this->healthy_backends.top();
    this->healthy_backends.pop();

    co_await backend->check_health();
    if (backend->health() == Health::Healthy) {
      new_healthy_backends.push(backend);
    } else {
      spdlog::warn("Backend at {} is unhealthy", backend->address());
      new_unhealthy_backends.push_back(backend);
    }
  }

  spdlog::info("Checking health of {} unhealthy backends", this->unhealthy_backends.size());
  for (auto backend : this->unhealthy_backends) {
    co_await backend->check_health();
    if (backend->health() == Health::Healthy) {
      spdlog::info("Backend at {} is now healthy", backend->address());
      new_healthy_backends.push(backend);
    } else {
      spdlog::warn("Backend at {} is still unhealthy", backend->address());
      new_unhealthy_backends.push_back(backend);
    }
  }

  this->healthy_backends = new_healthy_backends;
  this->unhealthy_backends = new_unhealthy_backends;

  this->backend_semaphore.release();

  auto end = std::chrono::high_resolution_clock::now();
  auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
  spdlog::info("Health check of all backends took {}ms", duration.count());

  co_return;
}

void LeastResponseLoadBalancer::start_health_checks() {
  this->health_check_thread_running = true;
  spdlog::info("Starting health checks every {}s", this->health_check_interval_s);

  drogon::async_run([this]() -> drogon::Task<void> {
    while (this->health_check_thread_running) {
      co_await this->check_backend_healths();
      co_await drogon::sleepCoro(trantor::EventLoop::getEventLoopOfCurrentThread(),
                                 std::chrono::seconds(this->health_check_interval_s));
    }
    spdlog::info("Stopped health checks");
    co_return;
  });
}

void LeastResponseLoadBalancer::stop_health_checks() {
  spdlog::info("Stopping health checks");
  this->health_check_thread_running = false;
}

drogon::Task<drogon::HttpResponsePtr> LeastResponseLoadBalancer::send_request(
    drogon::HttpRequestPtr request) {
  if (this->healthy_backends.empty()) {
    spdlog::error("No healthy backends available");
    throw std::runtime_error("No healthy backends available");
  }

  this->backend_semaphore.acquire();

  auto backend = this->healthy_backends.top();
  this->healthy_backends.pop();

  auto response = co_await backend->send_request(request);
  if (response->getStatusCode() < drogon::HttpStatusCode::k200OK
      || response->getStatusCode() > drogon::HttpStatusCode::k206PartialContent) {
    spdlog::error("Backend at {} returned status code {}", backend->address(),
                  static_cast<int>(response->getStatusCode()));
    this->unhealthy_backends.push_back(backend);
  } else {
    this->healthy_backends.push(backend);
  }

  this->backend_semaphore.release();

  co_return response;
}

} /* namespace load_balancer */
