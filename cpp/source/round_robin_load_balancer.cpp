#include <round_robin_load_balancer.h>

#include <atomic>
#include <memory>
#include <mutex>
#include <thread>
#include <vector>

namespace load_balancer {

RoundRobinLoadBalancer::RoundRobinLoadBalancer(std::vector<std::shared_ptr<Backend>> backends,
                                               int health_check_interval_s)
    : backends{backends},
      health_check_interval_s{health_check_interval_s},
      current_backend_index{0} {
  spdlog::info("Load balancer started with {} backends", backends.size());
  for (auto &backend : backends) {
    spdlog::info("Backend at {} with weight {}", backend->address(), backend->weight());
  }
}

std::shared_ptr<Backend> RoundRobinLoadBalancer::next_available_backend() {
  int tried_backends = 0;
  while (this->backends[this->current_backend_index]->health() == Health::Unhealthy) {
    this->current_backend_index = (this->current_backend_index + 1) % this->backends.size();
    tried_backends++;
    if (tried_backends >= (int)this->backends.size()) {
      spdlog::error("No healthy backends out of {} available", this->backends.size());
      throw std::runtime_error("No healthy backends available");
    }
  }

  auto index_to_return = this->current_backend_index;
  spdlog::info("Returning backend at {}", this->backends[index_to_return]->address());
  this->current_backend_index = (this->current_backend_index + 1) % this->backends.size();
  return this->backends[index_to_return];
}

void RoundRobinLoadBalancer::check_backend_healths() {
  spdlog::info("Checking health of all backends");

  auto start = std::chrono::high_resolution_clock::now();
  for (auto &backend : backends) {
    backend->check_health();
  }
  auto end = std::chrono::high_resolution_clock::now();
  auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
  spdlog::info("Health check of all backends took {}ms", duration.count());
}

void RoundRobinLoadBalancer::start_health_checks() {
  spdlog::info("Starting health checks every {}s", this->health_check_interval_s);
  this->health_check_thread = std::jthread([this](std::stop_token stop_token) {
    while (!stop_token.stop_requested()) {
      this->check_backend_healths();
      std::this_thread::sleep_for(std::chrono::seconds(this->health_check_interval_s));
    }
    spdlog::info("Stopped health checks");
  });
}
} /* namespace load_balancer */
