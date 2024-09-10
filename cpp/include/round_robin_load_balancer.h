#ifndef ROUND_ROBIN_LOAD_BALANCER
#define ROUND_ROBIN_LOAD_BALANCER

#include <backend.h>
#include <load_balancer.h>

#include <atomic>
#include <memory>
#include <mutex>
#include <semaphore>
#include <vector>

namespace load_balancer {
class RoundRobinLoadBalancer : public LoadBalancer {
public:
  RoundRobinLoadBalancer(std::vector<std::shared_ptr<Backend>> backends,
                         int health_check_interval_s);

  std::shared_ptr<Backend> next_available_backend();
  drogon::Task<void> check_backend_healths();

  void start_health_checks() override;
  void stop_health_checks() override;
  drogon::Task<drogon::HttpResponsePtr> send_request(drogon::HttpRequestPtr request) override;

private:
  std::vector<std::shared_ptr<Backend>> backends;
  int health_check_interval_s;
  std::atomic<unsigned int> current_backend_index;
  std::atomic<bool> health_check_thread_running;
  std::counting_semaphore<1> backend_semaphore;
};

}  // namespace load_balancer

#endif /* ifndef ROUND_ROBIN_LOAD_BALANCER */
