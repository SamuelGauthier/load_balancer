SOURCES := $(wildcard src/*.cpp)
OBJECTS := $(SOURCES:src/%.cpp=build/%.o)

all: build/build.ninja
	ninja -C build

build/build.ninja: Makefile
	@mkdir -p build
	@echo "Generating build.ninja..."
	@echo "cxx_flags = -fsanitize=address,undefined,bounds,vptr,enum,signed-integer-overflow,null,unsigned-integer-overflow,return,integer-divide-by-zero,unreachable,alignment" > build/build.ninja
	@echo "cxx_version = -std=c++20" >> build/build.ninja
	@echo "rule cxx" >> build/build.ninja
	@echo "  command = g++ \$$cxx_version \$$cxx_flags \$$in -o \$$out" >> build/build.ninja
	$(foreach src, $(SOURCES), \
	    echo "build $(basename $(notdir $(src))): cxx ../$(src)" >> build/build.ninja;)

clean:
	rm -rf build

