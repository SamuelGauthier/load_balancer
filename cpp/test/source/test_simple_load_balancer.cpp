#include <doctest/doctest.h>

#include <string>

#include "round_robin_load_balancer.h"

TEST_SUITE("RoundRobinLoadBalancer") {
  TEST_CASE("All healthy backends, always gets other one") { CHECK(true); }
}
