//! # MIDI payload parsing and serialization for RTP-MIDI netsync.
//!
//! This module handles parsing and building MIDI messages that are relevant for netsync, specifically:
//!
//! - **Quarter-Frame MTC (MIDI Time Code)**: `0xF1 nn` where `nn` encodes message type and value
//! - **Full-Frame MTC (SysEx)**: `F0 7F devID 01 01 hr mn sc fr F7` for complete timecode
//! - **MMC (MIDI Machine Control)**: `F0 7F devID 06 cmd F7` for transport control (Stop/Play)
//! - **Other MIDI messages**: Treated as raw data for pass-through
//!
//! # Message Format Assumptions
//!
//! - Messages are sent without delta-time (FLAGS=0 in RTP-MIDI)
//! - One message per packet (no multiple message handling required)
//! - Device ID is typically broadcast (0x7F) for universal real-time messages

use crate::error::ParseError;

/// Local result type alias for cleaner function signatures.
type Result<T> = core::result::Result<T, ParseError>;

// ============================================================================
// MIDI Protocol Constants
// ============================================================================

/// Start of System Exclusive (SysEx) message.
const SYSEX_START: u8 = 0xF0;

/// End of System Exclusive (SysEx) message.
const SYSEX_END: u8 = 0xF7;

/// System Common message start byte for Quarter-Frame MTC.
const SYSCOMMON_START: u8 = 0xF1;

/// Broadcast device ID used in Universal Real-Time SysEx messages.
const SYSEX_DEVICE_ID_BROADCAST: u8 = 0x7F;

/// Universal Real-Time SysEx ID (manufacturer ID for real-time messages).
const UNIVERSAL_REALTIME_ID: u8 = 0x7F;

/// Sub-ID for MIDI Machine Control (MMC) messages.
const MMC_SUB_ID: u8 = 0x06;

/// First sub-ID byte for Full-Frame MTC messages.
const MTC_FULL_FRAME_SUB_ID1: u8 = 0x01;

/// Second sub-ID byte for Full-Frame MTC messages.
const MTC_FULL_FRAME_SUB_ID2: u8 = 0x01;

/// MMC command byte for Stop transport control.
const MMC_STOP_CMD: u8 = 0x01;

/// MMC command byte for Play transport control.
const MMC_PLAY_CMD: u8 = 0x02;

/// Expected length of a complete Full-Frame MTC SysEx message.
const MTC_FULL_FRAME_LENGTH: usize = 10;

/// Expected length of a complete MMC SysEx message.
const MMC_LENGTH: usize = 6;

/// Expected length of a Quarter-Frame MTC message.
const MTC_QUARTER_FRAME_LENGTH: usize = 2;

// ============================================================================
// MIDI Event Types
// ============================================================================

/// # MIDI events relevant for netsync.
///
/// This enum represents the subset of MIDI messages that are commonly used
/// for synchronization between networked MIDI devices. Other MIDI messages
/// are preserved as raw byte data.
#[derive(Debug, Clone, PartialEq)]
pub enum MidiEvent {
    /// # Quarter-Frame MIDI Time Code message.
    ///
    /// Format: `0xF1 nn` where `nn` is `0nnn dddd`:
    /// - `0nnn` (bits 4-6): Message type (0-7)
    /// - `dddd` (bits 0-3): Data value (0-15)
    ///
    /// Quarter-frame messages are sent 8 times per frame to build up a complete
    /// timecode, with each message containing 4 bits of the total timecode data
    /// they are typically used to sync playback.
    MtcQuarter { msg_type: u8, value: u8 },

    /// # Full-Frame MIDI Time Code message.
    ///
    /// Format: `F0 7F devID 01 01 hr mn sc fr F7`
    ///
    /// Provides complete timecode information in a single message, typically
    /// sent when synchronization starts or when a jump in time occurs.
    MtcFull {
        hour: u8,
        minute: u8,
        second: u8,
        frame: u8,
    },

    /// # MIDI Machine Control (MMC) transport command.
    ///
    /// Format: `F0 7F devID 06 cmd F7` where:
    /// - `cmd = 0x01`: Stop
    /// - `cmd = 0x02`: Play
    ///
    /// Used to control transport state (play/stop) across networked devices.
    Mmc { stop: bool },

    /// # Any other MIDI message not specifically handled.
    ///
    /// Raw bytes are preserved to allow pass-through of other MIDI data
    /// such as note events, control changes, etc.
    Other(Vec<u8>),
}

// ============================================================================
// Parsing Functions
// ============================================================================

/// Parse exactly one MIDI message from a buffer.
///
/// This internal function handles the core parsing logic for different MIDI message types.
/// It assumes no delta-time is present (FLAGS=0 in RTP-MIDI context).
///
/// # Arguments
///
/// * `buf` - Buffer containing MIDI data
/// * `len` - Number of bytes to parse from the buffer
///
/// # Returns
///
/// Returns a tuple of `MidiEvent` on success.
///
/// # Errors
///
/// - `ParseError::EmptyBuffer` if the buffer is empty or length is 0
/// - `ParseError::BufferTooSmall` if `len` exceeds the buffer size
fn parse_midi(buf: &[u8], len: usize) -> Result<MidiEvent> {
    if len == 0 || buf.is_empty() {
        return Err(ParseError::EmptyBuffer);
    }
    if len > buf.len() {
        return Err(ParseError::BufferTooSmall {
            requested: len,
            available: buf.len(),
        });
    }

    let b0 = buf[0];

    // Quarter-Frame MTC (System Common)
    if b0 == SYSCOMMON_START {
        if len < MTC_QUARTER_FRAME_LENGTH {
            // Not enough data for complete quarter frame, treat as Other
            return Ok(MidiEvent::Other(buf[..len].to_vec()));
        }
        let data_byte = buf[1];
        // In some cases, there might be more than 2 bytes (padding after the command)
        // The data is 0nnn dddd, where nnn is the message type and dddd is the value
        return Ok(MidiEvent::MtcQuarter {
            msg_type: data_byte >> 4,
            value: data_byte & 0x0F,
        });
    }

    // SysEx-based messages
    if b0 == SYSEX_START {
        // Find SysEx terminator or use all available bytes
        let sysex_end_pos = buf.iter().position(|&b| b == SYSEX_END);

        let (cmd_slice, cmd_size) = match sysex_end_pos {
            Some(end_pos) => (&buf[..=end_pos], end_pos + 1),
            None => (&buf[..len], len),
        };

        // Full-Frame MTC: F0 7F devID 01 01 hr mn sc fr F7
        if cmd_slice.len() >= MTC_FULL_FRAME_LENGTH
            && cmd_slice[1] == UNIVERSAL_REALTIME_ID
            && cmd_slice[2] == SYSEX_DEVICE_ID_BROADCAST
            && cmd_slice[3] == MTC_FULL_FRAME_SUB_ID1
            && cmd_slice[4] == MTC_FULL_FRAME_SUB_ID2
            && cmd_slice[cmd_size - 1] == SYSEX_END
        {
            return Ok(MidiEvent::MtcFull {
                hour: cmd_slice[5],
                minute: cmd_slice[6],
                second: cmd_slice[7],
                frame: cmd_slice[8],
            });
        }

        // MMC Stop/Play: F0 7F devID 06 cmd F7
        if cmd_slice.len() >= MMC_LENGTH
            && cmd_slice[1] == UNIVERSAL_REALTIME_ID
            && cmd_slice[2] == SYSEX_DEVICE_ID_BROADCAST
            && cmd_slice[3] == MMC_SUB_ID
            && cmd_slice[cmd_size - 1] == SYSEX_END
        {
            return Ok(MidiEvent::Mmc {
                stop: cmd_slice[4] == MMC_STOP_CMD,
            });
        }

        // Fallback for other SysEx
        return Ok(MidiEvent::Other(cmd_slice.to_vec()));
    }

    // Fallback for any other MIDI message
    Ok(MidiEvent::Other(buf[..len].to_vec()))
}

/// Serialize a MIDI event into a byte buffer.
///
/// This internal function handles building the binary representation of MIDI events.
///
/// # Arguments
///
/// * `event` - The MIDI event to serialize
/// * `buf` - Buffer to append the serialized bytes to
fn build_midi(event: &MidiEvent, buf: &mut Vec<u8>) {
    match event {
        MidiEvent::MtcQuarter { msg_type, value } => {
            buf.push(SYSCOMMON_START);
            buf.push((msg_type << 4) | (value & 0x0F));
        }
        MidiEvent::MtcFull {
            hour,
            minute,
            second,
            frame,
        } => {
            buf.extend_from_slice(&[
                SYSEX_START,
                UNIVERSAL_REALTIME_ID,
                SYSEX_DEVICE_ID_BROADCAST,
                MTC_FULL_FRAME_SUB_ID1,
                MTC_FULL_FRAME_SUB_ID2,
                *hour,
                *minute,
                *second,
                *frame,
                SYSEX_END,
            ]);
        }
        MidiEvent::Mmc { stop } => {
            let cmd = if *stop { MMC_STOP_CMD } else { MMC_PLAY_CMD };
            buf.extend_from_slice(&[
                SYSEX_START,
                UNIVERSAL_REALTIME_ID,
                SYSEX_DEVICE_ID_BROADCAST,
                MMC_SUB_ID,
                cmd,
                SYSEX_END,
            ]);
        }
        MidiEvent::Other(bytes) => buf.extend_from_slice(bytes),
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Parse a single MIDI message from a buffer.
///
/// This function parses exactly one MIDI message from the provided buffer,
/// returning the parsed event and the number of bytes consumed. It's designed
/// for RTP-MIDI contexts where messages don't have delta-time prefixes.
///
/// # Arguments
///
/// * `buf` - Buffer containing MIDI message data
/// * `len` - Number of bytes to parse from the buffer (must not exceed `buf.len()`)
///
/// # Returns
///
/// Returns `Ok((event, bytes_consumed))` on success, where:
/// - `event` is the parsed MIDI event
/// - `bytes_consumed` is the number of bytes used from the buffer
///
/// # Errors
///
/// - `ParseError::BufferTooSmall` if `len` exceeds the actual buffer size
/// - `ParseError::EmptyBuffer` if the buffer is empty or `len` is 0
pub fn parse_midi_list(buf: &[u8], len: usize) -> Result<MidiEvent> {
    if len > buf.len() {
        return Err(ParseError::BufferTooSmall {
            requested: len,
            available: buf.len(),
        });
    }
    parse_midi(buf, len)
}

/// Build a complete MIDI message packet from an event.
///
/// Serializes a MIDI event into a byte vector suitable for transmission
/// over RTP-MIDI. No delta-time is included as per RTP-MIDI conventions
/// for single-message packets.
///
/// # Arguments
///
/// * `event` - The MIDI event to serialize
///
/// # Returns
///
/// * A `Vec<u8>` containing the complete MIDI message bytes.
pub fn build_midi_list(event: &MidiEvent) -> Vec<u8> {
    let mut buf = Vec::new();
    build_midi(event, &mut buf);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mtc_quarter_frame() {
        let buf = [0xF1, 0x23];
        let result = parse_midi_list(&buf, 2).unwrap();

        match result {
            MidiEvent::MtcQuarter { msg_type, value } => {
                assert_eq!(msg_type, 2);
                assert_eq!(value, 3);
            }
            _ => panic!("Expected MtcQuarter event"),
        }
    }

    #[test]
    fn test_buffer_length_validation() {
        let buf = [0xF1, 0x23];
        // Requesting more bytes than available should fail
        let result = parse_midi_list(&buf, 10);
        assert_eq!(
            result,
            Err(ParseError::BufferTooSmall {
                requested: (10),
                available: (2)
            })
        );
    }

    #[test]
    fn test_roundtrip_all_types() {
        let events = vec![
            MidiEvent::MtcQuarter {
                msg_type: 3,
                value: 7,
            },
            MidiEvent::MtcFull {
                hour: 1,
                minute: 30,
                second: 45,
                frame: 12,
            },
            MidiEvent::Mmc { stop: true },
            MidiEvent::Mmc { stop: false },
            MidiEvent::Other(vec![0x90, 0x60, 0x7F]),
        ];

        for event in events {
            let built = build_midi_list(&event);
            let parsed = parse_midi_list(&built, built.len()).unwrap();
            assert_eq!(event, parsed);
        }
    }
}
