#ifndef RTP_MIDI_NETSYNC_H
#define RTP_MIDI_NETSYNC_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    unsigned char flags;
    unsigned char len;
} FFIPayloadHeader;

FFIPayloadHeader FFI_parse_header(const unsigned char* buf, int buf_len);
void FFI_serialize_header(FFIPayloadHeader hdr, unsigned char* out);

#ifdef __cplusplus
}
#endif
#endif // RTP_MIDI_NETSYNC_H
