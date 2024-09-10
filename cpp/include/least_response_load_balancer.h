#ifndef LEAST_RESPONSE_LOAD_BALANCER
#define LEAST_RESPONSE_LOAD_BALANCER

#include <backend.h>
#include <load_balancer.h>

#include <atomic>
#include <memory>
#include <mutex>
#include <queue>
#include <semaphore>
#include <vector>

namespace load_balancer {
class LeastResponseLoadBalancer : public LoadBalancer {
public:
  LeastResponseLoadBalancer(std::vector<std::shared_ptr<Backend>> backends,
                            int health_check_interval_s);

  drogon::Task<void> check_backend_healths();

  void start_health_checks() override;
  void stop_health_checks() override;
  drogon::Task<drogon::HttpResponsePtr> send_request(drogon::HttpRequestPtr request) override;

private:
  struct MinHeapBackendComparator {
    bool operator()(const std::shared_ptr<Backend>& l, const std::shared_ptr<Backend>& r) const {
      return l->response_time() > r->response_time();
    }
  };

  std::priority_queue<std::shared_ptr<Backend>, std::vector<std::shared_ptr<Backend>>,
                      MinHeapBackendComparator>
      healthy_backends;
  std::vector<std::shared_ptr<Backend>> unhealthy_backends;
  int health_check_interval_s;
  std::atomic<unsigned int> current_backend_index;
  std::atomic<bool> health_check_thread_running;
  std::counting_semaphore<1> backend_semaphore;
};

}  // namespace load_balancer

#endif /* ifndef LEAST_RESPONSE_LOAD_BALANCER */
