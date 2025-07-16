#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include "../include/rtp_midi_netsync.h"

// Test result tracking
static int tests_run = 0;
static int tests_passed = 0;

// Macros for test framework
#define TEST_START(name)                     \
    do                                       \
    {                                        \
        printf("Running test: %s...", name); \
        tests_run++;                         \
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
        return 0;                   \
    } while (0)

#define ASSERT_EQ(actual, expected, msg)                                                      \
    do                                                                                        \
    {                                                                                         \
        if ((actual) != (expected))                                                           \
        {                                                                                     \
            printf(" FAIL: %s (expected %d, got %d)\n", msg, (int)(expected), (int)(actual)); \
            return 0;                                                                         \
        }                                                                                     \
    } while (0)

#define ASSERT_TRUE(condition, msg)     \
    do                                  \
    {                                   \
        if (!(condition))               \
        {                               \
            printf(" FAIL: %s\n", msg); \
            return 0;                   \
        }                               \
    } while (0)

// Helper function to print buffer contents
void print_buffer(const uint8_t *buffer, size_t len)
{
    printf("Buffer (%zu bytes): ", len);
    for (size_t i = 0; i < len; i++)
    {
        printf("%02X ", buffer[i]);
    }
    printf("\n");
}

// Test: Basic buffer size function
int test_vlc_rtpmidi_get_max_payload_size()
{
    TEST_START("vlc_rtpmidi_get_max_payload_size");

    size_t max_size = vlc_rtpmidi_get_max_payload_size();
    ASSERT_TRUE(max_size >= 16, "Max payload size should be at least 16 bytes");
    ASSERT_TRUE(max_size <= 64, "Max payload size should be reasonable (<=64 bytes)");

    TEST_PASS();
    return 1;
}

// Test: Error message function
int test_vlc_rtpmidi_get_error_message()
{
    TEST_START("vlc_rtpmidi_get_error_message");

    const char *success_msg = vlc_rtpmidi_get_error_message(VLC_RTPMIDI_ERROR_SUCCESS);
    ASSERT_TRUE(success_msg != NULL, "Success message should not be NULL");
    ASSERT_TRUE(strlen(success_msg) > 0, "Success message should not be empty");

    const char *error_msg = vlc_rtpmidi_get_error_message(VLC_RTPMIDI_ERROR_NULL_POINTER);
    ASSERT_TRUE(error_msg != NULL, "Error message should not be NULL");
    ASSERT_TRUE(strlen(error_msg) > 0, "Error message should not be empty");

    const char *unknown_msg = vlc_rtpmidi_get_error_message(999);
    ASSERT_TRUE(unknown_msg != NULL, "Unknown error message should not be NULL");

    TEST_PASS();
    return 1;
}

// Test: Helper function for creating MTC Quarter events
int test_vlc_rtpmidi_create_mtc_quarter_event()
{
    TEST_START("vlc_rtpmidi_create_mtc_quarter_event");

    VlcRtpmidiEvent event = vlc_rtpmidi_create_mtc_quarter_event(3, 7);

    ASSERT_EQ(event.event_type, VLC_RTPMIDI_EVENT_MTC_QUARTER, "Event type should be MTC Quarter");
    ASSERT_EQ(event.data_len, 2, "Data length should be 2");
    ASSERT_EQ(event.data[0], 3, "Message type should be 3");
    ASSERT_EQ(event.data[1], 7, "Value should be 7");

    TEST_PASS();
    return 1;
}

// Test: Helper function for creating MTC Full events
int test_vlc_rtpmidi_create_mtc_full_event()
{
    TEST_START("vlc_rtpmidi_create_mtc_full_event");

    VlcRtpmidiEvent event = vlc_rtpmidi_create_mtc_full_event(1, 30, 45, 15);

    ASSERT_EQ(event.event_type, VLC_RTPMIDI_EVENT_MTC_FULL, "Event type should be MTC Full");
    ASSERT_EQ(event.data_len, 4, "Data length should be 4");
    ASSERT_EQ(event.data[0], 1, "Hour should be 1");
    ASSERT_EQ(event.data[1], 30, "Minute should be 30");
    ASSERT_EQ(event.data[2], 45, "Second should be 45");
    ASSERT_EQ(event.data[3], 15, "Frame should be 15");

    TEST_PASS();
    return 1;
}

// Test: Helper functions for MMC events
int test_create_mmc_events()
{
    TEST_START("create_mmc_events");

    // Test MMC Stop
    VlcRtpmidiEvent stop_event = vlc_rtpmidi_create_mmc_stop_event();
    ASSERT_EQ(stop_event.event_type, VLC_RTPMIDI_EVENT_MMC_STOP, "Stop event type");
    ASSERT_EQ(stop_event.data_len, 0, "Stop event data length should be 0");

    // Test MMC Play
    VlcRtpmidiEvent play_event = vlc_rtpmidi_create_mmc_play_event();
    ASSERT_EQ(play_event.event_type, VLC_RTPMIDI_EVENT_MMC_PLAY, "Play event type");
    ASSERT_EQ(play_event.data_len, 0, "Play event data length should be 0");

    // Test MMC Locate
    VlcRtpmidiEvent locate_event = vlc_rtpmidi_create_mmc_locate_event(2, 15, 30, 10);
    ASSERT_EQ(locate_event.event_type, VLC_RTPMIDI_EVENT_MMC_LOCATE, "Locate event type");
    ASSERT_EQ(locate_event.data_len, 4, "Locate event data length should be 4");
    ASSERT_EQ(locate_event.data[0], 2, "Locate hour should be 2");
    ASSERT_EQ(locate_event.data[1], 15, "Locate minute should be 15");
    ASSERT_EQ(locate_event.data[2], 30, "Locate second should be 30");
    ASSERT_EQ(locate_event.data[3], 10, "Locate frame should be 10");

    TEST_PASS();
    return 1;
}

// Test: Master flow with null pointers
int test_master_flow_null_pointers()
{
    TEST_START("master_flow_null_pointers");

    VlcRtpmidiEvent event = vlc_rtpmidi_create_mmc_play_event();
    uint8_t buffer[16];
    size_t actual_size;

    // Test null event pointer
    int result = vlc_rtpmidi_master_netsync_flow_ffi(NULL, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_NULL_POINTER, "Should return NULL_POINTER error for null event");

    // Test null buffer pointer
    result = vlc_rtpmidi_master_netsync_flow_ffi(&event, NULL, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_NULL_POINTER, "Should return NULL_POINTER error for null buffer");

    // Test null actual_size pointer
    result = vlc_rtpmidi_master_netsync_flow_ffi(&event, buffer, sizeof(buffer), NULL);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_NULL_POINTER, "Should return NULL_POINTER error for null actual_size");

    TEST_PASS();
    return 1;
}

// Test: Slave flow with null pointers
int test_slave_flow_null_pointers()
{
    TEST_START("slave_flow_null_pointers");

    uint8_t buffer[16] = {0};
    VlcRtpmidiEvent event;

    // Test null buffer pointer
    int result = vlc_rtpmidi_slave_netsync_flow_ffi(NULL, sizeof(buffer), &event);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_NULL_POINTER, "Should return NULL_POINTER error for null buffer");

    // Test null event pointer
    result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, sizeof(buffer), NULL);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_NULL_POINTER, "Should return NULL_POINTER error for null event");

    TEST_PASS();
    return 1;
}

// Test: Master flow with buffer too small
int test_master_flow_buffer_too_small()
{
    TEST_START("master_flow_buffer_too_small");

    VlcRtpmidiEvent event = vlc_rtpmidi_create_mtc_full_event(1, 30, 45, 15);
    uint8_t small_buffer[1]; // Intentionally too small
    size_t actual_size;

    int result = vlc_rtpmidi_master_netsync_flow_ffi(&event, small_buffer, sizeof(small_buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_BUFFER_TOO_SMALL, "Should return BUFFER_TOO_SMALL error");
    ASSERT_EQ(actual_size, 0, "actual_size should be 0 on error");

    TEST_PASS();
    return 1;
}

// Test: Master flow with valid MTC Quarter event
int test_master_flow_mtc_quarter()
{
    TEST_START("master_flow_mtc_quarter");

    VlcRtpmidiEvent event = vlc_rtpmidi_create_mtc_quarter_event(0, 5);
    uint8_t buffer[vlc_rtpmidi_get_max_payload_size()];
    size_t actual_size;

    int result = vlc_rtpmidi_master_netsync_flow_ffi(&event, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "Should succeed");
    ASSERT_TRUE(actual_size > 0, "Should produce non-empty payload");
    ASSERT_TRUE(actual_size <= sizeof(buffer), "Payload size should fit in buffer");

    // Check that the payload starts with expected length (first 4 bits)
    ASSERT_TRUE((buffer[0] & 0x0F) > 0, "Payload length field should be non-zero");

    printf(" (payload size: %zu bytes)", actual_size);
    TEST_PASS();
    return 1;
}

// Test: Master flow with valid MTC Full event
int test_master_flow_mtc_full()
{
    TEST_START("master_flow_mtc_full");

    VlcRtpmidiEvent event = vlc_rtpmidi_create_mtc_full_event(1, 30, 45, 15);
    uint8_t buffer[vlc_rtpmidi_get_max_payload_size()];
    size_t actual_size;

    int result = vlc_rtpmidi_master_netsync_flow_ffi(&event, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "Should succeed");
    ASSERT_TRUE(actual_size > 0, "Should produce non-empty payload");

    printf(" (payload size: %zu bytes)", actual_size);
    TEST_PASS();
    return 1;
}

// Test: Master flow with MMC events
int test_master_flow_mmc_events()
{
    TEST_START("master_flow_mmc_events");

    uint8_t buffer[vlc_rtpmidi_get_max_payload_size()];
    size_t actual_size;
    int result;

    // Test MMC Play
    VlcRtpmidiEvent play_event = vlc_rtpmidi_create_mmc_play_event();
    result = vlc_rtpmidi_master_netsync_flow_ffi(&play_event, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Play should succeed");
    ASSERT_TRUE(actual_size > 0, "MMC Play should produce payload");

    // Test MMC Stop
    VlcRtpmidiEvent stop_event = vlc_rtpmidi_create_mmc_stop_event();
    result = vlc_rtpmidi_master_netsync_flow_ffi(&stop_event, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Stop should succeed");
    ASSERT_TRUE(actual_size > 0, "MMC Stop should produce payload");

    // Test MMC Locate
    VlcRtpmidiEvent locate_event = vlc_rtpmidi_create_mmc_locate_event(2, 15, 30, 10);
    result = vlc_rtpmidi_master_netsync_flow_ffi(&locate_event, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Locate should succeed");
    ASSERT_TRUE(actual_size > 0, "MMC Locate should produce payload");

    TEST_PASS();
    return 1;
}

// Test: Slave flow with empty buffer
int test_slave_flow_empty_buffer()
{
    TEST_START("slave_flow_empty_buffer");

    uint8_t empty_buffer[1] = {0}; // Buffer with minimal/invalid content
    VlcRtpmidiEvent event;

    int result = vlc_rtpmidi_slave_netsync_flow_ffi(empty_buffer, 0, &event); // Zero length
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_INVALID_SLAVE_EVENT, "Should return INVALID_SLAVE_EVENT for empty buffer");

    TEST_PASS();
    return 1;
}

// Test: Complete roundtrip for MTC Quarter
int test_roundtrip_mtc_quarter()
{
    TEST_START("roundtrip_mtc_quarter");

    // Original event
    VlcRtpmidiEvent original = vlc_rtpmidi_create_mtc_quarter_event(3, 7);

    // Encode
    uint8_t buffer[vlc_rtpmidi_get_max_payload_size()];
    size_t actual_size;
    int result = vlc_rtpmidi_master_netsync_flow_ffi(&original, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "Encoding should succeed");

    // Decode
    VlcRtpmidiEvent decoded;
    result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &decoded);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "Decoding should succeed");

    // Verify roundtrip
    ASSERT_EQ(decoded.event_type, original.event_type, "Event type should match");
    ASSERT_EQ(decoded.data_len, original.data_len, "Data length should match");
    ASSERT_EQ(decoded.data[0], original.data[0], "Message type should match");
    ASSERT_EQ(decoded.data[1], original.data[1], "Value should match");

    TEST_PASS();
    return 1;
}

// Test: Complete roundtrip for MTC Full
int test_roundtrip_mtc_full()
{
    TEST_START("roundtrip_mtc_full");

    // Original event
    VlcRtpmidiEvent original = vlc_rtpmidi_create_mtc_full_event(1, 30, 45, 15);

    // Encode
    uint8_t buffer[vlc_rtpmidi_get_max_payload_size()];
    size_t actual_size;
    int result = vlc_rtpmidi_master_netsync_flow_ffi(&original, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "Encoding should succeed");

    // Decode
    VlcRtpmidiEvent decoded;
    result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &decoded);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "Decoding should succeed");

    // Verify roundtrip
    ASSERT_EQ(decoded.event_type, original.event_type, "Event type should match");
    ASSERT_EQ(decoded.data_len, original.data_len, "Data length should match");
    ASSERT_EQ(decoded.data[0], original.data[0], "Hour should match");
    ASSERT_EQ(decoded.data[1], original.data[1], "Minute should match");
    ASSERT_EQ(decoded.data[2], original.data[2], "Second should match");
    ASSERT_EQ(decoded.data[3], original.data[3], "Frame should match");

    TEST_PASS();
    return 1;
}

// Test: Complete roundtrip for MMC commands
int test_roundtrip_mmc_commands()
{
    TEST_START("roundtrip_mmc_commands");

    uint8_t buffer[vlc_rtpmidi_get_max_payload_size()];
    size_t actual_size;
    int result;

    // Test MMC Play roundtrip
    VlcRtpmidiEvent original_play = vlc_rtpmidi_create_mmc_play_event();
    result = vlc_rtpmidi_master_netsync_flow_ffi(&original_play, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Play encoding should succeed");

    VlcRtpmidiEvent decoded_play;
    result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &decoded_play);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Play decoding should succeed");
    ASSERT_EQ(decoded_play.event_type, VLC_RTPMIDI_EVENT_MMC_PLAY, "MMC Play event type should match");

    // Test MMC Stop roundtrip
    VlcRtpmidiEvent original_stop = vlc_rtpmidi_create_mmc_stop_event();
    result = vlc_rtpmidi_master_netsync_flow_ffi(&original_stop, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Stop encoding should succeed");

    VlcRtpmidiEvent decoded_stop;
    result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &decoded_stop);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Stop decoding should succeed");
    ASSERT_EQ(decoded_stop.event_type, VLC_RTPMIDI_EVENT_MMC_STOP, "MMC Stop event type should match");

    // Test MMC Locate roundtrip
    VlcRtpmidiEvent original_locate = vlc_rtpmidi_create_mmc_locate_event(2, 15, 30, 10);
    result = vlc_rtpmidi_master_netsync_flow_ffi(&original_locate, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Locate encoding should succeed");

    VlcRtpmidiEvent decoded_locate;
    result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &decoded_locate);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Locate decoding should succeed");
    ASSERT_EQ(decoded_locate.event_type, VLC_RTPMIDI_EVENT_MMC_LOCATE, "MMC Locate event type should match");
    ASSERT_EQ(decoded_locate.data[0], 2, "MMC Locate hour should match");
    ASSERT_EQ(decoded_locate.data[1], 15, "MMC Locate minute should match");
    ASSERT_EQ(decoded_locate.data[2], 30, "MMC Locate second should match");
    ASSERT_EQ(decoded_locate.data[3], 10, "MMC Locate frame should match");

    TEST_PASS();
    return 1;
}

// Test: Invalid event data
int test_invalid_event_data()
{
    TEST_START("invalid_event_data");

    uint8_t buffer[vlc_rtpmidi_get_max_payload_size()];
    size_t actual_size;

    // Create an invalid MTC Quarter event (insufficient data)
    VlcRtpmidiEvent invalid_event = {
        .event_type = VLC_RTPMIDI_EVENT_MTC_QUARTER,
        .data = {0},
        .data_len = 1 // Should be 2 for MTC Quarter
    };

    int result = vlc_rtpmidi_master_netsync_flow_ffi(&invalid_event, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_INVALID_EVENT_TYPE, "Should return INVALID_EVENT_TYPE for malformed event");

    TEST_PASS();
    return 1;
}

// Test: Realistic usage scenario
int test_realistic_scenario()
{
    TEST_START("realistic_scenario");

    printf("\n    Simulating a realistic sync scenario:\n");

    // Scenario: Master sends multiple events to slave
    uint8_t buffer[vlc_rtpmidi_get_max_payload_size()];
    size_t actual_size;
    int result;

    // 1. Master sends MMC Locate to establish position
    printf("    1. Master: Sending MMC Locate (02:15:30.10)\n");
    VlcRtpmidiEvent locate_cmd = vlc_rtpmidi_create_mmc_locate_event(2, 15, 30, 10);
    result = vlc_rtpmidi_master_netsync_flow_ffi(&locate_cmd, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Locate should encode successfully");

    VlcRtpmidiEvent received_locate;
    result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &received_locate);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Locate should decode successfully");
    printf("    1. Slave: Received MMC Locate (%02d:%02d:%02d.%02d)\n",
           received_locate.data[0], received_locate.data[1],
           received_locate.data[2], received_locate.data[3]);

    // 2. Master sends MMC Play to start
    printf("    2. Master: Sending MMC Play\n");
    VlcRtpmidiEvent play_cmd = vlc_rtpmidi_create_mmc_play_event();
    result = vlc_rtpmidi_master_netsync_flow_ffi(&play_cmd, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Play should encode successfully");

    VlcRtpmidiEvent received_play;
    result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &received_play);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Play should decode successfully");
    printf("    2. Slave: Received MMC Play command\n");

    // 3. Master sends MTC Quarter Frame updates
    printf("    3. Master: Sending MTC Quarter Frame sequence\n");
    for (uint8_t msg_type = 0; msg_type < 8; msg_type++)
    {
        VlcRtpmidiEvent mtc_quarter = vlc_rtpmidi_create_mtc_quarter_event(msg_type, msg_type + 5);
        result = vlc_rtpmidi_master_netsync_flow_ffi(&mtc_quarter, buffer, sizeof(buffer), &actual_size);
        ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MTC Quarter Frame should encode successfully");

        VlcRtpmidiEvent received_quarter;
        result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &received_quarter);
        ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MTC Quarter Frame should decode successfully");
    }
    printf("    3. Slave: Received 8 MTC Quarter Frame messages\n");

    // 4. Master sends MMC Stop
    printf("    4. Master: Sending MMC Stop\n");
    VlcRtpmidiEvent stop_cmd = vlc_rtpmidi_create_mmc_stop_event();
    result = vlc_rtpmidi_master_netsync_flow_ffi(&stop_cmd, buffer, sizeof(buffer), &actual_size);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Stop should encode successfully");

    VlcRtpmidiEvent received_stop;
    result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &received_stop);
    ASSERT_EQ(result, VLC_RTPMIDI_ERROR_SUCCESS, "MMC Stop should decode successfully");
    printf("    4. Slave: Received MMC Stop command\n");

    printf("    Scenario completed successfully!\n");

    TEST_PASS();
    return 1;
}

// Main test runner
int main()
{
    printf("RTP-MIDI Netsync FFI Comprehensive Tests\n");
    printf("========================================\n\n");

    // Run all tests
    int success = 1;
    success &= test_vlc_rtpmidi_get_max_payload_size();
    success &= test_vlc_rtpmidi_get_error_message();
    success &= test_vlc_rtpmidi_create_mtc_quarter_event();
    success &= test_vlc_rtpmidi_create_mtc_full_event();
    success &= test_create_mmc_events();
    success &= test_master_flow_null_pointers();
    success &= test_slave_flow_null_pointers();
    success &= test_master_flow_buffer_too_small();
    success &= test_master_flow_mtc_quarter();
    success &= test_master_flow_mtc_full();
    success &= test_master_flow_mmc_events();
    success &= test_slave_flow_empty_buffer();
    success &= test_roundtrip_mtc_quarter();
    success &= test_roundtrip_mtc_full();
    success &= test_roundtrip_mmc_commands();
    success &= test_invalid_event_data();
    success &= test_realistic_scenario();

    // Print results
    printf("\n========================================\n");
    printf("Test Results:\n");
    printf("  Tests run:    %d\n", tests_run);
    printf("  Tests passed: %d\n", tests_passed);
    printf("  Tests failed: %d\n", tests_run - tests_passed);

    if (success && tests_passed == tests_run)
    {
        printf("\nAll tests PASSED!\n");
        return 0;
    }
    else
    {
        printf("\nSome tests FAILED!\n");
        return 1;
    }
}
