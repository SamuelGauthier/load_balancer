#ifndef LOAD_BALANCER
#define LOAD_BALANCER

#include <drogon/drogon.h>

namespace load_balancer {
class LoadBalancer {
public:
  virtual ~LoadBalancer() = default;

  virtual drogon::Task<drogon::HttpResponsePtr> send_request(drogon::HttpRequestPtr request) = 0;
};

}  // namespace load_balancer

#endif /* ifndef LOAD_BALANCER */
