#include <cstdio>

int a() { return std::puts("a"); }
int b() { return std::puts("b"); }
int c() { return std::puts("c"); }

void z(int, int, int) {}

int main() {
  z(a(), b(), c());       // all 6 permutations of output are allowed
  return a() + b() + c(); // all 6 permutations of output are allowed
}
