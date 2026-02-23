.PHONY: configure build test run clean c-configure c-build c-test c-run c-clean check-tools format format-check lint analyze precommit install-hooks submodules-init submodules-update

C_SOURCES := $(shell find . -type f -name '*.c' -not -path './build/*' -not -path './vendored/*' -not -path './rust/target/*' -not -path './Rust/target/*')
C_HEADERS := $(shell find . -type f -name '*.h' -not -path './build/*' -not -path './vendored/*' -not -path './rust/target/*' -not -path './Rust/target/*')
C_FILES := $(C_SOURCES) $(C_HEADERS)

UNAME := $(shell uname)
ifeq ($(UNAME),Darwin)
  LLVM_BIN ?= /opt/homebrew/opt/llvm/bin
  SDK_PATH := $(shell xcrun --show-sdk-path)
  TIDY_EXTRA_ARGS := --extra-arg=-isysroot --extra-arg=$(SDK_PATH)
else
  LLVM_BIN ?= /usr/bin
  TIDY_EXTRA_ARGS :=
endif

CLANG_FORMAT := $(LLVM_BIN)/clang-format
CLANG_TIDY := $(LLVM_BIN)/clang-tidy
SCAN_BUILD := $(LLVM_BIN)/scan-build
RUN_ARGS ?=
ARGS ?=
RUST_WORKSPACE_DIR ?= Rust
EFFECTIVE_RUN_ARGS := $(strip $(if $(RUN_ARGS),$(RUN_ARGS),$(ARGS)))

check-tools:
	@for tool in "$(CLANG_FORMAT)" "$(CLANG_TIDY)" "$(SCAN_BUILD)"; do \
		[ -x "$$tool" ] || { \
			echo "Missing required tool: $$tool"; \
			echo "Install Homebrew LLVM (brew install llvm), then retry."; \
			exit 1; \
		}; \
	done

configure:
	@echo "No configure step for Rust (default make targets now use the Rust port under $(RUST_WORKSPACE_DIR)/)."
	@echo "Use 'make c-configure' for the frozen C implementation."

build:
	cd $(RUST_WORKSPACE_DIR) && cargo build -p cui_app

test:
	cd $(RUST_WORKSPACE_DIR) && cargo test -p cui_app

run:
	cd $(RUST_WORKSPACE_DIR) && cargo run -p cui_app -- $(EFFECTIVE_RUN_ARGS)

clean:
	rm -rf build
	rm -rf $(RUST_WORKSPACE_DIR)/target

c-configure:
	cmake -S . -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON

c-build: c-configure
	cmake --build build

c-test: c-build
	ctest --test-dir build --output-on-failure

c-run: c-build
	@if [ -x ./build/Debug/cui ]; then \
		./build/Debug/cui $(EFFECTIVE_RUN_ARGS); \
	elif [ -x ./build/Release/cui ]; then \
		./build/Release/cui $(EFFECTIVE_RUN_ARGS); \
	elif [ -x ./build/cui ]; then \
		./build/cui $(EFFECTIVE_RUN_ARGS); \
	else \
		echo "Could not find executable: cui"; \
		echo "Expected one of: ./build/Debug/cui, ./build/Release/cui, ./build/cui"; \
		exit 1; \
	fi

format: check-tools
	$(CLANG_FORMAT) -i $(C_FILES)

format-check: check-tools
	$(CLANG_FORMAT) --dry-run --Werror $(C_FILES)

lint: check-tools configure
	$(CLANG_TIDY) $(C_SOURCES) -p build $(TIDY_EXTRA_ARGS)

analyze: check-tools
	cmake -S . -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON -DBUILD_TESTING=OFF
	$(SCAN_BUILD) --status-bugs --exclude vendored/SDL --exclude vendored/SDL_image cmake --build build --target cui

precommit: format-check lint analyze

install-hooks:
	git config core.hooksPath .githooks

submodules-init:
	git submodule update --init --recursive

submodules-update:
	git submodule update --init --recursive --remote
