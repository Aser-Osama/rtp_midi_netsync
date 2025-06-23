# Makefile for C FFI tests

# ==============================================================================
# Directory Configuration
# ==============================================================================

# Directory structure
BIN_DIR := bin
BUILD_DIR := build
SRC_DIR := .

# Include test configuration
include test_config.mk

# ==============================================================================
# Build Configuration
# ==============================================================================

# Compiler settings
CC := gcc
CFLAGS := -Wall -Wextra -std=c99 -O2 -D_POSIX_C_SOURCE=199309L
DEBUG_CFLAGS := -Wall -Wextra -std=c99 -g -O0 -D_POSIX_C_SOURCE=199309L -DDEBUG
INCLUDES := -I../include

# Build configuration (can be overridden: make BUILD=debug)
BUILD ?= release
ifeq ($(BUILD),debug)
    CFLAGS := $(DEBUG_CFLAGS)
    RUST_TARGET := debug
else
    RUST_TARGET := release
endif

# Rust library configuration
RUST_LIB_DIR := ../target/$(RUST_TARGET)
RUST_LIB := $(RUST_LIB_DIR)/librtp_midi_netsync.a
LDFLAGS := -L$(RUST_LIB_DIR) -lrtp_midi_netsync

# Generate full paths for all targets
ALL_TARGETS := $(foreach test,$(TESTS),$(BIN_DIR)/$($(test)_TARGET))

# ==============================================================================
# Build Rules
# ==============================================================================

.PHONY: all clean test rust-lib help install check-deps dirs
.PHONY: $(TESTS) debug release safety info lint format
.PHONY: clean-bins clean-build deep-clean
.DEFAULT_GOAL := all

# Main targets
all: dirs $(ALL_TARGETS)

debug:
	$(MAKE) BUILD=debug all

release:
	$(MAKE) BUILD=release all

# Create necessary directories
dirs:
	@mkdir -p $(BIN_DIR) $(BUILD_DIR)

# Build the Rust library
rust-lib:
	@echo "Building Rust library ($(RUST_TARGET))..."
	cd .. && cargo build $(if $(filter release,$(RUST_TARGET)),--release)

# Check if Rust library exists and is up to date
$(RUST_LIB): rust-lib

# Generic build rule for all tests
define BUILD_TEST_RULE
$(BIN_DIR)/$($(1)_TARGET): $(SRC_DIR)/$($(1)_SRC) $(RUST_LIB) | dirs
	@echo "Building $(1) test: $($(1)_DESC) ($(BUILD) mode)..."
	$(CC) $(CFLAGS) $(INCLUDES) -o $$@ $$< $(LDFLAGS) $($(1)_LIBS)
endef

# Generate build rules for all tests
$(foreach test,$(TESTS),$(eval $(call BUILD_TEST_RULE,$(test))))

# ==============================================================================
# Test Execution Rules
# ==============================================================================

# Generic test runner
define RUN_TEST_RULE
$(1): $(BIN_DIR)/$($(1)_TARGET)
	@echo "=== Running $(1) test: $($(1)_DESC) ==="
	@cd $(BIN_DIR) && ./$($(1)_TARGET)
	@echo ""
endef

# Generate run rules for all tests
$(foreach test,$(TESTS),$(eval $(call RUN_TEST_RULE,$(test))))

# Test suites
basic-tests: $(BASIC_TESTS)
	@echo "=== Basic test suite completed ==="

comprehensive-tests: $(COMPREHENSIVE_TESTS)
	@echo "=== Comprehensive test suite completed ==="

performance-tests: $(PERFORMANCE_TESTS)
	@echo "=== Performance test suite completed ==="

stress-tests: $(STRESS_TESTS)
	@echo "=== Stress test suite completed ==="

safety: $(SAFETY_TESTS)
	@echo "=== All safety/security tests completed ==="

quick: $(QUICK_TESTS)
	@echo "=== Quick test suite completed ==="

test: $(ALL_TESTS)
	@echo "=== All tests completed successfully ==="

# ==============================================================================
# Utility Targets
# ==============================================================================

# Check dependencies
check-deps:
	@echo "Checking dependencies..."
	@which $(CC) >/dev/null || (echo "Error: $(CC) not found" && exit 1)
	@which cargo >/dev/null || (echo "Error: cargo not found" && exit 1)
	@echo "All dependencies found."

# Install is now just ensuring bins are built (they're already in bin/)
install: all
	@echo "All executables are available in $(BIN_DIR)/"
	@ls -la $(BIN_DIR)/

# Lint C code (if available)
lint:
	@if which clang-tidy >/dev/null 2>&1; then \
		echo "Running clang-tidy..."; \
		clang-tidy $(foreach test,$(TESTS),$(SRC_DIR)/$($(test)_SRC)) -- $(INCLUDES); \
	else \
		echo "clang-tidy not available, skipping lint"; \
	fi

# Format C code (if available)
format:
	@if which clang-format >/dev/null 2>&1; then \
		echo "Formatting C code..."; \
		clang-format -i $(foreach test,$(TESTS),$(SRC_DIR)/$($(test)_SRC)); \
		echo "Code formatting completed."; \
	else \
		echo "clang-format not available, skipping format"; \
	fi

# Clean targets
clean-bins:
	@echo "Cleaning binary directory..."
	rm -rf $(BIN_DIR)

clean-build:
	@echo "Cleaning build directory..."
	rm -rf $(BUILD_DIR)

clean: clean-bins clean-build
	@echo "Clean completed."

# Deep clean (including Rust artifacts)
deep-clean: clean
	@echo "Cleaning Rust artifacts..."
	cd .. && cargo clean

# Show build information
info:
	@echo "Build Configuration:"
	@echo "  BUILD_MODE: $(BUILD)"
	@echo "  CC: $(CC)"
	@echo "  CFLAGS: $(CFLAGS)"
	@echo "  LDFLAGS: $(LDFLAGS)"
	@echo "  RUST_TARGET: $(RUST_TARGET)"
	@echo "  RUST_LIB: $(RUST_LIB)"
	@echo ""
	@echo "Directories:"
	@echo "  SRC_DIR: $(SRC_DIR)"
	@echo "  BIN_DIR: $(BIN_DIR)"
	@echo "  BUILD_DIR: $(BUILD_DIR)"
	@echo ""
	@echo "Available tests:"
	@$(foreach test,$(TESTS),echo "  $(test): $($(test)_DESC)";)

# List available binaries
list-bins:
	@echo "Available test executables:"
	@if [ -d $(BIN_DIR) ]; then \
		ls -la $(BIN_DIR)/; \
	else \
		echo "No binaries built yet. Run 'make all' to build them."; \
	fi

# Help target
help:
	@echo "C FFI Test Suite Makefile"
	@echo "========================="
	@echo ""
	@echo "Directory Structure:"
	@echo "  Source files:     $(SRC_DIR)/"
	@echo "  Executables:      $(BIN_DIR)/"
	@echo "  Build artifacts:  $(BUILD_DIR)/"
	@echo ""
	@echo "Build Targets:"
	@echo "  all              - Build all test executables"
	@echo "  debug            - Build all tests in debug mode"
	@echo "  release          - Build all tests in release mode (default)"
	@echo "  rust-lib         - Build the Rust library"
	@echo "  dirs             - Create necessary directories"
	@echo ""
	@echo "Test Suites:"
	@echo "  basic-tests      - Run basic functionality tests"
	@echo "  comprehensive-tests - Run comprehensive test suite"
	@echo "  performance-tests   - Run performance tests"
	@echo "  stress-tests     - Run stress tests"
	@echo "  safety           - Run all safety/security tests"
	@echo "  quick            - Run quick test suite"
	@echo "  test             - Run all tests"
	@echo ""
	@echo "Individual Tests:"
	@$(foreach test,$(TESTS),echo "  $(test)$(shell printf '%*s' $$((16 - $(shell echo $(test) | wc -c))) '') - $($(test)_DESC)";)
	@echo ""
	@echo "Utility Targets:"
	@echo "  check-deps       - Check if required dependencies are available"
	@echo "  install          - Show location of built executables"
	@echo "  list-bins        - List available test executables"
	@echo "  lint             - Run clang-tidy (if available)"
	@echo "  format           - Format C code with clang-format (if available)"
	@echo "  info             - Show build configuration"
	@echo ""
	@echo "Clean Targets:"
	@echo "  clean-bins       - Clean binary directory"
	@echo "  clean-build      - Clean build directory"
	@echo "  clean            - Clean all build artifacts"
	@echo "  deep-clean       - Clean all artifacts including Rust"
	@echo ""
	@echo "Build Options:"
	@echo "  BUILD=debug      - Build in debug mode"
	@echo "  BUILD=release    - Build in release mode (default)"
	@echo ""
	@echo "Examples:"
	@echo "  make test                    # Run all tests in release mode"
	@echo "  make BUILD=debug quick       # Run quick tests in debug mode"
	@echo "  make safety                  # Run security-focused tests"
	@echo "  make list-bins               # Show built executables"
