#include <chrono>
#include <cstdlib>  // for std::system
#include <future>
#include <iostream>
#include <thread>

#ifdef _WIN32
#  include <Windows.h>  // For Sleep
#else
#  include <unistd.h>  // For sleep
#endif

void start_python_server(int port) {
  std::string command = "python3 -m http.server " + std::to_string(port);
  std::system(command.c_str());
}

std::future<void> run_server_async(int port) {
  // Run the server asynchronously using std::async
  return std::async(std::launch::async, start_python_server, port);
}

void stop_python_server(std::future<void>& server_future) {
  // Check if the server is still running
  if (server_future.valid()) {
    // This only works if the Python server can be terminated externally
    // A more robust solution may involve sending a specific request to shutdown

#ifdef _WIN32
    // On Windows, use the taskkill command to stop the Python server
    std::system("taskkill /IM python3.exe /F");
#else
    // On Unix-like systems, use pkill to terminate the Python server process
    std::system("pkill -f \"python3 -m http.server\"");
#endif

    // Ensure the server_future has finished
    server_future.wait();
  }
}

int main() {
  int port = 8080;
  std::cout << "Starting Python HTTP server on port " << port << "..." << std::endl;

  // Start the server asynchronously
  auto server_future = run_server_async(port);

  // Simulate some test operations
  std::cout << "Running tests..." << std::endl;
#ifdef _WIN32
  Sleep(5000);  // Sleep for 5 seconds to simulate test duration on Windows
#else
  sleep(5);  // Sleep for 5 seconds to simulate test duration on Unix-like systems
#endif

  // Stop the server after tests
  std::cout << "Stopping server..." << std::endl;
  stop_python_server(server_future);
  std::cout << "Server stopped." << std::endl;

  return 0;
}
