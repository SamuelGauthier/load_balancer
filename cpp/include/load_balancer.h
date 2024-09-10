#ifndef LOAD_BALANCER
#define LOAD_BALANCER

#include <drogon/drogon.h>

namespace load_balancer {
class LoadBalancer {
public:
  virtual ~LoadBalancer() = default;

  virtual void start_health_checks() = 0;
  virtual void stop_health_checks() = 0;
  virtual drogon::Task<drogon::HttpResponsePtr> send_request(drogon::HttpRequestPtr request) = 0;
};

}  // namespace load_balancer

#endif /* ifndef LOAD_BALANCER */
