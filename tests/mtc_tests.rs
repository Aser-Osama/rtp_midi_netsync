#[cfg(test)]
mod mtc_tests {
    use rtp_midi_netsync::error::MtcError;
    use rtp_midi_netsync::mtc::{
        quarter_frames_to_smpte, smpte_to_quarter_frames, smpte_to_us, us_to_smpte, MtcFullFrame,
        MtcQuarterFrame,
    };

    // === Helper Functions ===

    /// Create a valid MTC full frame for testing
    fn create_test_smpte(hours: u8, minutes: u8, seconds: u8, frames: u8) -> MtcFullFrame {
        MtcFullFrame {
            hours,
            minutes,
            seconds,
            frames,
        }
    }

    /// Create a valid MTC quarter frame for testing
    fn create_test_quarter_frame(frame_type: u8, value: u8) -> MtcQuarterFrame {
        MtcQuarterFrame { frame_type, value }
    }

    // === Basic Conversion Tests ===

    #[test]
    fn test_us_to_smpte_zero() {
        let result = us_to_smpte(0);
        assert_eq!(
            result,
            MtcFullFrame {
                hours: 0,
                minutes: 0,
                seconds: 0,
                frames: 0
            }
        );
    }

    #[test]
    fn test_us_to_smpte_one_second() {
        // 1 second = 1,000,000 microseconds = 30 frames at 30fps
        let result = us_to_smpte(1_000_000);
        assert_eq!(
            result,
            MtcFullFrame {
                hours: 0,
                minutes: 0,
                seconds: 1,
                frames: 0
            }
        );
    }

    #[test]
    fn test_us_to_smpte_one_minute() {
        // 1 minute = 60,000,000 microseconds
        let result = us_to_smpte(60_000_000);
        assert_eq!(
            result,
            MtcFullFrame {
                hours: 0,
                minutes: 1,
                seconds: 0,
                frames: 0
            }
        );
    }

    #[test]
    fn test_us_to_smpte_one_hour() {
        // 1 hour = 3,600,000,000 microseconds
        let result = us_to_smpte(3_600_000_000);
        assert_eq!(
            result,
            MtcFullFrame {
                hours: 1,
                minutes: 0,
                seconds: 0,
                frames: 0
            }
        );
    }

    #[test]
    fn test_us_to_smpte_single_frame() {
        // 1 frame at 30fps = 1/30 second = 33,333.33... microseconds
        let frame_duration = 1_000_000 / 30; // 33,333 microseconds
        let result = us_to_smpte(frame_duration);
        assert_eq!(
            result,
            MtcFullFrame {
                hours: 0,
                minutes: 0,
                seconds: 0,
                frames: 1
            }
        );
    }

    #[test]
    fn test_us_to_smpte_complex_time() {
        // Test: 2:30:45:15 (2 hours, 30 minutes, 45 seconds, 15 frames)
        let microseconds = (2 * 3600 + 30 * 60 + 45) * 1_000_000 + (15 * 1_000_000 / 30);
        let result = us_to_smpte(microseconds);
        assert_eq!(
            result,
            MtcFullFrame {
                hours: 2,
                minutes: 30,
                seconds: 45,
                frames: 15
            }
        );
    }

    #[test]
    fn test_smpte_to_us_zero() {
        let smpte = create_test_smpte(0, 0, 0, 0);
        let result = smpte_to_us(&smpte);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_smpte_to_us_one_second() {
        let smpte = create_test_smpte(0, 0, 1, 0);
        let result = smpte_to_us(&smpte);
        assert_eq!(result, 1_000_000);
    }

    #[test]
    fn test_smpte_to_us_one_minute() {
        let smpte = create_test_smpte(0, 1, 0, 0);
        let result = smpte_to_us(&smpte);
        assert_eq!(result, 60_000_000);
    }

    #[test]
    fn test_smpte_to_us_one_hour() {
        let smpte = create_test_smpte(1, 0, 0, 0);
        let result = smpte_to_us(&smpte);
        assert_eq!(result, 3_600_000_000);
    }

    #[test]
    fn test_smpte_to_us_single_frame() {
        let smpte = create_test_smpte(0, 0, 0, 1);
        let result = smpte_to_us(&smpte);
        assert_eq!(result, 1_000_000 / 30); // 33,333 microseconds
    }

    #[test]
    fn test_smpte_to_us_complex_time() {
        let smpte = create_test_smpte(2, 30, 45, 15);
        let result = smpte_to_us(&smpte);
        let expected = (2 * 3600 + 30 * 60 + 45) * 1_000_000 + (15 * 1_000_000 / 30);
        assert_eq!(result, expected);
    }

    // === Round-trip Conversion Tests ===

    #[test]
    fn test_roundtrip_us_smpte_us() {
        let test_cases = vec![
            0,
            1_000_000,     // 1 second
            60_000_000,    // 1 minute
            3_600_000_000, // 1 hour
            33_333,        // ~1 frame
            9_095_500_000, // 2:31:35:15
        ];

        for original_us in test_cases {
            let smpte = us_to_smpte(original_us);
            let result_us = smpte_to_us(&smpte);

            // Allow small rounding differences due to floating point arithmetic
            let diff = if original_us > result_us {
                original_us - result_us
            } else {
                result_us - original_us
            };

            assert!(
                diff <= 33_333, // Within one frame duration
                "Roundtrip failed for {}: got {}, diff = {}",
                original_us,
                result_us,
                diff
            );
        }
    }

    #[test]
    fn test_roundtrip_smpte_us_smpte() {
        let test_cases = vec![
            create_test_smpte(0, 0, 0, 0),
            create_test_smpte(0, 0, 1, 0),
            create_test_smpte(0, 1, 0, 0),
            create_test_smpte(1, 0, 0, 0),
            create_test_smpte(0, 0, 0, 1),
            create_test_smpte(23, 59, 59, 29), // Maximum valid SMPTE time
            create_test_smpte(12, 30, 45, 15),
        ];

        for original_smpte in test_cases {
            let us = smpte_to_us(&original_smpte);
            let result_smpte = us_to_smpte(us);
            assert_eq!(original_smpte, result_smpte);
        }
    }

    // === Quarter Frame Conversion Tests ===

    #[test]
    fn test_smpte_to_quarter_frames_zero() {
        let smpte = create_test_smpte(0, 0, 0, 0);
        let frames = smpte_to_quarter_frames(&smpte);

        assert_eq!(frames[0], create_test_quarter_frame(0, 0)); // frames low
        assert_eq!(frames[1], create_test_quarter_frame(1, 0)); // frames high
        assert_eq!(frames[2], create_test_quarter_frame(2, 0)); // seconds low
        assert_eq!(frames[3], create_test_quarter_frame(3, 0)); // seconds high
        assert_eq!(frames[4], create_test_quarter_frame(4, 0)); // minutes low
        assert_eq!(frames[5], create_test_quarter_frame(5, 0)); // minutes high
        assert_eq!(frames[6], create_test_quarter_frame(6, 0)); // hours low
        assert_eq!(frames[7], create_test_quarter_frame(7, 0x06)); // hours high + frame rate (30fps = 0x03 << 1)
    }

    #[test]
    fn test_smpte_to_quarter_frames_complex() {
        // Test: 12:34:56:29 (max frames for 30fps)
        let smpte = create_test_smpte(12, 34, 56, 29);
        let frames = smpte_to_quarter_frames(&smpte);

        assert_eq!(frames[0], create_test_quarter_frame(0, 29 & 0x0F)); // frames low = 13 (0x0D)
        assert_eq!(frames[1], create_test_quarter_frame(1, (29 >> 4) & 0x01)); // frames high = 1
        assert_eq!(frames[2], create_test_quarter_frame(2, 56 & 0x0F)); // seconds low = 8
        assert_eq!(frames[3], create_test_quarter_frame(3, (56 >> 4) & 0x03)); // seconds high = 3
        assert_eq!(frames[4], create_test_quarter_frame(4, 34 & 0x0F)); // minutes low = 2
        assert_eq!(frames[5], create_test_quarter_frame(5, (34 >> 4) & 0x03)); // minutes high = 2
        assert_eq!(frames[6], create_test_quarter_frame(6, 12 & 0x0F)); // hours low = 12 (0x0C)
        assert_eq!(
            frames[7],
            create_test_quarter_frame(7, ((12 >> 4) & 0x01) | (0x03 << 1))
        ); // hours high + frame rate
    }

    #[test]
    fn test_smpte_to_quarter_frames_max_values() {
        // Test maximum valid SMPTE values: 23:59:59:29
        let smpte = create_test_smpte(23, 59, 59, 29);
        let frames = smpte_to_quarter_frames(&smpte);

        assert_eq!(frames[0], create_test_quarter_frame(0, 29 & 0x0F)); // 13
        assert_eq!(frames[1], create_test_quarter_frame(1, (29 >> 4) & 0x01)); // 1
        assert_eq!(frames[2], create_test_quarter_frame(2, 59 & 0x0F)); // 11
        assert_eq!(frames[3], create_test_quarter_frame(3, (59 >> 4) & 0x03)); // 3
        assert_eq!(frames[4], create_test_quarter_frame(4, 59 & 0x0F)); // 11
        assert_eq!(frames[5], create_test_quarter_frame(5, (59 >> 4) & 0x03)); // 3
        assert_eq!(frames[6], create_test_quarter_frame(6, 23 & 0x0F)); // 7
        assert_eq!(
            frames[7],
            create_test_quarter_frame(7, ((23 >> 4) & 0x01) | (0x03 << 1))
        ); // 1 + frame rate
    }

    #[test]
    fn test_quarter_frames_to_smpte_zero() {
        let frames = [
            create_test_quarter_frame(0, 0),
            create_test_quarter_frame(1, 0),
            create_test_quarter_frame(2, 0),
            create_test_quarter_frame(3, 0),
            create_test_quarter_frame(4, 0),
            create_test_quarter_frame(5, 0),
            create_test_quarter_frame(6, 0),
            create_test_quarter_frame(7, 0x06), // 30fps frame rate
        ];

        let result = quarter_frames_to_smpte(&frames).unwrap();
        assert_eq!(result, create_test_smpte(0, 0, 0, 0));
    }

    #[test]
    fn test_quarter_frames_to_smpte_complex() {
        // Reconstruct 12:34:56:29
        let frames = [
            create_test_quarter_frame(0, 13),   // 29 & 0x0F
            create_test_quarter_frame(1, 1),    // (29 >> 4) & 0x01
            create_test_quarter_frame(2, 8),    // 56 & 0x0F
            create_test_quarter_frame(3, 3),    // (56 >> 4) & 0x03
            create_test_quarter_frame(4, 2),    // 34 & 0x0F
            create_test_quarter_frame(5, 2),    // (34 >> 4) & 0x03
            create_test_quarter_frame(6, 12),   // 12 & 0x0F
            create_test_quarter_frame(7, 0x06), // ((12 >> 4) & 0x01) | (0x03 << 1)
        ];

        let result = quarter_frames_to_smpte(&frames).unwrap();
        assert_eq!(result, create_test_smpte(12, 34, 56, 29));
    }

    // === Round-trip Quarter Frame Tests ===

    #[test]
    fn test_roundtrip_smpte_quarter_frames_smpte() {
        let test_cases = vec![
            create_test_smpte(0, 0, 0, 0),
            create_test_smpte(1, 2, 3, 4),
            create_test_smpte(23, 59, 59, 29),
            create_test_smpte(12, 30, 45, 15),
        ];

        for original_smpte in test_cases {
            let quarter_frames = smpte_to_quarter_frames(&original_smpte);
            let result_smpte = quarter_frames_to_smpte(&quarter_frames).unwrap();
            assert_eq!(original_smpte, result_smpte);
        }
    }

    // === Error Condition Tests ===

    #[test]
    fn test_quarter_frames_to_smpte_invalid_frame_type() {
        // Frame type out of sequence
        let frames = [
            create_test_quarter_frame(0, 0),
            create_test_quarter_frame(2, 0), // Should be 1
            create_test_quarter_frame(2, 0),
            create_test_quarter_frame(3, 0),
            create_test_quarter_frame(4, 0),
            create_test_quarter_frame(5, 0),
            create_test_quarter_frame(6, 0),
            create_test_quarter_frame(7, 0x06),
        ];

        let result = quarter_frames_to_smpte(&frames);
        assert_eq!(result, Err(MtcError::InvalidFrameType));
    }

    #[test]
    fn test_quarter_frames_to_smpte_invalid_value_range() {
        // Value too large
        let frames = [
            create_test_quarter_frame(0, 16), // Max is 15
            create_test_quarter_frame(1, 0),
            create_test_quarter_frame(2, 0),
            create_test_quarter_frame(3, 0),
            create_test_quarter_frame(4, 0),
            create_test_quarter_frame(5, 0),
            create_test_quarter_frame(6, 0),
            create_test_quarter_frame(7, 0x06),
        ];

        let result = quarter_frames_to_smpte(&frames);
        assert_eq!(result, Err(MtcError::InvalidValue));
    }

    #[test]
    fn test_quarter_frames_to_smpte_invalid_frames_value() {
        // Frames value > 29 (invalid for 30fps)
        let frames = [
            create_test_quarter_frame(0, 14), // 30 & 0x0F = 14
            create_test_quarter_frame(1, 1),  // (30 >> 4) = 1, total frames = 30
            create_test_quarter_frame(2, 0),
            create_test_quarter_frame(3, 0),
            create_test_quarter_frame(4, 0),
            create_test_quarter_frame(5, 0),
            create_test_quarter_frame(6, 0),
            create_test_quarter_frame(7, 0x06),
        ];

        let result = quarter_frames_to_smpte(&frames);
        assert_eq!(result, Err(MtcError::InvalidValue));
    }

    #[test]
    fn test_quarter_frames_to_smpte_invalid_seconds() {
        // Seconds value > 59
        let frames = [
            create_test_quarter_frame(0, 0),
            create_test_quarter_frame(1, 0),
            create_test_quarter_frame(2, 12), // 60 & 0x0F = 12
            create_test_quarter_frame(3, 3),  // (60 >> 4) = 3, total seconds = 60
            create_test_quarter_frame(4, 0),
            create_test_quarter_frame(5, 0),
            create_test_quarter_frame(6, 0),
            create_test_quarter_frame(7, 0x06),
        ];

        let result = quarter_frames_to_smpte(&frames);
        assert_eq!(result, Err(MtcError::InvalidValue));
    }

    #[test]
    fn test_quarter_frames_to_smpte_invalid_minutes() {
        // Minutes value > 59
        let frames = [
            create_test_quarter_frame(0, 0),
            create_test_quarter_frame(1, 0),
            create_test_quarter_frame(2, 0),
            create_test_quarter_frame(3, 0),
            create_test_quarter_frame(4, 12), // 60 & 0x0F = 12
            create_test_quarter_frame(5, 3),  // (60 >> 4) = 3, total minutes = 60
            create_test_quarter_frame(6, 0),
            create_test_quarter_frame(7, 0x06),
        ];

        let result = quarter_frames_to_smpte(&frames);
        assert_eq!(result, Err(MtcError::InvalidValue));
    }

    #[test]
    fn test_quarter_frames_to_smpte_invalid_hours() {
        // Hours value > 23
        let frames = [
            create_test_quarter_frame(0, 0),
            create_test_quarter_frame(1, 0),
            create_test_quarter_frame(2, 0),
            create_test_quarter_frame(3, 0),
            create_test_quarter_frame(4, 0),
            create_test_quarter_frame(5, 0),
            create_test_quarter_frame(6, 8),    // 24 & 0x0F = 8
            create_test_quarter_frame(7, 0x07), // ((24 >> 4) & 0x01) | (0x03 << 1) = 1 | 6 = 7, total hours = 24
        ];

        let result = quarter_frames_to_smpte(&frames);
        assert_eq!(result, Err(MtcError::InvalidValue));
    }

    #[test]
    fn test_quarter_frames_to_smpte_unsupported_frame_rate() {
        // Frame rate other than 30fps non-drop (0x03)
        let frames = [
            create_test_quarter_frame(0, 0),
            create_test_quarter_frame(1, 0),
            create_test_quarter_frame(2, 0),
            create_test_quarter_frame(3, 0),
            create_test_quarter_frame(4, 0),
            create_test_quarter_frame(5, 0),
            create_test_quarter_frame(6, 0),
            create_test_quarter_frame(7, 0x04), // Frame rate = 0x02 (25fps), not supported
        ];

        let result = quarter_frames_to_smpte(&frames);
        assert_eq!(result, Err(MtcError::InvalidValue));
    }

    // === Edge Case Tests ===

    #[test]
    fn test_frame_rate_encoding_in_quarter_frame_7() {
        // Verify that frame rate is always encoded as 30fps non-drop (0x03 << 1 = 0x06)
        let test_cases = vec![
            create_test_smpte(0, 0, 0, 0),
            create_test_smpte(15, 30, 45, 20),
            create_test_smpte(23, 59, 59, 29),
        ];

        for smpte in test_cases {
            let frames = smpte_to_quarter_frames(&smpte);
            let frame_rate_bits = (frames[7].value >> 1) & 0x03;
            assert_eq!(
                frame_rate_bits, 0x03,
                "Frame rate should always be 30fps non-drop"
            );
        }
    }

    #[test]
    fn test_bit_masking_in_conversions() {
        // Test that bit masking works correctly for edge values
        let smpte = create_test_smpte(31, 63, 63, 31); // Values that would overflow without masking
        let frames = smpte_to_quarter_frames(&smpte);

        // The conversion should mask values appropriately
        // Hours: 31 & 0x0F = 15, (31 >> 4) & 0x01 = 1
        assert_eq!(frames[6].value, 15);
        assert_eq!(frames[7].value & 0x01, 1);

        // Minutes: 63 & 0x0F = 15, (63 >> 4) & 0x03 = 3
        assert_eq!(frames[4].value, 15);
        assert_eq!(frames[5].value, 3);

        // Seconds: 63 & 0x0F = 15, (63 >> 4) & 0x03 = 3
        assert_eq!(frames[2].value, 15);
        assert_eq!(frames[3].value, 3);

        // Frames: 31 & 0x0F = 15, (31 >> 4) & 0x01 = 1
        assert_eq!(frames[0].value, 15);
        assert_eq!(frames[1].value, 1);
    }

    #[test]
    fn test_microsecond_precision() {
        // Test precision near frame boundaries
        let frame_duration = 1_000_000 / 30; // 33,333 microseconds per frame

        // Test values that should clearly be frame 0
        let result1 = us_to_smpte(10_000); // 0.3 frames
        assert_eq!(result1.frames, 0);

        // Test 16,666 microseconds (should be just under 0.5 frames)
        let result2 = us_to_smpte(16_666);
        assert_eq!(result2.frames, 0); // 0.49998 frames rounds down

        // Test 16,667 microseconds (should be just over 0.5 frames)
        let result3 = us_to_smpte(16_667);
        assert_eq!(result3.frames, 1); // 0.50001 frames rounds up

        // Test exactly at one frame duration
        let result4 = us_to_smpte(frame_duration); // 33,333 µs
        assert_eq!(result4.frames, 1);

        // Test one and a half frames
        let result5 = us_to_smpte(frame_duration + 16_667); // ~1.5 frames
        assert_eq!(result5.frames, 2);

        // Test values that are clearly different frames
        let result6 = us_to_smpte(50_000); // 1.5 frames
        assert_eq!(result6.frames, 2);
    }

    #[test]
    fn test_large_timestamp() {
        // Test with a large timestamp (24 hours)
        let twenty_four_hours = 24 * 3600 * 1_000_000_u64;
        let result = us_to_smpte(twenty_four_hours);

        print!(
            "Res = {}{}{}{}",
            result.hours, result.minutes, result.seconds, result.frames
        );
        // Should wrap to 0 hours (24 hours = 0 hours in 24-hour format)
        assert_eq!(result.hours, 0);
        assert_eq!(result.minutes, 0);
        assert_eq!(result.seconds, 0);
        assert_eq!(result.frames, 0);
    }
}
