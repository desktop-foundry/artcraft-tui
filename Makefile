# Makefile for artcraft-tui (Rust).
# `make` shows available targets.

SHELL := bash
.SHELLFLAGS := -eu -o pipefail -c
.ONESHELL:
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules
MAKEFLAGS += --no-builtin-variables
.DEFAULT_GOAL := help

V ?= 0
ifeq ($(V),1)
  Q :=
else
  Q := @
endif

export PATH := $(HOME)/.cargo/bin:$(PATH)
CARGO ?= cargo
BINARY := artcraft
TARGET := target/release/$(BINARY)

PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
DESTDIR ?=

ARGS ?=

##@ Build

.PHONY: build
build: ## Build the release binary
	$(Q)$(CARGO) build --release
	$(Q)echo "built $(TARGET)"

.PHONY: run
run: build ## Build and run (make run ARGS='...')
	$(Q)$(TARGET) $(ARGS)

.PHONY: check
check: fmt-check clippy test ## Run everything CI runs

##@ Quality

.PHONY: test
test: ## Run tests
	$(Q)$(CARGO) test --workspace

.PHONY: test-verbose
test-verbose: ## Run tests with output
	$(Q)$(CARGO) test --workspace -- --nocapture

.PHONY: clippy
clippy: ## Run clippy
	$(Q)$(CARGO) clippy --workspace -- -D warnings

.PHONY: fmt
fmt: ## Format code
	$(Q)$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check: ## Fail if any file is unformatted
	$(Q)$(CARGO) fmt --all -- --check

##@ Install

.PHONY: install
install: build ## Install the binary into PREFIX/bin
	$(Q)install -D -m 0755 $(TARGET) $(DESTDIR)$(BINDIR)/$(BINARY)
	$(Q)echo "installed $(DESTDIR)$(BINDIR)/$(BINARY)"

.PHONY: uninstall
uninstall: ## Remove the installed binary
	$(Q)rm -f $(DESTDIR)$(BINDIR)/$(BINARY)

.PHONY: clean
clean: ## Remove build artifacts
	$(Q)rm -rf target

##@ Help

.PHONY: help
help: ## Show this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} \
		/^[a-zA-Z0-9_%\/-]+:.*?##/ { printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2 } \
		/^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) }' $(MAKEFILE_LIST)
	@echo ""
