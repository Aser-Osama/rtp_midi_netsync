/*
 * RTP-MIDI Netsync FFI Performance Test Suite
 *
 * This file provides performance benchmarking of the RTP-MIDI Netsync FFI interface,
 * measuring encoding/decoding throughput and memory usage patterns.
 *
 * Copyright (c) 2024 VideoLAN
 * Licensed under the MIT License
 */

#define _POSIX_C_SOURCE 199309L // Enable POSIX functions like clock_gettime
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h> // For struct timespec and clock_gettime
#include "../include/rtp_midi_netsync.h"

/**
 * Calculate time difference between two timespec structures in seconds
 */
static double get_time_diff(struct timespec start, struct timespec end)
{
    return (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
}

/**
 * Performance test for encoding operations
 */
static void test_encoding_performance(void)
{
    printf("Encoding Performance Tests\n");
    printf("=========================\n");

    const int iterations = 100000;
    uint8_t buffer[get_max_payload_size()];
    size_t actual_size;
    struct timespec start, end;

    // Test MTC Quarter Frame encoding performance
    CMidiEvent mtc_quarter = create_mtc_quarter_event(3, 7);
    clock_gettime(CLOCK_MONOTONIC, &start);

    for (int i = 0; i < iterations; i++)
    {
        int result = master_netsync_flow_ffi(&mtc_quarter, buffer, sizeof(buffer), &actual_size);
        if (result != ERROR_SUCCESS)
        {
            printf("  ERROR: MTC Quarter encoding failed at iteration %d\n", i);
            return;
        }
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    double mtc_quarter_time = get_time_diff(start, end);

    // Test MTC Full Frame encoding performance
    CMidiEvent mtc_full = create_mtc_full_event(1, 30, 45, 15);
    clock_gettime(CLOCK_MONOTONIC, &start);

    for (int i = 0; i < iterations; i++)
    {
        int result = master_netsync_flow_ffi(&mtc_full, buffer, sizeof(buffer), &actual_size);
        if (result != ERROR_SUCCESS)
        {
            printf("  ERROR: MTC Full encoding failed at iteration %d\n", i);
            return;
        }
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    double mtc_full_time = get_time_diff(start, end);

    // Test MMC Locate encoding performance
    CMidiEvent mmc_locate = create_mmc_locate_event(2, 15, 30, 10);
    clock_gettime(CLOCK_MONOTONIC, &start);

    for (int i = 0; i < iterations; i++)
    {
        int result = master_netsync_flow_ffi(&mmc_locate, buffer, sizeof(buffer), &actual_size);
        if (result != ERROR_SUCCESS)
        {
            printf("  ERROR: MMC Locate encoding failed at iteration %d\n", i);
            return;
        }
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    double mmc_locate_time = get_time_diff(start, end);

    // Report results
    printf("  MTC Quarter Frame: %d operations in %.3f seconds (%.0f ops/sec)\n",
           iterations, mtc_quarter_time, iterations / mtc_quarter_time);
    printf("  MTC Full Frame:    %d operations in %.3f seconds (%.0f ops/sec)\n",
           iterations, mtc_full_time, iterations / mtc_full_time);
    printf("  MMC Locate:        %d operations in %.3f seconds (%.0f ops/sec)\n",
           iterations, mmc_locate_time, iterations / mmc_locate_time);
    printf("\n");
}

/**
 * Performance test for decoding operations
 */
static void test_decoding_performance(void)
{
    printf("Decoding Performance Tests\n");
    printf("=========================\n");

    const int iterations = 100000;
    uint8_t buffer[get_max_payload_size()];
    size_t actual_size;
    CMidiEvent decoded_event;
    struct timespec start, end;

    // Prepare test payloads
    CMidiEvent mtc_quarter = create_mtc_quarter_event(3, 7);
    int result = master_netsync_flow_ffi(&mtc_quarter, buffer, sizeof(buffer), &actual_size);
    if (result != ERROR_SUCCESS)
    {
        printf("  ERROR: Failed to prepare test payload\n");
        return;
    }
    size_t mtc_quarter_size = actual_size;

    clock_gettime(CLOCK_MONOTONIC, &start);

    for (int i = 0; i < iterations; i++)
    {
        result = slave_netsync_flow_ffi(buffer, mtc_quarter_size, &decoded_event);
        if (result != ERROR_SUCCESS)
        {
            printf("  ERROR: MTC Quarter decoding failed at iteration %d\n", i);
            return;
        }
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    double decode_time = get_time_diff(start, end);

    printf("  MTC Quarter Decode: %d operations in %.3f seconds (%.0f ops/sec)\n",
           iterations, decode_time, iterations / decode_time);
    printf("\n");
}

/**
 * Stress test with mixed encode/decode operations
 */
static void test_mixed_operations(void)
{
    printf("Mixed Operations Performance Test\n");
    printf("=================================\n");

    const int iterations = 50000;
    uint8_t buffer[get_max_payload_size()];
    size_t actual_size;
    CMidiEvent decoded_event;
    struct timespec start, end;

    // Create different event types for variety
    CMidiEvent events[] = {
        create_mtc_quarter_event(0, 5),
        create_mtc_quarter_event(1, 10),
        create_mtc_full_event(1, 30, 45, 15),
        create_mmc_play_event(),
        create_mmc_stop_event(),
        create_mmc_locate_event(2, 15, 30, 10)};
    int num_events = sizeof(events) / sizeof(events[0]);

    clock_gettime(CLOCK_MONOTONIC, &start);

    for (int i = 0; i < iterations; i++)
    {
        CMidiEvent *event = &events[i % num_events];

        // Encode
        int result = master_netsync_flow_ffi(event, buffer, sizeof(buffer), &actual_size);
        if (result != ERROR_SUCCESS)
        {
            printf("  ERROR: Encoding failed at iteration %d\n", i);
            return;
        }

        // Decode
        result = slave_netsync_flow_ffi(buffer, actual_size, &decoded_event);
        if (result != ERROR_SUCCESS)
        {
            printf("  ERROR: Decoding failed at iteration %d\n", i);
            return;
        }

        // Verify event type matches
        if (decoded_event.event_type != event->event_type)
        {
            printf("  ERROR: Event type mismatch at iteration %d\n", i);
            return;
        }
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    double mixed_time = get_time_diff(start, end);

    printf("  Mixed operations: %d cycles in %.3f seconds (%.0f cycles/sec)\n",
           iterations, mixed_time, iterations / mixed_time);
    printf("\n");
}

/**
 * Memory usage pattern analysis
 */
static void test_memory_usage(void)
{
    printf("Memory Usage Analysis\n");
    printf("====================\n");

    // Report structure sizes
    printf("  Event structure size: %zu bytes\n", sizeof(CMidiEvent));
    printf("  Maximum payload size: %zu bytes\n", get_max_payload_size());

    // Test with various buffer sizes
    uint8_t small_buffer[8];
    uint8_t medium_buffer[16];
    uint8_t large_buffer[64];
    size_t actual_size;

    CMidiEvent test_event = create_mtc_full_event(1, 30, 45, 15);

    // Test small buffer (should fail)
    int result = master_netsync_flow_ffi(&test_event, small_buffer, sizeof(small_buffer), &actual_size);
    printf("  Small buffer (8 bytes):   %s\n",
           result == ERROR_BUFFER_TOO_SMALL ? "Correctly rejected" : "Unexpectedly succeeded");

    // Test medium buffer (should succeed)
    result = master_netsync_flow_ffi(&test_event, medium_buffer, sizeof(medium_buffer), &actual_size);
    printf("  Medium buffer (16 bytes): %s (payload: %zu bytes)\n",
           result == ERROR_SUCCESS ? "Success" : "Failed", actual_size);

    // Test large buffer (should succeed)
    result = master_netsync_flow_ffi(&test_event, large_buffer, sizeof(large_buffer), &actual_size);
    printf("  Large buffer (64 bytes):  %s (payload: %zu bytes)\n",
           result == ERROR_SUCCESS ? "Success" : "Failed", actual_size);
    printf("\n");
}

int main(void)
{
    printf("RTP-MIDI Netsync FFI Performance Test Suite\n");
    printf("==========================================\n\n");

    test_encoding_performance();
    test_decoding_performance();
    test_mixed_operations();
    test_memory_usage();

    printf("Performance testing completed successfully.\n");
    return 0;
}
