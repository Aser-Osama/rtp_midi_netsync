# Test Configuration
# Defines all test cases and their properties

# ==============================================================================
# Test Definitions
# ==============================================================================

# List of all tests
TESTS := \
    basic \
    comprehensive \
    performance \
    stress \


# Test executable names (will be placed in $(BIN_DIR))
basic_TARGET := test_ffi
comprehensive_TARGET := test_ffi_comprehensive
performance_TARGET := test_ffi_performance
stress_TARGET := test_ffi_stress


# Test source files (in current directory)
basic_SRC := test_ffi.c
comprehensive_SRC := test_ffi_comprehensive.c
performance_SRC := test_ffi_performance.c
stress_SRC := test_ffi_stress.c



# Additional libraries for specific tests
basic_LIBS :=
comprehensive_LIBS :=
performance_LIBS := -lrt
stress_LIBS := -lrt


# Test descriptions (for help/info)
basic_DESC := Basic FFI functionality test
comprehensive_DESC := Comprehensive test suite
performance_DESC := Performance benchmarks
stress_DESC := Basic stress tests


# Test categories for grouping
BASIC_TESTS := basic
COMPREHENSIVE_TESTS := comprehensive
PERFORMANCE_TESTS := performance
STRESS_TESTS := stress
SAFETY_TESTS := $(STRESS_TESTS)
QUICK_TESTS := basic
ALL_TESTS := $(TESTS)
