#include <stdio.h>
#include "../include/rtp_midi_netsync.h"

int main() {
    unsigned char buf[1] = { 0xAF };
    FFIPayloadHeader hdr = FFI_parse_header(buf, 1);

    printf("Parsed: flags=0x%X len=0x%X\n", hdr.flags, hdr.len);

    unsigned char out[1] = { 0 };
    FFI_serialize_header(hdr, out);
    printf("Serialized: 0x%X\n", out[0]);

    return (out[0] == buf[0]) ? 0 : 1;
}
