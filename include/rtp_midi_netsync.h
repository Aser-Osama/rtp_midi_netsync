#ifndef RTP_MIDI_NETSYNC_H
#define RTP_MIDI_NETSYNC_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// MIDI event types
typedef enum {
    MIDI_EVENT_MTC_QUARTER = 0,
    MIDI_EVENT_MTC_FULL = 1,
    MIDI_EVENT_MMC_STOP = 2,
    MIDI_EVENT_MMC_PLAY = 3,
    MIDI_EVENT_MMC_LOCATE = 4
} CMidiEventType;

// MIDI event structure
typedef struct {
    CMidiEventType event_type;
    uint8_t data[8];    // Event-specific data, max data is 5 bytes + 3 padding
    uint8_t data_len;   // Number of valid bytes in data
} CMidiEvent;

// Error codes
typedef enum {
    ERROR_SUCCESS = 0,
    ERROR_INVALID_MASTER_EVENT = 1,
    ERROR_INVALID_SLAVE_EVENT = 2,
    ERROR_BUFFER_TOO_SMALL = 3,
    ERROR_NULL_POINTER = 4,
    ERROR_INVALID_EVENT_TYPE = 5
} CErrorCode;

// Core netsync functions (C provides buffers)
int master_netsync_flow_ffi(const CMidiEvent* event,
                           uint8_t* buffer,
                           size_t buffer_size,
                           size_t* actual_size);

int slave_netsync_flow_ffi(const uint8_t* buffer,
                         size_t buffer_len,
                         CMidiEvent* event);

// Utility functions
size_t get_max_payload_size(void);
const char* get_error_message(int error_code);

// Helper functions for creating MIDI events
CMidiEvent create_mtc_quarter_event(uint8_t msg_type, uint8_t value);
CMidiEvent create_mtc_full_event(uint8_t hour, uint8_t minute, uint8_t second, uint8_t frame);
CMidiEvent create_mmc_stop_event(void);
CMidiEvent create_mmc_play_event(void);
CMidiEvent create_mmc_locate_event(uint8_t hour, uint8_t minute, uint8_t second, uint8_t frame);

#ifdef __cplusplus
}
#endif

#endif // RTP_MIDI_NETSYNC_H
