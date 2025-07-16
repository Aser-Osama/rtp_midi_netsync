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
    VlcRtpmidiEvent mtc_event = vlc_rtpmidi_create_mtc_full_event(1, 30, 45, 15);

    // Allocate buffer on stack (C manages memory)
    uint8_t buffer[vlc_rtpmidi_get_max_payload_size()];
    size_t actual_size;

    int result = vlc_rtpmidi_master_netsync_flow_ffi(&mtc_event, buffer, sizeof(buffer), &actual_size);

    if (result == VLC_RTPMIDI_ERROR_SUCCESS)
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
        VlcRtpmidiEvent parsed_event;
        result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &parsed_event);

        if (result == VLC_RTPMIDI_ERROR_SUCCESS)
        {
            if (parsed_event.event_type == VLC_RTPMIDI_EVENT_MTC_FULL)
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
            printf("   Decode failed: %s\n", vlc_rtpmidi_get_error_message(result));
        }
    }
    else
    {
        printf("   Encode failed: %s\n", vlc_rtpmidi_get_error_message(result));
    }

    printf("\n");

    // Example 3: MMC commands
    printf("3. MMC Commands\n");

    // Test MMC Play
    VlcRtpmidiEvent mmc_play = vlc_rtpmidi_create_mmc_play_event();
    result = vlc_rtpmidi_master_netsync_flow_ffi(&mmc_play, buffer, sizeof(buffer), &actual_size);

    if (result == VLC_RTPMIDI_ERROR_SUCCESS)
    {
        printf("   MMC Play encoded successfully (%zu bytes)\n", actual_size);

        VlcRtpmidiEvent play_decoded;
        result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &play_decoded);
        if (result == VLC_RTPMIDI_ERROR_SUCCESS && play_decoded.event_type == VLC_RTPMIDI_EVENT_MMC_PLAY)
        {
            printf("   MMC Play decoded successfully\n");
        }
    }

    // Test MMC Locate
    VlcRtpmidiEvent mmc_locate = vlc_rtpmidi_create_mmc_locate_event(2, 15, 30, 10);
    result = vlc_rtpmidi_master_netsync_flow_ffi(&mmc_locate, buffer, sizeof(buffer), &actual_size);

    if (result == VLC_RTPMIDI_ERROR_SUCCESS)
    {
        printf("   MMC Locate encoded successfully (%zu bytes)\n", actual_size);

        VlcRtpmidiEvent locate_decoded;
        result = vlc_rtpmidi_slave_netsync_flow_ffi(buffer, actual_size, &locate_decoded);
        if (result == VLC_RTPMIDI_ERROR_SUCCESS && locate_decoded.event_type == VLC_RTPMIDI_EVENT_MMC_LOCATE)
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
