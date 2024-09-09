#include <round_robin_load_balancer.h>

namespace load_balancer {

RoundRobinLoadBalancer::RoundRobinLoadBalancer(std::vector<std::shared_ptr<Backend>> backends,
                                               int health_check_interval_s)
    : backends{backends},
      health_check_interval_s{health_check_interval_s},
      current_backend_index{0},
      health_check_thread_running{false},
      backend_semaphore{1} {
  spdlog::info("Load balancer started with {} backends", backends.size());
  for (auto &backend : backends) {
    spdlog::info("Backend at {} with response time {}", backend->address(),
                 backend->response_time().count());
  }
}

std::shared_ptr<Backend> RoundRobinLoadBalancer::next_available_backend() {
  int tried_backends = 0;

  this->backend_semaphore.acquire();
  while (this->backends[this->current_backend_index]->health() == Health::Unhealthy) {
    spdlog::info("Skipping unhealthy backend at {} with index {}",
                 this->backends[this->current_backend_index]->address(),
                 this->current_backend_index.load());
    this->current_backend_index = (this->current_backend_index + 1) % this->backends.size();

    tried_backends++;
    if (tried_backends >= (int)this->backends.size()) {
      spdlog::error("No healthy backends out of {} available", this->backends.size());
      throw std::runtime_error("No healthy backends available");
    }
  }

  auto index_to_return = this->current_backend_index.load();

  spdlog::info("Returning backend at {}", this->backends[index_to_return]->address());

  this->current_backend_index = (this->current_backend_index + 1) % this->backends.size();

  auto backend = this->backends[index_to_return];
  this->backend_semaphore.release();

  return backend;
}

drogon::Task<void> RoundRobinLoadBalancer::check_backend_healths() {
  spdlog::info("Checking health of all backends");

  auto start = std::chrono::high_resolution_clock::now();

  for (auto &backend : backends) {
    co_await backend->check_health();
  }

  auto end = std::chrono::high_resolution_clock::now();
  auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
  spdlog::info("Health check of all backends took {}ms", duration.count());
  co_return;
}

void RoundRobinLoadBalancer::start_health_checks() {
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

void RoundRobinLoadBalancer::stop_health_checks() {
  spdlog::info("Stopping health checks");
  this->health_check_thread_running = false;
}

drogon::Task<drogon::HttpResponsePtr> RoundRobinLoadBalancer::send_request(
    drogon::HttpRequestPtr request) {
  auto backend = this->next_available_backend();

  auto response = co_await backend->send_request(request);
  co_return response;
}

} /* namespace load_balancer */
