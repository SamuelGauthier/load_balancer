#include <iostream>
#include <mutex>
#include <thread>
#include <vector>

using namespace std;

int main() {
  std::mutex mtx;
  int shared_counter = 0;

  auto increment_shared_counter = [&](int id) {
    for (int i = 0; i < 100; ++i) {
      std::lock_guard<std::mutex> lock(mtx);
      shared_counter++;
    }
  };

  const int num_threads = 3;
  std::vector<std::thread> threads;
  for (int i = 0; i < num_threads; ++i) {
    threads.emplace_back(increment_shared_counter, i);
  }

  for (auto &t : threads) {
    t.join();
  }

  std::cout << "Final shared_counter value: " << shared_counter << std::endl;

  return 0;
}
