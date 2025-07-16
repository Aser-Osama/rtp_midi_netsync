CC = gcc
CFLAGS = -Wall -std=c99 -I../include
LDFLAGS = -L../target/release -lrtp_midi_netsync

TESTS = test_ffi test_ffi_comprehensive test_ffi_performance test_ffi_stress
BIN_DIR = bin

.PHONY: all test clean rust-lib

all: rust-lib $(addprefix $(BIN_DIR)/,$(TESTS))

rust-lib:
	cd .. && cargo build --release

$(BIN_DIR)/%: %.c | $(BIN_DIR)
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

$(BIN_DIR):
	mkdir -p $(BIN_DIR)

test: all
	@for test in $(TESTS); do \
		printf "\n\n----------\n\n Running %s...\n\n----------\n\n" "$$test"; \
		$(BIN_DIR)/$$test; \
	done

clean:
	rm -rf $(BIN_DIR)

help:
	@echo "Targets:"
	@echo "  all   - Build all tests"
	@echo "  test  - Build and run all tests"
	@echo "  clean - Remove built files"
