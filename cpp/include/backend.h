#ifndef BACKEND
#define BACKEND

#include <drogon/drogon.h>
#include <health.h>
#include <spdlog/spdlog.h>

#include <string>

namespace load_balancer {

class Backend {
public:
  virtual ~Backend() = default;

  virtual void check_health() = 0;
  virtual Health health() = 0;
  virtual drogon::Task<drogon::HttpResponsePtr> send_request() = 0;
  virtual std::string address() = 0;
  virtual int weight() = 0;
};

class SimpleBackend : public Backend {
public:
  SimpleBackend(std::string address, int weight, Health health);

  void check_health() override;
  Health health() override;
  drogon::Task<drogon::HttpResponsePtr> send_request() override;
  std::string address() override;
  int weight() override;

private:
  std::string backend_address;
  int backend_weight;
  std::atomic<Health> backend_health;
  drogon::HttpClientPtr client;
};
} /* namespace load_balancer */
#endif /* ifndef BACKEND */
