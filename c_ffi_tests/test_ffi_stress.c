#define _POSIX_C_SOURCE 199309L
#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>
#include <assert.h>
#include <signal.h>
#include <unistd.h>
#include <limits.h>
#include <sys/time.h>
#include "../include/rtp_midi_netsync.h"

// Test result tracking
static int tests_run = 0;
static int tests_passed = 0;
static int tests_failed = 0;

// Macros for test framework
#define TEST_START(name)                            \
    do                                              \
    {                                               \
        printf("Running stress test: %s...", name); \
        tests_run++;                                \
    } while (0)

#define TEST_PASS()        \
    do                     \
    {                      \
        printf(" PASS\n"); \
        tests_passed++;    \
    } while (0)

#define TEST_FAIL(msg)              \
    do                              \
    {                               \
        printf(" FAIL: %s\n", msg); \
        tests_failed++;             \
        return 0;                   \
    } while (0)

#define EXPECT_ERROR(actual, expected, msg)                                                         \
    do                                                                                              \
    {                                                                                               \
        if ((actual) != (expected))                                                                 \
        {                                                                                           \
            printf(" FAIL: %s (expected error %d, got %d)\n", msg, (int)(expected), (int)(actual)); \
            tests_failed++;                                                                         \
            return 0;                                                                               \
        }                                                                                           \
    } while (0)

#define EXPECT_SUCCESS(actual, msg)                                                                                    \
    do                                                                                                                 \
    {                                                                                                                  \
        if ((actual) != ERROR_SUCCESS)                                                                                 \
        {                                                                                                              \
            printf(" FAIL: %s (expected success, got error %d: %s)\n", msg, (int)(actual), get_error_message(actual)); \
            tests_failed++;                                                                                            \
            return 0;                                                                                                  \
        }                                                                                                              \
    } while (0)

// Helper to create corrupted events
CMidiEvent create_corrupted_event(CMidiEventType type, uint8_t data_len, uint8_t *data)
{
    CMidiEvent event;
    event.event_type = type;
    event.data_len = data_len;
    memset(event.data, 0, sizeof(event.data));
    if (data && data_len <= sizeof(event.data))
    {
        memcpy(event.data, data, data_len);
    }
    return event;
}

// Test: Null pointer stress tests
int test_null_pointer_stress()
{
    TEST_START("null_pointer_stress");

    CMidiEvent event = create_mmc_play_event();
    uint8_t buffer[16];
    size_t actual_size;

    // Test all possible null pointer combinations for master flow
    EXPECT_ERROR(master_netsync_flow_ffi(NULL, NULL, 0, NULL), ERROR_NULL_POINTER, "All nulls");
    EXPECT_ERROR(master_netsync_flow_ffi(&event, NULL, 0, NULL), ERROR_NULL_POINTER, "Buffer and size null");
    EXPECT_ERROR(master_netsync_flow_ffi(NULL, buffer, sizeof(buffer), NULL), ERROR_NULL_POINTER, "Event and size null");
    EXPECT_ERROR(master_netsync_flow_ffi(NULL, NULL, sizeof(buffer), &actual_size), ERROR_NULL_POINTER, "Event and buffer null");

    // Test all possible null pointer combinations for slave flow
    EXPECT_ERROR(slave_netsync_flow_ffi(NULL, 0, NULL), ERROR_NULL_POINTER, "All nulls slave");
    EXPECT_ERROR(slave_netsync_flow_ffi(buffer, sizeof(buffer), NULL), ERROR_NULL_POINTER, "Event null slave");
    EXPECT_ERROR(slave_netsync_flow_ffi(NULL, sizeof(buffer), &event), ERROR_NULL_POINTER, "Buffer null slave");

    TEST_PASS();
    return 1;
}

// Test: Buffer size edge cases
int test_buffer_size_extremes()
{
    TEST_START("buffer_size_extremes");

    CMidiEvent event = create_mtc_full_event(1, 30, 45, 15);
    size_t actual_size;

    // Test zero-sized buffer
    uint8_t zero_buffer[1];
    EXPECT_ERROR(master_netsync_flow_ffi(&event, zero_buffer, 0, &actual_size),
                 ERROR_BUFFER_TOO_SMALL, "Zero-sized buffer");

    // Test 1-byte buffer
    EXPECT_ERROR(master_netsync_flow_ffi(&event, zero_buffer, 1, &actual_size),
                 ERROR_BUFFER_TOO_SMALL, "1-byte buffer");

    // Test with extremely large buffer size (but still reasonable)
    uint8_t *large_buffer = malloc(65536);
    if (large_buffer)
    {
        int result = master_netsync_flow_ffi(&event, large_buffer, 65536, &actual_size);
        EXPECT_SUCCESS(result, "Extremely large buffer should work");
        free(large_buffer);
    }

    // Test SIZE_MAX (this should not crash, but may fail reasonably)
    // Note: We can't actually allocate SIZE_MAX bytes, so this tests parameter validation
    int result = master_netsync_flow_ffi(&event, zero_buffer, SIZE_MAX, &actual_size);
    // This should either succeed (if buffer is actually large enough for the payload)
    // or fail with BUFFER_TOO_SMALL (if the payload is larger than our tiny buffer)
    if (result != ERROR_SUCCESS && result != ERROR_BUFFER_TOO_SMALL)
    {
        TEST_FAIL("SIZE_MAX buffer size should not cause undefined behavior");
    }

    TEST_PASS();
    return 1;
}

// Test: Invalid event type values
int test_invalid_event_types()
{
    TEST_START("invalid_event_types");

    uint8_t buffer[16];
    size_t actual_size;

    // Test with invalid enum values (outside valid range)
    CMidiEvent invalid_events[] = {
        {.event_type = (CMidiEventType)255, .data_len = 0, .data = {0}},
        {.event_type = (CMidiEventType)100, .data_len = 0, .data = {0}},
        {.event_type = (CMidiEventType)-1, .data_len = 0, .data = {0}},
        {.event_type = (CMidiEventType)42, .data_len = 0, .data = {0}},
    };

    for (size_t i = 0; i < sizeof(invalid_events) / sizeof(invalid_events[0]); i++)
    {
        int result = master_netsync_flow_ffi(&invalid_events[i], buffer, sizeof(buffer), &actual_size);
        // Should either be INVALID_EVENT_TYPE or INVALID_MASTER_EVENT
        if (result != ERROR_INVALID_EVENT_TYPE && result != ERROR_INVALID_MASTER_EVENT)
        {
            printf(" FAIL: Invalid event type %d should return appropriate error, got %d\n",
                   (int)invalid_events[i].event_type, result);
            tests_failed++;
            return 0;
        }
    }

    TEST_PASS();
    return 1;
}

// Test: Data length mismatches
int test_data_length_mismatches()
{
    TEST_START("data_length_mismatches");

    uint8_t buffer[16];
    size_t actual_size;
    uint8_t test_data[8] = {1, 2, 3, 4, 5, 6, 7, 8};

    // MTC Quarter Frame should have 2 bytes, test with wrong lengths
    CMidiEvent mtc_quarter_short = create_corrupted_event(MIDI_EVENT_MTC_QUARTER, 1, test_data);
    EXPECT_ERROR(master_netsync_flow_ffi(&mtc_quarter_short, buffer, sizeof(buffer), &actual_size),
                 ERROR_INVALID_EVENT_TYPE, "MTC Quarter with 1 byte");

    CMidiEvent mtc_quarter_long = create_corrupted_event(MIDI_EVENT_MTC_QUARTER, 8, test_data);
    // This might succeed (extra data ignored) or fail, both are acceptable
    int result = master_netsync_flow_ffi(&mtc_quarter_long, buffer, sizeof(buffer), &actual_size);

    // MTC Full Frame should have 4 bytes, test with wrong lengths
    CMidiEvent mtc_full_short = create_corrupted_event(MIDI_EVENT_MTC_FULL, 3, test_data);
    EXPECT_ERROR(master_netsync_flow_ffi(&mtc_full_short, buffer, sizeof(buffer), &actual_size),
                 ERROR_INVALID_EVENT_TYPE, "MTC Full with 3 bytes");

    CMidiEvent mtc_full_zero = create_corrupted_event(MIDI_EVENT_MTC_FULL, 0, NULL);
    EXPECT_ERROR(master_netsync_flow_ffi(&mtc_full_zero, buffer, sizeof(buffer), &actual_size),
                 ERROR_INVALID_EVENT_TYPE, "MTC Full with 0 bytes");

    // MMC Locate should have 4 bytes, test with wrong lengths
    CMidiEvent mmc_locate_short = create_corrupted_event(MIDI_EVENT_MMC_LOCATE, 2, test_data);
    EXPECT_ERROR(master_netsync_flow_ffi(&mmc_locate_short, buffer, sizeof(buffer), &actual_size),
                 ERROR_INVALID_EVENT_TYPE, "MMC Locate with 2 bytes");

    // Test maximum data_len value (255)
    CMidiEvent max_data_len = create_corrupted_event(MIDI_EVENT_MTC_QUARTER, 255, test_data);
    result = master_netsync_flow_ffi(&max_data_len, buffer, sizeof(buffer), &actual_size);
    // Should fail with invalid event type since 255 > 2 (required for MTC Quarter)
    EXPECT_ERROR(result, ERROR_INVALID_EVENT_TYPE, "MTC Quarter with data_len=255");

    TEST_PASS();
    return 1;
}

// Test: Extreme time values
int test_extreme_time_values()
{
    TEST_START("extreme_time_values");

    uint8_t buffer[16];
    size_t actual_size;

    // Test with maximum possible time values
    CMidiEvent max_time = create_mtc_full_event(255, 255, 255, 255);
    int result = master_netsync_flow_ffi(&max_time, buffer, sizeof(buffer), &actual_size);
    // This should succeed - the library should handle any uint8_t values
    EXPECT_SUCCESS(result, "Maximum time values should be handled");

    // Test roundtrip with extreme values
    CMidiEvent decoded;
    result = slave_netsync_flow_ffi(buffer, actual_size, &decoded);
    EXPECT_SUCCESS(result, "Extreme values should decode successfully");

    // Test MMC Locate with extreme values
    CMidiEvent extreme_locate = create_mmc_locate_event(255, 255, 255, 255);
    result = master_netsync_flow_ffi(&extreme_locate, buffer, sizeof(buffer), &actual_size);
    EXPECT_SUCCESS(result, "Extreme MMC Locate values should be handled");

    // Test MTC Quarter with maximum msg_type and value
    CMidiEvent extreme_quarter = create_mtc_quarter_event(255, 255);
    result = master_netsync_flow_ffi(&extreme_quarter, buffer, sizeof(buffer), &actual_size);
    EXPECT_SUCCESS(result, "Extreme MTC Quarter values should be handled");

    TEST_PASS();
    return 1;
}

// Test: Malformed network payloads
int test_malformed_payloads()
{
    TEST_START("malformed_payloads");

    CMidiEvent decoded_event;

    // Test completely empty payload
    uint8_t empty_payload[] = {};
    EXPECT_ERROR(slave_netsync_flow_ffi(empty_payload, 0, &decoded_event),
                 ERROR_INVALID_SLAVE_EVENT, "Empty payload");

    // Test single byte payloads
    uint8_t single_byte_payloads[][1] = {
        {0x00}, {0xFF}, {0x80}, {0x7F}, {0x01}, {0xFE}};

    for (size_t i = 0; i < sizeof(single_byte_payloads) / sizeof(single_byte_payloads[0]); i++)
    {
        int result = slave_netsync_flow_ffi(single_byte_payloads[i], 1, &decoded_event);
        EXPECT_ERROR(result, ERROR_INVALID_SLAVE_EVENT, "Single byte payload should fail");
    }

    // Test payload with invalid header
    uint8_t invalid_headers[] = {
        0x00, 0x01, // Length 0 or 1 (too short)
        0x0F, 0x80, // Length 15 but only 2 bytes total
        0xF0, 0x80, // Invalid high nibble
        0x80, 0x90, // High bit set (invalid MIDI)
    };

    for (size_t i = 0; i < sizeof(invalid_headers) / 2; i++)
    {
        int result = slave_netsync_flow_ffi(&invalid_headers[i * 2], 2, &decoded_event);
        EXPECT_ERROR(result, ERROR_INVALID_SLAVE_EVENT, "Invalid header should fail");
    }

    // Test payload with correct length field but insufficient data
    uint8_t short_payload[] = {0x08, 0xF0}; // Claims 8 bytes but only has 2
    EXPECT_ERROR(slave_netsync_flow_ffi(short_payload, sizeof(short_payload), &decoded_event),
                 ERROR_INVALID_SLAVE_EVENT, "Short payload should fail");

    // Test payload with random garbage
    uint8_t garbage[] = {0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE};
    EXPECT_ERROR(slave_netsync_flow_ffi(garbage, sizeof(garbage), &decoded_event),
                 ERROR_INVALID_SLAVE_EVENT, "Garbage payload should fail");

    TEST_PASS();
    return 1;
}

// Test: Memory boundary conditions
int test_memory_boundaries()
{
    TEST_START("memory_boundaries");

    // Test with buffer exactly at the boundary of being too small
    CMidiEvent event = create_mtc_full_event(1, 30, 45, 15);
    size_t actual_size;

    // First, find out what size we actually need
    uint8_t temp_buffer[64];
    int result = master_netsync_flow_ffi(&event, temp_buffer, sizeof(temp_buffer), &actual_size);
    EXPECT_SUCCESS(result, "Getting actual payload size");

    // Now test with buffer exactly that size (should succeed)
    uint8_t *exact_buffer = malloc(actual_size);
    if (exact_buffer)
    {
        size_t exact_actual_size;
        result = master_netsync_flow_ffi(&event, exact_buffer, actual_size, &exact_actual_size);
        EXPECT_SUCCESS(result, "Exact size buffer should work");

        // Test with buffer one byte too small (should fail)
        if (actual_size > 0)
        {
            result = master_netsync_flow_ffi(&event, exact_buffer, actual_size - 1, &exact_actual_size);
            EXPECT_ERROR(result, ERROR_BUFFER_TOO_SMALL, "One byte too small should fail");
        }

        free(exact_buffer);
    }

    // Test reading from buffer boundaries in slave flow
    // Use the payload we generated above
    CMidiEvent decoded;
    result = slave_netsync_flow_ffi(temp_buffer, actual_size, &decoded);
    EXPECT_SUCCESS(result, "Decoding exact size payload should work");

    // Test with payload one byte too short
    if (actual_size > 0)
    {
        result = slave_netsync_flow_ffi(temp_buffer, actual_size - 1, &decoded);
        EXPECT_ERROR(result, ERROR_INVALID_SLAVE_EVENT, "One byte short payload should fail");
    }

    TEST_PASS();
    return 1;
}

// Test: Concurrent access simulation
int test_concurrent_access_simulation()
{
    TEST_START("concurrent_access_simulation");

    // Simulate rapid-fire calls that might expose race conditions
    // Note: This is single-threaded but tests for any global state issues

    uint8_t buffer[16];
    size_t actual_size;
    CMidiEvent decoded;

    // Rapid encode/decode cycles with different events
    CMidiEvent events[] = {
        create_mtc_quarter_event(0, 1),
        create_mtc_quarter_event(7, 15),
        create_mtc_full_event(23, 59, 59, 29),
        create_mmc_play_event(),
        create_mmc_stop_event(),
        create_mmc_locate_event(0, 0, 0, 0),
        create_mmc_locate_event(255, 255, 255, 255),
    };

    // Rapid fire test - encode and decode many times
    for (int iteration = 0; iteration < 1000; iteration++)
    {
        for (size_t i = 0; i < sizeof(events) / sizeof(events[0]); i++)
        {
            // Encode
            int result = master_netsync_flow_ffi(&events[i], buffer, sizeof(buffer), &actual_size);
            if (result != ERROR_SUCCESS)
            {
                printf(" FAIL: Encode failed at iteration %d, event %zu\n", iteration, i);
                tests_failed++;
                return 0;
            }

            // Decode
            result = slave_netsync_flow_ffi(buffer, actual_size, &decoded);
            if (result != ERROR_SUCCESS)
            {
                printf(" FAIL: Decode failed at iteration %d, event %zu\n", iteration, i);
                tests_failed++;
                return 0;
            }

            // Verify type matches
            if (decoded.event_type != events[i].event_type)
            {
                printf(" FAIL: Event type mismatch at iteration %d, event %zu\n", iteration, i);
                tests_failed++;
                return 0;
            }
        }
    }

    TEST_PASS();
    return 1;
}

// Test: Stack overflow attempts
int test_stack_overflow_attempts()
{
    TEST_START("stack_overflow_attempts");

    // Test with extremely large data_len values
    // This tests if the implementation properly bounds-checks array access

    CMidiEvent dangerous_event = {
        .event_type = MIDI_EVENT_MTC_QUARTER,
        .data_len = 255, // Much larger than the 8-byte data array
        .data = {1, 2, 3, 4, 5, 6, 7, 8}};

    uint8_t buffer[16];
    size_t actual_size;

    // This should not crash or cause buffer overflow
    int result = master_netsync_flow_ffi(&dangerous_event, buffer, sizeof(buffer), &actual_size);
    // Should fail with invalid event type due to length mismatch
    EXPECT_ERROR(result, ERROR_INVALID_EVENT_TYPE, "Oversized data_len should be rejected");

    // Test with corrupted data array (fill with pattern that might cause issues)
    CMidiEvent pattern_event = create_mtc_full_event(0xAA, 0xBB, 0xCC, 0xDD);
    // Corrupt the remaining data with a pattern
    for (int i = 4; i < 8; i++)
    {
        pattern_event.data[i] = 0xEE;
    }

    result = master_netsync_flow_ffi(&pattern_event, buffer, sizeof(buffer), &actual_size);
    EXPECT_SUCCESS(result, "Patterned data should not cause issues");

    TEST_PASS();
    return 1;
}

// Test: Resource exhaustion simulation
int test_resource_exhaustion()
{
    TEST_START("resource_exhaustion");

    // Test many allocations to see if there are any hidden memory leaks
    // or resource issues (this should all be stack-based, so no leaks expected)

    uint8_t buffer[32];
    size_t actual_size;
    CMidiEvent decoded;

    const int large_iteration_count = 100000;

    // Test with a variety of events in a loop to stress test
    for (int i = 0; i < large_iteration_count; i++)
    {
        // Create different events based on iteration
        CMidiEvent event;
        switch (i % 6)
        {
        case 0:
            event = create_mtc_quarter_event(i % 8, i % 16);
            break;
        case 1:
            event = create_mtc_full_event(i % 24, i % 60, i % 60, i % 30);
            break;
        case 2:
            event = create_mmc_play_event();
            break;
        case 3:
            event = create_mmc_stop_event();
            break;
        case 4:
            event = create_mmc_locate_event(i % 24, i % 60, i % 60, i % 30);
            break;
        default:
            event = create_mtc_quarter_event((i * 7) % 8, (i * 11) % 16);
            break;
        }

        // Encode
        int result = master_netsync_flow_ffi(&event, buffer, sizeof(buffer), &actual_size);
        if (result != ERROR_SUCCESS)
        {
            printf(" FAIL: Encode failed at iteration %d (error: %s)\n",
                   i, get_error_message(result));
            tests_failed++;
            return 0;
        }

        // Decode
        result = slave_netsync_flow_ffi(buffer, actual_size, &decoded);
        if (result != ERROR_SUCCESS)
        {
            printf(" FAIL: Decode failed at iteration %d (error: %s)\n",
                   i, get_error_message(result));
            tests_failed++;
            return 0;
        }

        // Quick sanity check every 10000 iterations
        if (i % 10000 == 0 && i > 0)
        {
            if (decoded.event_type != event.event_type)
            {
                printf(" FAIL: Type mismatch at iteration %d\n", i);
                tests_failed++;
                return 0;
            }
        }
    }

    printf(" (completed %d iterations)", large_iteration_count);
    TEST_PASS();
    return 1;
}

// Test: Error message robustness
int test_error_message_robustness()
{
    TEST_START("error_message_robustness");

    // Test error message function with various invalid inputs

    // Test with extreme error code values
    int extreme_codes[] = {
        -1000, -1, 1000, 999999, -999999,
        INT32_MAX, INT32_MIN, 0x7FFFFFFF, (int)0x80000000};

    for (size_t i = 0; i < sizeof(extreme_codes) / sizeof(extreme_codes[0]); i++)
    {
        const char *msg = get_error_message(extreme_codes[i]);
        if (msg == NULL)
        {
            printf(" FAIL: get_error_message returned NULL for code %d\n", extreme_codes[i]);
            tests_failed++;
            return 0;
        }

        // Verify it's a valid C string (has null terminator within reasonable bounds)
        size_t len = strnlen(msg, 1000); // Arbitrary reasonable limit
        if (len >= 1000)
        {
            printf(" FAIL: Error message too long or not null-terminated for code %d\n", extreme_codes[i]);
            tests_failed++;
            return 0;
        }
    }

    // Test all valid error codes
    for (int code = ERROR_SUCCESS; code <= ERROR_INVALID_EVENT_TYPE; code++)
    {
        const char *msg = get_error_message(code);
        if (msg == NULL || strlen(msg) == 0)
        {
            printf(" FAIL: Invalid error message for valid code %d\n", code);
            tests_failed++;
            return 0;
        }
    }

    TEST_PASS();
    return 1;
}

// Test: Helper function parameter extremes
int test_helper_function_extremes()
{
    TEST_START("helper_function_extremes");

    // Test helper functions with extreme parameter values
    // These should all succeed since they just create data structures

    CMidiEvent event;

    // Test MTC Quarter with extreme values
    event = create_mtc_quarter_event(255, 255);
    if (event.event_type != MIDI_EVENT_MTC_QUARTER || event.data_len != 2)
    {
        TEST_FAIL("MTC Quarter helper with extreme values");
    }

    // Test MTC Full with extreme values
    event = create_mtc_full_event(255, 255, 255, 255);
    if (event.event_type != MIDI_EVENT_MTC_FULL || event.data_len != 4)
    {
        TEST_FAIL("MTC Full helper with extreme values");
    }

    // Test MMC Locate with extreme values
    event = create_mmc_locate_event(255, 255, 255, 255);
    if (event.event_type != MIDI_EVENT_MMC_LOCATE || event.data_len != 4)
    {
        TEST_FAIL("MMC Locate helper with extreme values");
    }

    // Test with zero values
    event = create_mtc_quarter_event(0, 0);
    if (event.event_type != MIDI_EVENT_MTC_QUARTER || event.data[0] != 0 || event.data[1] != 0)
    {
        TEST_FAIL("MTC Quarter helper with zero values");
    }

    event = create_mtc_full_event(0, 0, 0, 0);
    if (event.event_type != MIDI_EVENT_MTC_FULL ||
        event.data[0] != 0 || event.data[1] != 0 || event.data[2] != 0 || event.data[3] != 0)
    {
        TEST_FAIL("MTC Full helper with zero values");
    }

    event = create_mmc_locate_event(0, 0, 0, 0);
    if (event.event_type != MIDI_EVENT_MMC_LOCATE ||
        event.data[0] != 0 || event.data[1] != 0 || event.data[2] != 0 || event.data[3] != 0)
    {
        TEST_FAIL("MMC Locate helper with zero values");
    }

    TEST_PASS();
    return 1;
}

// Performance stress test - measure under extreme load
int test_performance_under_stress()
{
    TEST_START("performance_under_stress");

    // This test hammers the functions with maximum speed to see if performance degrades

    const int stress_iterations = 1000000; // 1 million operations
    uint8_t buffer[32];
    size_t actual_size;
    CMidiEvent decoded;
    struct timespec start, end;

    CMidiEvent stress_event = create_mtc_quarter_event(3, 7);

    // Time the stress test
    clock_gettime(CLOCK_MONOTONIC, &start);

    for (int i = 0; i < stress_iterations; i++)
    {
        // Encode
        int result = master_netsync_flow_ffi(&stress_event, buffer, sizeof(buffer), &actual_size);
        if (result != ERROR_SUCCESS)
        {
            printf(" FAIL: Stress encode failed at iteration %d\n", i);
            tests_failed++;
            return 0;
        }

        // Decode
        result = slave_netsync_flow_ffi(buffer, actual_size, &decoded);
        if (result != ERROR_SUCCESS)
        {
            printf(" FAIL: Stress decode failed at iteration %d\n", i);
            tests_failed++;
            return 0;
        }

        // Quick verification every 100k iterations
        if (i % 100000 == 0 && i > 0)
        {
            if (decoded.event_type != stress_event.event_type)
            {
                printf(" FAIL: Stress test data corruption at iteration %d\n", i);
                tests_failed++;
                return 0;
            }
        }
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    double elapsed = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;

    printf(" (%.0f ops/sec)", (stress_iterations * 2) / elapsed); // *2 because encode+decode
    TEST_PASS();
    return 1;
}

// Test: Integer overflow scenarios
int test_integer_overflow_scenarios()
{
    TEST_START("integer_overflow_scenarios");

    uint8_t buffer[32];
    size_t actual_size;

    // Test with maximum size_t values for buffer operations
    CMidiEvent event = create_mtc_quarter_event(1, 2);

    // Test with SIZE_MAX-1 (should handle gracefully)
    int result = master_netsync_flow_ffi(&event, buffer, SIZE_MAX - 1, &actual_size);
    if (result != ERROR_SUCCESS && result != ERROR_BUFFER_TOO_SMALL)
    {
        TEST_FAIL("SIZE_MAX-1 buffer size should not cause undefined behavior");
    }

    // Test with very large but reasonable buffer sizes
    size_t large_sizes[] = {
        1024 * 1024,      // 1MB
        1024 * 1024 * 16, // 16MB (if we had that much memory)
        UINT32_MAX,       // Maximum 32-test_poison_value_patterns(bit value
    };

    for (size_t i = 0; i < sizeof(large_sizes) / sizeof(large_sizes[0]); i++)
    {
        result = master_netsync_flow_ffi(&event, buffer, large_sizes[i], &actual_size);
        // Should succeed since our actual buffer is tiny but reported size is huge
        if (result != ERROR_SUCCESS)
        {
            printf(" FAIL: Large size %zu should not cause overflow issues\n", large_sizes[i]);
            tests_failed++;
            return 0;
        }
    }

    TEST_PASS();
    return 1;
}

// Test: Concurrent simulation with state corruption attempts
int test_concurrent_state_corruption()
{
    TEST_START("concurrent_state_corruption");

    // Simulate rapid context switching by interleaving operations
    uint8_t buffer1[32], buffer2[32], buffer3[32];
    size_t size1, size2, size3;
    CMidiEvent decoded1, decoded2, decoded3;

    CMidiEvent events[3] = {
        create_mtc_quarter_event(1, 5),
        create_mtc_full_event(12, 34, 56, 78),
        create_mmc_locate_event(23, 45, 12, 29)};

    // Interleave operations to test for hidden global state
    for (int iteration = 0; iteration < 1000; iteration++)
    {
        // Start all three encodes
        int r1 = master_netsync_flow_ffi(&events[0], buffer1, sizeof(buffer1), &size1);
        int r2 = master_netsync_flow_ffi(&events[1], buffer2, sizeof(buffer2), &size2);
        int r3 = master_netsync_flow_ffi(&events[2], buffer3, sizeof(buffer3), &size3);

        if (r1 != ERROR_SUCCESS || r2 != ERROR_SUCCESS || r3 != ERROR_SUCCESS)
        {
            printf(" FAIL: Concurrent encode simulation failed at iteration %d\n", iteration);
            tests_failed++;
            return 0;
        }

        // Interleave decodes in different order
        r3 = slave_netsync_flow_ffi(buffer3, size3, &decoded3);
        r1 = slave_netsync_flow_ffi(buffer1, size1, &decoded1);
        r2 = slave_netsync_flow_ffi(buffer2, size2, &decoded2);

        if (r1 != ERROR_SUCCESS || r2 != ERROR_SUCCESS || r3 != ERROR_SUCCESS)
        {
            printf(" FAIL: Concurrent decode simulation failed at iteration %d\n", iteration);
            tests_failed++;
            return 0;
        }

        // Verify no cross-contamination
        if (decoded1.event_type != events[0].event_type ||
            decoded2.event_type != events[1].event_type ||
            decoded3.event_type != events[2].event_type)
        {
            printf(" FAIL: State corruption detected at iteration %d\n", iteration);
            tests_failed++;
            return 0;
        }
    }

    TEST_PASS();
    return 1;
}

// Test: Payload fuzzing
int test_payload_fuzzing()
{
    TEST_START("payload_fuzzing");

    CMidiEvent decoded;
    srand(time(NULL));

    // Generate random payloads and ensure they don't crash
    for (int test_case = 0; test_case < 10000; test_case++)
    {
        size_t payload_size = rand() % 32 + 1; // 1-32 bytes
        uint8_t *random_payload = malloc(payload_size);

        if (!random_payload)
            continue;

        // Fill with random data
        for (size_t i = 0; i < payload_size; i++)
        {
            random_payload[i] = rand() % 256;
        }

        // This should either succeed or fail gracefully, never crash
        int result = slave_netsync_flow_ffi(random_payload, payload_size, &decoded);
        result = result;

        // We don't care about the result, just that it doesn't crash
        // Most random payloads should fail, which is correct behavior

        free(random_payload);

        // Print progress every 1000 tests
        if (test_case % 1000 == 0 && test_case > 0)
        {
            printf(".");
            fflush(stdout);
        }
    }

    TEST_PASS();
    return 1;
}

// Test: Performance consistency (not security)
int test_performance_consistency()
{
    TEST_START("performance_consistency");

    struct timespec start, end;
    double valid_time;
    uint8_t buffer[32];
    size_t actual_size;
    CMidiEvent decoded;

    // Time valid operations only
    CMidiEvent valid_event = create_mtc_quarter_event(3, 7);
    clock_gettime(CLOCK_MONOTONIC, &start);

    for (int i = 0; i < 100000; i++)
    {
        master_netsync_flow_ffi(&valid_event, buffer, sizeof(buffer), &actual_size);
        slave_netsync_flow_ffi(buffer, actual_size, &decoded);
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    valid_time = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;

    printf(" (%.3f ops/sec)", 100000.0 / valid_time);

    TEST_PASS();
    return 1;
}

// Test: Structure padding and size assumptions
int test_structure_assumptions()
{
    TEST_START("structure_assumptions");

    // Verify our assumptions about structure sizes and layout
    printf("\n    Structure sizes:");
    printf("\n      CMidiEvent: %zu bytes", sizeof(CMidiEvent));
    printf("\n      CMidiEventType: %zu bytes", sizeof(CMidiEventType));
    printf("\n      size_t: %zu bytes", sizeof(size_t));
    printf("\n      uint8_t[8]: %zu bytes", sizeof(uint8_t[8]));

    // Test that structure packing is as expected
    CMidiEvent test_event = create_mtc_full_event(1, 2, 3, 4);

    // Calculate expected size: enum (4 bytes) + array (8 bytes) + len (1 byte) + padding
    size_t expected_min_size = sizeof(CMidiEventType) + 8 + sizeof(uint8_t);

    if (sizeof(CMidiEvent) < expected_min_size)
    {
        printf(" FAIL: CMidiEvent size (%zu) smaller than expected minimum (%zu)\n",
               sizeof(CMidiEvent), expected_min_size);
        tests_failed++;
        return 0;
    }

    // Test that we can safely access all fields
    volatile CMidiEventType type = test_event.event_type;
    volatile uint8_t len = test_event.data_len;
    volatile uint8_t data0 = test_event.data[0];
    volatile uint8_t data7 = test_event.data[7];

    // Suppress unused variable warnings
    (void)type;
    (void)len;
    (void)data0;
    (void)data7;

    TEST_PASS();
    return 1;
}

// Test: Extreme memory boundaries (near address space limits)
int test_extreme_memory_boundaries()
{
    TEST_START("extreme_memory_boundaries");

    CMidiEvent event = create_mtc_quarter_event(1, 2);
    size_t actual_size;

    // Test with pointer arithmetic edge cases
    uint8_t stack_buffer[64];

    // Test with buffer at different stack positions
    for (int offset = 0; offset < 32; offset++)
    {
        int result = master_netsync_flow_ffi(&event, stack_buffer + offset, 64 - offset, &actual_size);
        if (result != ERROR_SUCCESS && result != ERROR_BUFFER_TOO_SMALL)
        {
            printf(" FAIL: Stack buffer offset %d caused unexpected error %d\n", offset, result);
            tests_failed++;
            return 0;
        }
    }

    // Test with very high memory addresses (if possible)
    void *high_mem = malloc(64);
    if (high_mem)
    {
        int result = master_netsync_flow_ffi(&event, (uint8_t *)high_mem, 64, &actual_size);
        if (result != ERROR_SUCCESS)
        {
            printf(" FAIL: High memory allocation failed\n");
            free(high_mem);
            tests_failed++;
            return 0;
        }
        free(high_mem);
    }

    TEST_PASS();
    return 1;
}

// Signal handler needs to be outside the function
static volatile sig_atomic_t signal_count = 0;

void alarm_handler(int sig)
{
    (void)sig;
    signal_count++;
}

// Test: Signal interruption during operations
int test_signal_interruption()
{
    TEST_START("signal_interruption");

    // Reset signal count
    signal_count = 0;

    signal(SIGALRM, alarm_handler);

    uint8_t buffer[32];
    size_t actual_size;
    CMidiEvent decoded;

    // Use setitimer for more frequent interruptions (every 10ms)
    struct itimerval timer;
    timer.it_value.tv_sec = 0;
    timer.it_value.tv_usec = 10000;  // 10ms
    timer.it_interval.tv_sec = 0;
    timer.it_interval.tv_usec = 10000;  // repeat every 10ms

    setitimer(ITIMER_REAL, &timer, NULL);

    int operations_completed = 0;
    for (int i = 0; i < 100000 && signal_count < 50; i++)
    {
        CMidiEvent event = create_mtc_quarter_event(i % 8, i % 16);

        int result = master_netsync_flow_ffi(&event, buffer, sizeof(buffer), &actual_size);
        if (result != ERROR_SUCCESS)
        {
            printf(" FAIL: Signal interruption caused encode failure\n");
            tests_failed++;
            goto cleanup;
        }

        result = slave_netsync_flow_ffi(buffer, actual_size, &decoded);
        if (result != ERROR_SUCCESS)
        {
            printf(" FAIL: Signal interruption caused decode failure\n");
            tests_failed++;
            goto cleanup;
        }

        // Verify data integrity
        if (decoded.event_type != event.event_type)
        {
            printf(" FAIL: Signal interruption caused data corruption\n");
            tests_failed++;
            goto cleanup;
        }

        operations_completed++;

        // Add a small delay to increase chance of interruption
        usleep(100);  // 0.1ms delay
    }

cleanup:
    // Stop the timer
    timer.it_value.tv_sec = 0;
    timer.it_value.tv_usec = 0;
    timer.it_interval.tv_sec = 0;
    timer.it_interval.tv_usec = 0;
    setitimer(ITIMER_REAL, &timer, NULL);

    signal(SIGALRM, SIG_DFL); // Reset signal handler

    printf(" (survived %d signals, completed %d operations)", (int)signal_count, operations_completed);

    if (signal_count == 0) {
        printf(" WARN: No signals were delivered during test");
    }

    TEST_PASS();
    return 1;
}


// Test: Cross-platform type size validation
int test_cross_platform_assumptions()
{
    TEST_START("cross_platform_assumptions");

    // Verify critical type sizes that affect wire protocol
    printf("\n    Platform assumptions:");
    printf("\n      sizeof(int): %zu", sizeof(int));
    printf("\n      sizeof(long): %zu", sizeof(long));
    printf("\n      sizeof(void*): %zu", sizeof(void *));
    printf("\n      sizeof(size_t): %zu", sizeof(size_t));
    printf("\n      sizeof(uint32_t): %zu", sizeof(uint32_t));
    printf("\n      sizeof(uint8_t): %zu", sizeof(uint8_t));

    // These sizes are critical for the protocol
    if (sizeof(uint8_t) != 1)
    {
        TEST_FAIL("uint8_t must be exactly 1 byte");
    }
    if (sizeof(uint32_t) != 4)
    {
        TEST_FAIL("uint32_t must be exactly 4 bytes");
    }

    // Test endianness detection
    uint32_t test_value = 0x12345678;
    uint8_t *bytes = (uint8_t *)&test_value;

    printf("\n      Endianness: ");
    if (bytes[0] == 0x12)
    {
        printf("Big-endian");
    }
    else if (bytes[0] == 0x78)
    {
        printf("Little-endian");
    }
    else
    {
        printf("Unknown");
    }

    // Test that our structures have consistent layout
    CMidiEvent test_event = create_mtc_full_event(0x12, 0x34, 0x56, 0x78);
    if (test_event.data[0] != 0x12 || test_event.data[1] != 0x34 ||
        test_event.data[2] != 0x56 || test_event.data[3] != 0x78)
    {
        TEST_FAIL("Structure data layout is inconsistent");
    }

    TEST_PASS();
    return 1;
}

// Test: Memory pressure simulation
int test_memory_pressure_simulation()
{
    TEST_START("memory_pressure_simulation");

    // Allocate many buffers to simulate memory pressure
    const size_t num_buffers = 1000;
    void *buffers[num_buffers];
    size_t allocated = 0;

    // Allocate memory to create pressure
    for (size_t i = 0; i < num_buffers; i++)
    {
        buffers[i] = malloc(1024 * 1024); // 1MB each
        if (buffers[i])
        {
            allocated++;
            // Fill with pattern to ensure it's actually allocated
            memset(buffers[i], (int)(i & 0xFF), 1024 * 1024);
        }
        else
        {
            break; // Out of memory
        }
    }

    printf(" (allocated %zu MB)", allocated);

    // Now test FFI operations under memory pressure
    uint8_t stack_buffer[32];
    size_t actual_size;
    CMidiEvent decoded;

    for (int i = 0; i < 1000; i++)
    {
        CMidiEvent event = create_mtc_quarter_event(i % 8, i % 16);

        int result = master_netsync_flow_ffi(&event, stack_buffer, sizeof(stack_buffer), &actual_size);
        if (result != ERROR_SUCCESS)
        {
            printf(" FAIL: Memory pressure caused encode failure\n");
            // Cleanup
            for (size_t j = 0; j < allocated; j++)
            {
                free(buffers[j]);
            }
            tests_failed++;
            return 0;
        }

        result = slave_netsync_flow_ffi(stack_buffer, actual_size, &decoded);
        if (result != ERROR_SUCCESS)
        {
            printf(" FAIL: Memory pressure caused decode failure\n");
            // Cleanup
            for (size_t j = 0; j < allocated; j++)
            {
                free(buffers[j]);
            }
            tests_failed++;
            return 0;
        }
    }

    // Cleanup
    for (size_t i = 0; i < allocated; i++)
    {
        free(buffers[i]);
    }

    TEST_PASS();
    return 1;
}

// Test: Bit-level corruption detection
int test_bit_level_corruption()
{
    TEST_START("bit_level_corruption");

    CMidiEvent original = create_mtc_full_event(12, 34, 56, 78);
    uint8_t buffer[32];
    size_t actual_size;

    // Encode the original
    int result = master_netsync_flow_ffi(&original, buffer, sizeof(buffer), &actual_size);
    if (result != ERROR_SUCCESS)
    {
        TEST_FAIL("Original encoding failed");
    }

    // Test single-bit corruption in each byte of the payload
    for (size_t byte_idx = 0; byte_idx < actual_size; byte_idx++)
    {
        for (int bit_idx = 0; bit_idx < 8; bit_idx++)
        {
            uint8_t corrupted_buffer[32];
            memcpy(corrupted_buffer, buffer, actual_size);

            // Flip one bit
            corrupted_buffer[byte_idx] ^= (1 << bit_idx);

            CMidiEvent decoded;
            result = slave_netsync_flow_ffi(corrupted_buffer, actual_size, &decoded);

            // Most single-bit corruptions should cause decode failures
            // (this tests the robustness of the protocol)
            if (result == ERROR_SUCCESS)
            {
                // If it succeeds, verify the data is actually different
                if (memcmp(&decoded, &original, sizeof(CMidiEvent)) == 0)
                {
                    printf(" WARNING: Bit corruption at byte %zu, bit %d was not detected\n",
                           byte_idx, bit_idx);
                }
            }
            // We don't fail the test here - some corruptions might be in non-critical areas
        }
    }

    TEST_PASS();
    return 1;
}

// Test: Protocol version resilience
int test_protocol_version_resilience()
{
    TEST_START("protocol_version_resilience");

    // Test how the system handles potential future protocol changes
    CMidiEvent event = create_mtc_quarter_event(3, 7);
    uint8_t buffer[32];
    size_t actual_size;

    // Encode normally
    int result = master_netsync_flow_ffi(&event, buffer, sizeof(buffer), &actual_size);
    if (result != ERROR_SUCCESS)
    {
        TEST_FAIL("Base encoding failed");
    }

    // Simulate various protocol modifications
    uint8_t modified_buffer[32];
    CMidiEvent decoded;

    // Test with extra trailing data (future extension)
    memcpy(modified_buffer, buffer, actual_size);
    modified_buffer[actual_size] = 0xFF; // Extra byte
    result = slave_netsync_flow_ffi(modified_buffer, actual_size + 1, &decoded);
    // This should fail gracefully, not crash

    // Test with header modifications
    for (int i = 0; i < (int)actual_size && i < 4; i++)
    {
        memcpy(modified_buffer, buffer, actual_size);
        modified_buffer[i] |= 0x80; // Set high bit (might indicate future version)
        result = slave_netsync_flow_ffi(modified_buffer, actual_size, &decoded);
        // Should fail gracefully
    }

    TEST_PASS();
    return 1;
}

// Test: Defensive programming validation
int test_defensive_programming()
{
    TEST_START("defensive_programming");

    // Test that the library handles obviously invalid scenarios gracefully
    uint8_t buffer[32];
    size_t actual_size;
    CMidiEvent decoded;

    // Test with absurd event configurations
    CMidiEvent absurd_events[4];

    // Initialize each event properly
    absurd_events[0].event_type = MIDI_EVENT_MTC_QUARTER;
    absurd_events[0].data_len = 255;
    for (int i = 0; i < 8; i++)
        absurd_events[0].data[i] = 0xFF;

    absurd_events[1].event_type = MIDI_EVENT_MTC_FULL;
    absurd_events[1].data_len = 0;
    for (int i = 0; i < 8; i++)
        absurd_events[1].data[i] = 0;

    absurd_events[2].event_type = 999999;
    absurd_events[2].data_len = 100;
    uint8_t pattern[] = {0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11};
    for (int i = 0; i < 8; i++)
        absurd_events[2].data[i] = pattern[i];

    absurd_events[3].event_type = UINT32_MAX;
    absurd_events[3].data_len = UINT8_MAX;
    for (int i = 0; i < 8; i++)
        absurd_events[3].data[i] = 0xFF;

    for (size_t i = 0; i < 4; i++)
    {
        int result = master_netsync_flow_ffi(&absurd_events[i], buffer, sizeof(buffer), &actual_size);
        // Should return appropriate error, not crash
        if (result != ERROR_INVALID_EVENT_TYPE && result != ERROR_INVALID_MASTER_EVENT)
        {
            printf(" WARNING: Absurd event %zu returned unexpected result %d\n", i, result);
        }
    }

    // Test with absurd payloads
    uint8_t absurd_payloads[][8] = {
        {0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF},
        {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
        {0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55},
    };

    for (size_t i = 0; i < sizeof(absurd_payloads) / sizeof(absurd_payloads[0]); i++)
    {
        int result = slave_netsync_flow_ffi(absurd_payloads[i], 8, &decoded);
        // Should fail gracefully
        if (result == ERROR_SUCCESS)
        {
            printf(" WARNING: Absurd payload %zu was accepted\n", i);
        }
    }

    TEST_PASS();
    return 1;
}

// Test: Resource cleanup verification
int test_resource_cleanup()
{
    TEST_START("resource_cleanup");

    // Verify no resources are leaked during error conditions
    uint8_t buffer[16];
    size_t actual_size;
    CMidiEvent decoded;

    // Rapid-fire operations with errors to test cleanup
    for (int i = 0; i < 10000; i++)
    {
        // Deliberately cause errors
        CMidiEvent invalid_event;
        invalid_event.event_type = 999;
        invalid_event.data_len = 255;
        for (int j = 0; j < 8; j++)
            invalid_event.data[j] = 0xFF;

        master_netsync_flow_ffi(&invalid_event, buffer, sizeof(buffer), &actual_size);

        uint8_t invalid_payload[] = {0xFF, 0xFF, 0xFF};
        slave_netsync_flow_ffi(invalid_payload, sizeof(invalid_payload), &decoded);

        // Mix in some valid operations
        if (i % 10 == 0)
        {
            CMidiEvent valid_event = create_mtc_quarter_event(i % 8, i % 16);
            master_netsync_flow_ffi(&valid_event, buffer, sizeof(buffer), &actual_size);
            slave_netsync_flow_ffi(buffer, actual_size, &decoded);
        }
    }

    // If we get here without issues, cleanup is probably working
    TEST_PASS();
    return 1;
}

// Performance regression test
int test_performance_regression()
{
    TEST_START("performance_regression");

    const int iterations = 100000;
    struct timespec start, end;
    uint8_t buffer[32];
    size_t actual_size;
    CMidiEvent decoded;

    CMidiEvent test_event = create_mtc_quarter_event(3, 7);

    clock_gettime(CLOCK_MONOTONIC, &start);

    for (int i = 0; i < iterations; i++)
    {
        master_netsync_flow_ffi(&test_event, buffer, sizeof(buffer), &actual_size);
        slave_netsync_flow_ffi(buffer, actual_size, &decoded);
    }

    clock_gettime(CLOCK_MONOTONIC, &end);

    double elapsed = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    double ops_per_sec = (iterations * 2) / elapsed; // *2 for encode+decode

    // Expect at least 1M operations per second
    if (ops_per_sec < 1000000)
    {
        printf(" WARNING: Performance may have regressed (%.0f ops/sec)", ops_per_sec);
    }

    printf(" (%.0f ops/sec)", ops_per_sec);
    TEST_PASS();
    return 1;
}

// Main test runner
int main()
{
    printf("RTP-MIDI Netsync FFI Stress Tests\n");

    // Run all stress tests
    test_null_pointer_stress();
    test_buffer_size_extremes();
    test_invalid_event_types();
    test_data_length_mismatches();
    test_extreme_time_values();
    test_malformed_payloads();
    test_memory_boundaries();
    test_concurrent_access_simulation();
    test_stack_overflow_attempts();
    test_resource_exhaustion();
    test_error_message_robustness();
    test_helper_function_extremes();
    test_performance_under_stress();
    test_integer_overflow_scenarios();
    test_concurrent_state_corruption();
    test_payload_fuzzing();
    test_performance_consistency();
    test_structure_assumptions();
    test_extreme_memory_boundaries();
    test_signal_interruption();
    test_cross_platform_assumptions();
    test_memory_pressure_simulation();
    test_bit_level_corruption();
    test_protocol_version_resilience();
    test_defensive_programming();
    test_resource_cleanup();
    test_performance_regression();

    // Print results
    printf("\n==================================\n");
    printf("Stress Test Results:\n");
    printf("  Tests run:    %d\n", tests_run);
    printf("  Tests passed: %d\n", tests_passed);
    printf("  Tests failed: %d\n", tests_failed);

    if (tests_failed == 0)
    {
        printf(" All edge-case/stress-test test cases passed.\n");
        return 0;
    }
    else
    {
        printf("\n Some stress tests failed! Review the issues above.\n");
        return 1;
    }
}
