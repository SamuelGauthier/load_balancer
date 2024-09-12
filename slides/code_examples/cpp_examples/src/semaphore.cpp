#include <chrono>
#include <iostream>
#include <semaphore>
#include <thread>
#include <vector>

using namespace std;

int main() {
  constexpr int MAX_COUNT = 3;
  counting_semaphore<MAX_COUNT> semaphore(MAX_COUNT);
  binary_semaphore cout_semaphore(1);

  auto print_sync = [&](const string &msg) {
    cout_semaphore.acquire();
    cout << msg << endl;
    cout_semaphore.release();
  };

  auto worker = [&](int id) {
    print_sync("Thread " + to_string(id) +
               " attempting to acquire the semaphore...");

    semaphore.acquire();

    print_sync("Thread " + to_string(id) + " acquired the semaphore.");

    this_thread::sleep_for(chrono::seconds(1));

    print_sync("Thread " + to_string(id) + " releasing the semaphore.");

    semaphore.release();
  };

  vector<thread> threads;

  for (int i = 1; i <= 5; i++) {
    threads.emplace_back(worker, i);
  }

  for (auto &t : threads) {
    t.join();
  }

  return 0;
}
