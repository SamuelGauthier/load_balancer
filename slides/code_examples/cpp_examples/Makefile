SOURCES := $(wildcard src/*.cpp)
OBJECTS := $(SOURCES:src/%.cpp=build/%.o)

all: build/build.ninja
	ninja -C build

build/build.ninja: Makefile
	@mkdir -p build
	@echo "Generating build.ninja..."
	@echo "cxx_flags =  -fsanitize=address,undefined,bounds,vptr,enum,signed-integer-overflow,null,unsigned-integer-overflow,return,integer-divide-by-zero,unreachable,alignment -isysroot /opt/homebrew/opt/llvm/include -fexperimental-library -std=gnu++20 -arch arm64 -isysroot /Library/Developer/CommandLineTools/SDKs/MacOSX14.4.sdk" > build/build.ninja
	@echo "ldflags = -L/opt/homebrew/opt/llvm/lib" >> build/build.ninja
	@echo "compiler = /opt/homebrew/opt/llvm/bin/clang++" >> build/build.ninja
	@echo "rule cxx" >> build/build.ninja
	@echo "  command = \$$compiler \$$cxx_flags \$$in -o \$$out \$$ldflags" >> build/build.ninja
	$(foreach src, $(SOURCES), \
	    echo "build $(basename $(notdir $(src))): cxx ../$(src)" >> build/build.ninja;)

clean:
	rm -rf build

