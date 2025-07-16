#ifndef RTP_MIDI_NETSYNC_H
#define RTP_MIDI_NETSYNC_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// MIDI event types
typedef enum {
    VLC_RTPMIDI_EVENT_MTC_QUARTER = 0,
    VLC_RTPMIDI_EVENT_MTC_FULL = 1,
    VLC_RTPMIDI_EVENT_MMC_STOP = 2,
    VLC_RTPMIDI_EVENT_MMC_PLAY = 3,
    VLC_RTPMIDI_EVENT_MMC_LOCATE = 4
} VlcRtpmidiEventType;

// MIDI event structure
typedef struct {
    VlcRtpmidiEventType event_type;
    uint8_t data[8];    // Event-specific data, max data is 5 bytes + 3 padding
    uint8_t data_len;   // Number of valid bytes in data
} VlcRtpmidiEvent;

// Error codes
typedef enum {
    VLC_RTPMIDI_ERROR_SUCCESS = 0,
    VLC_RTPMIDI_ERROR_INVALID_MASTER_EVENT = 1,
    VLC_RTPMIDI_ERROR_INVALID_SLAVE_EVENT = 2,
    VLC_RTPMIDI_ERROR_BUFFER_TOO_SMALL = 3,
    VLC_RTPMIDI_ERROR_NULL_POINTER = 4,
    VLC_RTPMIDI_ERROR_INVALID_EVENT_TYPE = 5
} VlcRtpmidiErrorCode;

// Core netsync functions (C provides buffers)
int vlc_rtpmidi_master_netsync_flow_ffi(const VlcRtpmidiEvent* event,
                                       uint8_t* buffer,
                                       size_t buffer_size,
                                       size_t* actual_size);

int vlc_rtpmidi_slave_netsync_flow_ffi(const uint8_t* buffer,
                                     size_t buffer_len,
                                     VlcRtpmidiEvent* event);

// Utility functions
size_t vlc_rtpmidi_get_max_payload_size(void);
const char* vlc_rtpmidi_get_error_message(int error_code);

// Helper functions for creating MIDI events
VlcRtpmidiEvent vlc_rtpmidi_create_mtc_quarter_event(uint8_t msg_type, uint8_t value);
VlcRtpmidiEvent vlc_rtpmidi_create_mtc_full_event(uint8_t hour, uint8_t minute, uint8_t second, uint8_t frame);
VlcRtpmidiEvent vlc_rtpmidi_create_mmc_stop_event(void);
VlcRtpmidiEvent vlc_rtpmidi_create_mmc_play_event(void);
VlcRtpmidiEvent vlc_rtpmidi_create_mmc_locate_event(uint8_t hour, uint8_t minute, uint8_t second, uint8_t frame);

#ifdef __cplusplus
}
#endif

#endif // RTP_MIDI_NETSYNC_H
