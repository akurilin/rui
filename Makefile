.PHONY: help build run test format format-check lint precommit clean

APP ?= cui_app
RUN_ARGS ?=
ARGS ?=
DEFAULT_RUN_ARGS := --page test --width 2304 --height 1296
EFFECTIVE_RUN_ARGS := $(strip $(if $(RUN_ARGS),$(RUN_ARGS),$(if $(ARGS),$(ARGS),$(DEFAULT_RUN_ARGS))))

help:
	@echo "Rust workspace Makefile targets:"
	@echo "  make build        # cargo build -p $(APP)"
	@echo "  make run          # cargo run -p $(APP) -- <defaults or RUN_ARGS|ARGS>"
	@echo "  make test         # cargo test -p $(APP)"
	@echo "  make format       # cargo fmt --all"
	@echo "  make format-check # cargo fmt --all --check"
	@echo "  make lint         # cargo clippy -p $(APP) --all-targets --all-features"
	@echo "  make precommit    # format-check + lint + test"
	@echo "  make clean        # remove target/"

build:
	cargo build -p $(APP)

run:
	cargo run -p $(APP) -- $(EFFECTIVE_RUN_ARGS)

test:
	cargo test -p $(APP)

format:
	cargo fmt --all

format-check:
	cargo fmt --all --check

lint:
	cargo clippy -p $(APP) --all-targets --all-features

precommit: format-check lint test

clean:
	rm -rf target
