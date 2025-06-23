#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/rtp_midi_netsync.h"

int main()
{
    printf("RTP-MIDI Netsync FFI Example\n");
    printf("===============================================\n\n");

    // Example 1: Create and encode an MTC Full Frame event
    printf("1. Master Flow - Encoding MTC Full Frame\n");
    CMidiEvent mtc_event = create_mtc_full_event(1, 30, 45, 15);

    // Allocate buffer on stack (C manages memory)
    uint8_t buffer[get_max_payload_size()];
    size_t actual_size;

    int result = master_netsync_flow_ffi(&mtc_event, buffer, sizeof(buffer), &actual_size);

    if (result == ERROR_SUCCESS)
    {
        printf("   Success! Payload length: %zu bytes\n", actual_size);
        printf("   Payload: ");
        for (size_t i = 0; i < actual_size; i++)
        {
            printf("%02X ", buffer[i]);
        }
        printf("\n\n");

        // Example 2: Decode the same payload (slave flow)
        printf("2. Slave Flow - Decoding payload\n");
        CMidiEvent parsed_event;
        result = slave_netsync_flow_ffi(buffer, actual_size, &parsed_event);

        if (result == ERROR_SUCCESS)
        {
            if (parsed_event.event_type == MIDI_EVENT_MTC_FULL)
            {
                printf("   Decoded MTC Full Frame: %d:%02d:%02d.%02d\n",
                       parsed_event.data[0],  // hour
                       parsed_event.data[1],  // minute
                       parsed_event.data[2],  // second
                       parsed_event.data[3]); // frame
            }
        }
        else
        {
            printf("   Decode failed: %s\n", get_error_message(result));
        }
    }
    else
    {
        printf("   Encode failed: %s\n", get_error_message(result));
    }

    printf("\n");

    // Example 3: MMC commands
    printf("3. MMC Commands\n");

    // Test MMC Play
    CMidiEvent mmc_play = create_mmc_play_event();
    result = master_netsync_flow_ffi(&mmc_play, buffer, sizeof(buffer), &actual_size);

    if (result == ERROR_SUCCESS)
    {
        printf("   MMC Play encoded successfully (%zu bytes)\n", actual_size);

        CMidiEvent play_decoded;
        result = slave_netsync_flow_ffi(buffer, actual_size, &play_decoded);
        if (result == ERROR_SUCCESS && play_decoded.event_type == MIDI_EVENT_MMC_PLAY)
        {
            printf("   MMC Play decoded successfully\n");
        }
    }

    // Test MMC Locate
    CMidiEvent mmc_locate = create_mmc_locate_event(2, 15, 30, 10);
    result = master_netsync_flow_ffi(&mmc_locate, buffer, sizeof(buffer), &actual_size);

    if (result == ERROR_SUCCESS)
    {
        printf("   MMC Locate encoded successfully (%zu bytes)\n", actual_size);

        CMidiEvent locate_decoded;
        result = slave_netsync_flow_ffi(buffer, actual_size, &locate_decoded);
        if (result == ERROR_SUCCESS && locate_decoded.event_type == MIDI_EVENT_MMC_LOCATE)
        {
            printf("   MMC Locate decoded: %d:%02d:%02d.%02d\n",
                   locate_decoded.data[0],
                   locate_decoded.data[1],
                   locate_decoded.data[2],
                   locate_decoded.data[3]);
        }
    }

    printf("\nExample completed successfully!\n");
    return 0;
}
