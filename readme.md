# rtp\_midi\_netsync

Rust projcet for parsing and building RTP-MIDI payloads ([RFC 6295](https://www.rfc-editor.org/rfc/rfc6295.html)) to be used in VLC’s netsync system.


## Features

* RTP-MIDI header and payload handling
* MIDI + MTC (quarter/full-frame) event parsing/serialization
* Converts between raw payloads and high-level `Event` enum
* C FFI interface


## Build & Test

```bash
cargo build
cargo test
cargo bench
```

## FFI

C header: [`include/rtp_midi_netsync.h`](include/rtp_midi_netsync.h)
Exposes minimal API for VLC netsync integration.
