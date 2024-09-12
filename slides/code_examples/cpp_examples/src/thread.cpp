#include <chrono>
#include <iostream>
#include <thread>

using namespace std;

void worker(stop_token stopToken) {
  while (!stopToken.stop_requested()) {
    cout << "Working..." << endl;
    this_thread::sleep_for(chrono::seconds(1));
  }
  cout << "Stopping gracefully..." << endl;
}

int main() {
  jthread thread(worker);
  this_thread::sleep_for(chrono::seconds(5));
  cout << "Main thread finished." << endl;
  return 0;
}
