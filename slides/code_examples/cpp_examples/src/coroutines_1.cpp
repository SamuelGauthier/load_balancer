#include <coroutine>
#include <iostream>

struct Awaitable {
  bool await_ready() { return false; }
  void await_suspend(std::coroutine_handle<> h) {
    std::cout << "Suspending coroutine" << std::endl;
  }
  void await_resume() { std::cout << "Resuming coroutine" << std::endl; }
};

struct SimpleTask {
  struct promise_type {
    SimpleTask get_return_object() {
      return SimpleTask{
          std::coroutine_handle<promise_type>::from_promise(*this)};
    }
    std::suspend_never initial_suspend() { return {}; }
    std::suspend_always final_suspend() noexcept { return {}; }
    void return_void() {}
    void unhandled_exception() { std::terminate(); }
  };

  std::coroutine_handle<promise_type> handle;

  SimpleTask(std::coroutine_handle<promise_type> h) : handle(h) {}

  ~SimpleTask() {
    if (handle)
      handle.destroy();
  }

  void resume() {
    if (handle && !handle.done()) {
      handle.resume();
    }
  }
};

SimpleTask myCoroutine() {
  std::cout << "Start of coroutine" << std::endl;
  co_await Awaitable{};
  std::cout << "End of coroutine" << std::endl;
}

int main() {
  SimpleTask task = myCoroutine(); // Get the coroutine handle from the task
  task.resume();                   // Resume the coroutine after suspension
}
