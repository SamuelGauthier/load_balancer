#ifndef ROUND_ROBIN_LOAD_BALANCER
#define ROUND_ROBIN_LOAD_BALANCER

#include <backend.h>

#include <atomic>
#include <memory>
#include <mutex>
#include <thread>
#include <vector>

namespace load_balancer {
class RoundRobinLoadBalancer {
public:
  RoundRobinLoadBalancer(std::vector<std::shared_ptr<Backend>> backends,
                         int health_check_interval_s);

  std::shared_ptr<Backend> next_available_backend();

  void check_backend_healths();
  void start_health_checks();

private:
  std::vector<std::shared_ptr<Backend>> backends;
  int health_check_interval_s;
  unsigned int current_backend_index;
  std::jthread health_check_thread;
  std::mutex health_check_mutex;
};

}  // namespace load_balancer

#endif /* ifndef ROUND_ROBIN_LOAD_BALANCER */
