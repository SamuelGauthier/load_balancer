#ifndef BACKEND
#define BACKEND

#include <drogon/drogon.h>
#include <health.h>
#include <spdlog/spdlog.h>

#include <atomic>
#include <chrono>
#include <string>

namespace load_balancer {

class Backend {
public:
  virtual ~Backend() = default;

  virtual drogon::Task<void> check_health() = 0;
  virtual Health health() = 0;
  virtual drogon::Task<drogon::HttpResponsePtr> send_request(drogon::HttpRequestPtr request) = 0;
  virtual std::string address() = 0;
  virtual std::chrono::milliseconds response_time() = 0;
};

class SimpleBackend : public Backend {
public:
  SimpleBackend(std::string address, Health health);

  drogon::Task<void> check_health() override;
  Health health() override;
  drogon::Task<drogon::HttpResponsePtr> send_request(drogon::HttpRequestPtr request) override;
  std::string address() override;
  std::chrono::milliseconds response_time() override;

private:
  std::string backend_address;
  std::atomic<Health> backend_health;
  drogon::HttpClientPtr client;
  std::atomic<std::chrono::milliseconds> response_time_ms;

  void update_health_from_status_code(drogon::HttpResponsePtr response);
};
} /* namespace load_balancer */
#endif /* ifndef BACKEND */
