use rtp_midi_netsync::error::ParseError;
use rtp_midi_netsync::midi::{build_midi_list, parse_midi_list, MidiEvent};

#[cfg(test)]
mod tests {
    use super::*;

    // === Basic Parsing Tests ===

    #[test]
    fn test_parse_mtc_quarter_frame() {
        // Parse standard MTC quarter-frame message (0xF1 0x23)
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
    fn test_parse_mtc_quarter_frame_insufficient_data() {
        // Incomplete quarter-frame should fall through to Other
        let buf = [0xF1];
        let result = parse_midi_list(&buf, 1);

        assert!(result.is_ok());
        match result.unwrap() {
            MidiEvent::Other(bytes) => assert_eq!(bytes, vec![0xF1]),
            _ => panic!("Expected Other event for insufficient MTC quarter frame data"),
        }
    }

    #[test]
    fn test_parse_mtc_quarter_frame_with_padding() {
        // Quarter-frame with extra padding bytes should still parse correctly (Case of "MidiOX")
        let buf = [0xF1, 0x23, 0x00, 0x00];
        let result = parse_midi_list(&buf, 4).unwrap();

        match result {
            MidiEvent::MtcQuarter { msg_type, value } => {
                assert_eq!(msg_type, 2);
                assert_eq!(value, 3);
            }
            _ => panic!("Expected MtcQuarter event"),
        }
    }

    #[test]
    fn test_parse_mtc_full_frame() {
        // Parse complete MTC full-frame SysEx message
        let buf = [0xF0, 0x7F, 0x7F, 0x01, 0x01, 0x01, 0x23, 0x45, 0x67, 0xF7];
        let result = parse_midi_list(&buf, 10).unwrap();

        match result {
            MidiEvent::MtcFull {
                hour,
                minute,
                second,
                frame,
            } => {
                assert_eq!(hour, 0x01);
                assert_eq!(minute, 0x23);
                assert_eq!(second, 0x45);
                assert_eq!(frame, 0x67);
            }
            _ => panic!("Expected MtcFull event"),
        }
    }

    #[test]
    fn test_parse_mtc_full_wrong_device_id() {
        // MTC full-frame with wrong device ID should fall through to Other
        let buf = [0xF0, 0x7F, 0x00, 0x01, 0x01, 0x01, 0x23, 0x45, 0x67, 0xF7];
        let result = parse_midi_list(&buf, 10).unwrap();

        match result {
            MidiEvent::Other(bytes) => assert_eq!(bytes.len(), 10),
            _ => panic!("Expected Other event for wrong device ID"),
        }
    }

    #[test]
    fn test_parse_mmc_stop() {
        // Parse MMC Stop command
        let buf = [0xF0, 0x7F, 0x7F, 0x06, 0x01, 0xF7];
        let result = parse_midi_list(&buf, 6).unwrap();

        match result {
            MidiEvent::Mmc { stop } => assert!(stop),
            _ => panic!("Expected Mmc Stop event"),
        }
    }

    #[test]
    fn test_parse_mmc_play() {
        // Parse MMC Play command
        let buf = [0xF0, 0x7F, 0x7F, 0x06, 0x02, 0xF7];
        let result = parse_midi_list(&buf, 6).unwrap();

        match result {
            MidiEvent::Mmc { stop } => assert!(!stop),
            _ => panic!("Expected Mmc Play event"),
        }
    }

    #[test]
    fn test_parse_mmc_invalid_command() {
        // MMC with unrecognized command should fall back to play (false)
        let buf = [0xF0, 0x7F, 0x7F, 0x06, 0x99, 0xF7];
        let result = parse_midi_list(&buf, 6).unwrap();

        match result {
            MidiEvent::Mmc { stop } => assert!(!stop), // Should default to play
            _ => panic!("Expected Mmc event with default play behavior"),
        }
    }

    #[test]
    fn test_parse_other_sysex() {
        // Non-standard SysEx should be treated as Other
        let buf = [0xF0, 0x43, 0x12, 0x34, 0xF7];
        let result = parse_midi_list(&buf, 5).unwrap();

        match result {
            MidiEvent::Other(bytes) => assert_eq!(bytes, vec![0xF0, 0x43, 0x12, 0x34, 0xF7]),
            _ => panic!("Expected Other event"),
        }
    }

    #[test]
    fn test_parse_sysex_without_end_byte() {
        // SysEx without terminator should still be parsed as Other
        let buf = [0xF0, 0x43, 0x12, 0x34];
        let result = parse_midi_list(&buf, 4).unwrap();

        match result {
            MidiEvent::Other(bytes) => assert_eq!(bytes, vec![0xF0, 0x43, 0x12, 0x34]),
            _ => panic!("Expected Other event"),
        }
    }

    #[test]
    fn test_parse_empty_sysex() {
        // Empty SysEx (just start and end) should be treated as Other
        let buf = [0xF0, 0xF7];
        let result = parse_midi_list(&buf, 2).unwrap();

        match result {
            MidiEvent::Other(bytes) => assert_eq!(bytes, vec![0xF0, 0xF7]),
            _ => panic!("Expected Other event"),
        }
    }

    #[test]
    fn test_parse_sysex_exact_length_but_wrong_content() {
        // SysEx with correct length but wrong sub-ID should fall through to Other
        let buf = [0xF0, 0x7F, 0x7F, 0x99, 0x01, 0xF7];
        let result = parse_midi_list(&buf, 6).unwrap();

        match result {
            MidiEvent::Other(bytes) => assert_eq!(bytes, vec![0xF0, 0x7F, 0x7F, 0x99, 0x01, 0xF7]),
            _ => panic!("Expected Other event"),
        }
    }

    #[test]
    fn test_parse_other_midi() {
        // Regular MIDI messages should be treated as Other
        let buf = [0x90, 0x60, 0x7F]; // Note on C4, velocity 127
        let result = parse_midi_list(&buf, 3).unwrap();

        match result {
            MidiEvent::Other(bytes) => assert_eq!(bytes, vec![0x90, 0x60, 0x7F]),
            _ => panic!("Expected Other event"),
        }
    }

    // === Error Condition Tests ===

    #[test]
    fn test_parse_empty_buffer() {
        // Empty buffer should return EmptyBuffer error
        let buf = [];
        let result = parse_midi_list(&buf, 0);

        assert_eq!(result, Err(ParseError::EmptyBuffer));
    }

    #[test]
    fn test_parse_buffer_too_small() {
        // Requesting more bytes than available should return BufferTooSmall error
        let buf = [0xF1, 0x23];
        let result = parse_midi_list(&buf, 10);

        assert_eq!(
            result,
            Err(ParseError::BufferTooSmall {
                requested: 10,
                available: 2
            })
        );
    }

    #[test]
    fn test_parse_with_exact_buffer_size() {
        // Requesting exactly the buffer size should work
        let buf = [0xF1, 0x23];
        let result = parse_midi_list(&buf, 2).unwrap();

        match result {
            MidiEvent::MtcQuarter { .. } => {}
            _ => panic!("Expected MtcQuarter event"),
        }
    }

    // === Building/Serialization Tests ===

    #[test]
    fn test_build_mtc_quarter_frame() {
        // Build standard MTC quarter-frame message
        let event = MidiEvent::MtcQuarter {
            msg_type: 2,
            value: 3,
        };
        let result = build_midi_list(&event);

        assert_eq!(result, vec![0xF1, 0x23]);
    }

    #[test]
    fn test_build_mtc_quarter_frame_overflow() {
        // Test bit masking behavior with overflow values
        let event = MidiEvent::MtcQuarter {
            msg_type: 0xFF,
            value: 0xFF,
        };
        let result = build_midi_list(&event);

        assert_eq!(result, vec![0xF1, 0xFF]); // High bits truncated, low bits preserved
    }

    #[test]
    fn test_mtc_quarter_frame_bit_masking() {
        // Verify explicit bit masking behavior in quarter-frame encoding
        let event = MidiEvent::MtcQuarter {
            msg_type: 0x8F, // Only lower 4 bits used after shift
            value: 0xAB,    // Only lower 4 bits used
        };
        let result = build_midi_list(&event);

        // msg_type: 0x8F << 4 = 0x8F0 (truncated to 0xF0 in u8)
        // value: 0xAB & 0x0F = 0x0B
        // result: 0xF0 | 0x0B = 0xFB
        assert_eq!(result, vec![0xF1, 0xFB]);
    }

    #[test]
    fn test_build_mtc_full_frame() {
        // Build complete MTC full-frame SysEx message
        let event = MidiEvent::MtcFull {
            hour: 0x01,
            minute: 0x23,
            second: 0x45,
            frame: 0x67,
        };
        let result = build_midi_list(&event);

        assert_eq!(
            result,
            vec![0xF0, 0x7F, 0x7F, 0x01, 0x01, 0x01, 0x23, 0x45, 0x67, 0xF7]
        );
    }

    #[test]
    fn test_build_mmc_stop() {
        // Build MMC Stop command
        let event = MidiEvent::Mmc { stop: true };
        let result = build_midi_list(&event);

        assert_eq!(result, vec![0xF0, 0x7F, 0x7F, 0x06, 0x01, 0xF7]);
    }

    #[test]
    fn test_build_mmc_play() {
        // Build MMC Play command
        let event = MidiEvent::Mmc { stop: false };
        let result = build_midi_list(&event);

        assert_eq!(result, vec![0xF0, 0x7F, 0x7F, 0x06, 0x02, 0xF7]);
    }

    #[test]
    fn test_build_other() {
        // Build Other message preserves raw bytes
        let bytes = vec![0x90, 0x60, 0x7F];
        let event = MidiEvent::Other(bytes.clone());
        let result = build_midi_list(&event);

        assert_eq!(result, bytes);
    }

    // === Roundtrip Tests ===

    #[test]
    fn test_roundtrip_mtc_quarter() {
        // Parse -> build -> compare for MTC quarter-frame
        let original = [0xF1, 0x23];
        let event = parse_midi_list(&original, 2).unwrap();
        let rebuilt = build_midi_list(&event);

        assert_eq!(original.to_vec(), rebuilt);
    }

    #[test]
    fn test_roundtrip_mtc_full() {
        // Parse -> build -> compare for MTC full-frame
        let original = [0xF0, 0x7F, 0x7F, 0x01, 0x01, 0x01, 0x23, 0x45, 0x67, 0xF7];
        let event = parse_midi_list(&original, 10).unwrap();
        let rebuilt = build_midi_list(&event);

        assert_eq!(original.to_vec(), rebuilt);
    }

    #[test]
    fn test_roundtrip_mmc() {
        // Parse -> build -> compare for MMC Stop and Play
        let original_stop = [0xF0, 0x7F, 0x7F, 0x06, 0x01, 0xF7];
        let event = parse_midi_list(&original_stop, 6).unwrap();
        let rebuilt = build_midi_list(&event);
        assert_eq!(original_stop.to_vec(), rebuilt);

        let original_play = [0xF0, 0x7F, 0x7F, 0x06, 0x02, 0xF7];
        let event = parse_midi_list(&original_play, 6).unwrap();
        let rebuilt = build_midi_list(&event);
        assert_eq!(original_play.to_vec(), rebuilt);
    }

    #[test]
    fn test_roundtrip_other() {
        // Parse -> build -> compare for Other messages
        let original = [0x90, 0x60, 0x7F];
        let event = parse_midi_list(&original, 3).unwrap();
        let rebuilt = build_midi_list(&event);

        assert_eq!(original.to_vec(), rebuilt);
    }

    // === Edge Case Tests ===

    #[test]
    fn test_partial_sysex_messages() {
        // Truncated SysEx messages should fall through to Other

        // Test MTC full frame with wrong length
        let buf = [0xF0, 0x7F, 0x7F, 0x01, 0x01, 0x01, 0x23, 0x45]; // Missing frame and F7
        let result = parse_midi_list(&buf, 8).unwrap();

        match result {
            MidiEvent::Other(bytes) => assert_eq!(bytes.len(), 8),
            _ => panic!("Expected Other event for partial MTC full frame"),
        }

        // Test MMC with wrong length
        let buf = [0xF0, 0x7F, 0x7F, 0x06, 0x01]; // Missing F7
        let result = parse_midi_list(&buf, 5).unwrap();

        match result {
            MidiEvent::Other(bytes) => assert_eq!(bytes.len(), 5),
            _ => panic!("Expected Other event for partial MMC"),
        }
    }

    #[test]
    fn test_mtc_quarter_edge_cases() {
        // Test maximum valid values for MTC quarter-frame
        let event = MidiEvent::MtcQuarter {
            msg_type: 7,
            value: 15,
        };
        let built = build_midi_list(&event);
        let parsed = parse_midi_list(&built, built.len()).unwrap();

        match parsed {
            MidiEvent::MtcQuarter { msg_type, value } => {
                assert_eq!(msg_type, 7);
                assert_eq!(value, 15);
            }
            _ => panic!("Expected MtcQuarter event"),
        }
    }

    #[test]
    fn test_sysex_wrong_universal_realtime_id() {
        // SysEx with wrong Universal Real-Time ID should fall through to Other
        let buf = [0xF0, 0x43, 0x7F, 0x01, 0x01, 0x01, 0x23, 0x45, 0x67, 0xF7]; // Wrong first ID
        let result = parse_midi_list(&buf, 10).unwrap();

        match result {
            MidiEvent::Other(bytes) => assert_eq!(bytes.len(), 10),
            _ => panic!("Expected Other event for wrong Universal Real-Time ID"),
        }
    }

    #[test]
    fn test_sysex_wrong_sub_ids() {
        // SysEx with correct structure but wrong sub-IDs should fall through to Other
        let buf = [0xF0, 0x7F, 0x7F, 0x99, 0x99, 0x01, 0x23, 0x45, 0x67, 0xF7]; // Wrong sub-IDs
        let result = parse_midi_list(&buf, 10).unwrap();

        match result {
            MidiEvent::Other(bytes) => assert_eq!(bytes.len(), 10),
            _ => panic!("Expected Other event for wrong sub-IDs"),
        }
    }
}
