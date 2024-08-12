#include <iostream>
#include <thread>

int data = 0;

void write() {
  data = 42; // Mutable access
}

void read() {
  std::cout << data << std::endl; // Concurrent immutable access
}

int main() {
  std::thread t1(write);
  std::thread t2(read);

  t1.join();
  t2.join();

  return 0;
}
