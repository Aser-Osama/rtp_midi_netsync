//! # MIDI Time Code (MTC) to SMPTE Timecode Conversion
//!
//! This module provides conversion between MIDI Time Code (MTC) quarter frames and SMPTE timecode,
//! following the VLC media player's implementation for consistent timing behavior.
//!
//! ## SMPTE Timecode Format
//!
//! SMPTE timecode represents time as `HH:MM:SS:FF` where:
//! - `HH`: Hours (0-23)
//! - `MM`: Minutes (0-59)
//! - `SS`: Seconds (0-59)
//! - `FF`: Frames (0-29 for 30fps non-drop frame)
//!
//! ## MTC Quarter Frame
//!
//! MTC transmits timecode information as 8 quarter frames, each containing 4 bits of data:
//!
//! | Quarter Frame | Content | Value Range |
//! |---------------|---------|-------------|
//! | 0 | Frames low nibble | 0-15 |
//! | 1 | Frames high nibble | 0-1 |
//! | 2 | Seconds low nibble | 0-15 |
//! | 3 | Seconds high nibble | 0-3 |
//! | 4 | Minutes low nibble | 0-15 |
//! | 5 | Minutes high nibble | 0-3 |
//! | 6 | Hours low nibble | 0-15 |
//! | 7 | Hours high nibble + frame rate | 0-1 (+ rate bits) |
//!
//!
//! ## Limitations
//!
//! - Only supports 30fps non-drop frame format
//! - Drop frame timecode is not implemented
//! - Frame rates other than 30fps are not supported
use crate::error::MtcError;

/// SMPTE timecode: HH:MM:SS:FF (30fps non-drop frame)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MtcFullFrame {
    pub hours: u8,   // 0-23
    pub minutes: u8, // 0-59
    pub seconds: u8, // 0-59
    pub frames: u8,  // 0-29
}

/// MTC quarter frame: frame_type (0-7) + value (0-15)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MtcQuarterFrame {
    /// Frame type identifier (0-7), determines which part of timecode this frame contains
    pub frame_type: u8,
    /// 4-bit data value (0-15)
    pub value: u8,
}

/// Microseconds timestamp (VLC-compatible)
pub type VlcTickT = u64;

/// SMPTE frame rate: 30 frames per second (non-drop frame)
const SMPTE_30_FPS: u32 = 30;

/// SMPTE frame rate base (always 1 for non-drop frame)
const SMPTE_30_FPS_BASE: u32 = 1;

/// Microseconds per second conversion factor
const VLC_TICK_FROM_SEC: u64 = 1_000_000;

/// # Converts microseconds to SMPTE timecode (30fps non-drop frame).
///
///
/// Follows the following conversion from `vlc_player_SendSmpteTimerSourceUpdates()`
/// ```c
/// framenum = round(point->ts * frame_rate / (double) frame_rate_base / VLC_TICK_FROM_SEC(1));
/// tc.frames = framenum % (frame_rate / frame_rate_base);
/// tc.seconds = (framenum * frame_rate_base / frame_rate) % 60;
/// tc.minutes = (framenum * frame_rate_base / frame_rate / 60) % 60;
/// tc.hours = framenum * frame_rate_base / frame_rate / 3600;
/// ```
/// # Arguments
///
/// * `us` - Timestamp in microseconds since epoch
///
/// # Returns
///
/// SMPTE timecode structure with hours, minutes, seconds, and frames
pub fn us_to_smpte(us: VlcTickT) -> MtcFullFrame {
    let frame_rate = SMPTE_30_FPS;
    let frame_rate_base = SMPTE_30_FPS_BASE;

    let framenum =
        ((us * frame_rate as u64) as f64 / frame_rate_base as f64 / VLC_TICK_FROM_SEC as f64)
            .round() as u64;

    let frames = (framenum % (frame_rate / frame_rate_base) as u64) as u8;
    let seconds = ((framenum * frame_rate_base as u64 / frame_rate as u64) % 60) as u8;
    let minutes = ((framenum * frame_rate_base as u64 / frame_rate as u64 / 60) % 60) as u8;
    let hours = ((framenum * frame_rate_base as u64 / frame_rate as u64 / 3600) % 24) as u8; // Limit hours to 0-23 unlike VLC impl

    MtcFullFrame {
        hours,
        minutes,
        seconds,
        frames,
    }
}

/// # Converts SMPTE timecode to microseconds (30fps non-drop frame).
///
/// This function performs the inverse operation of [`us_to_smpte`]
/// # Arguments
///
/// * `smpte` - SMPTE timecode structure
///
/// # Returns
///
/// Timestamp in microseconds
pub fn smpte_to_us(smpte: &MtcFullFrame) -> VlcTickT {
    let frame_rate = SMPTE_30_FPS;
    let frame_rate_base = SMPTE_30_FPS_BASE;

    let framenum = (smpte.hours as u64 * 3600 * frame_rate as u64 / frame_rate_base as u64)
        + (smpte.minutes as u64 * 60 * frame_rate as u64 / frame_rate_base as u64)
        + (smpte.seconds as u64 * frame_rate as u64 / frame_rate_base as u64)
        + (smpte.frames as u64);

    (framenum * frame_rate_base as u64 * VLC_TICK_FROM_SEC) / frame_rate as u64
}

/// # Converts SMPTE timecode to 8 MTC quarter frames.
///
/// This function splits the SMPTE timecode components into 8 quarter frames according to the MTC specification.
/// Each quarter frame contains 4 bits of data representing different parts of the timecode.
///
/// The frame rate type is set to 30fps non-drop frame (binary 11 = 0x03).
///
/// # Arguments
///
/// * `smpte` - SMPTE timecode structure to convert
///
/// # Returns
///
/// Array of 8 MTC quarter frames containing the timecode data
pub fn smpte_to_quarter_frames(smpte: &MtcFullFrame) -> [MtcQuarterFrame; 8] {
    [
        MtcQuarterFrame {
            frame_type: 0,
            value: smpte.frames & 0x0F,
        },
        MtcQuarterFrame {
            frame_type: 1,
            value: (smpte.frames >> 4) & 0x01,
        },
        MtcQuarterFrame {
            frame_type: 2,
            value: smpte.seconds & 0x0F,
        },
        MtcQuarterFrame {
            frame_type: 3,
            value: (smpte.seconds >> 4) & 0x03,
        },
        MtcQuarterFrame {
            frame_type: 4,
            value: smpte.minutes & 0x0F,
        },
        MtcQuarterFrame {
            frame_type: 5,
            value: (smpte.minutes >> 4) & 0x03,
        },
        MtcQuarterFrame {
            frame_type: 6,
            value: smpte.hours & 0x0F,
        },
        MtcQuarterFrame {
            frame_type: 7,
            value: ((smpte.hours >> 4) & 0x01) | (0x03 << 1), // 30fps non-drop = 11 (0x03)
        },
    ]
}

/// # Converts 8 MTC quarter frames to SMPTE timecode.
///
/// This function reconstructs SMPTE timecode from 8 MTC quarter frames. It validates that:
/// - Frame types are in correct sequence (0-7)
/// - Values are within valid ranges for each component
/// - Frame rate type indicates 30fps non-drop frame
/// - Resulting timecode values are within valid SMPTE ranges
///
/// Limitiations:
/// Does not allow duplicate frames (some maybe sent twice for redundancy, should be supported later)
///
/// # Arguments
///
/// * `frames` - Array of exactly 8 MTC quarter frames
///
/// # Returns
///
/// * `Ok(Mtc_full)` - Successfully reconstructed SMPTE timecode
/// * `Err(MtcError)` - Invalid frame data or unsupported frame rate
pub fn quarter_frames_to_smpte(frames: &[MtcQuarterFrame; 8]) -> Result<MtcFullFrame, MtcError> {
    // Validate frame types are in correct order (0-7) and values are within range
    for (i, frame) in frames.iter().enumerate() {
        if frame.frame_type != i as u8 {
            return Err(MtcError::InvalidFrameType);
        }
        if frame.value > 15 {
            return Err(MtcError::InvalidValue);
        }
    }

    // Reconstruct timecode components from quarter frames
    let frames_val = frames[0].value | ((frames[1].value & 0x01) << 4);
    let seconds_val = frames[2].value | ((frames[3].value & 0x03) << 4);
    let minutes_val = frames[4].value | ((frames[5].value & 0x03) << 4);
    let hours_val = frames[6].value | ((frames[7].value & 0x01) << 4);

    // Validate SMPTE ranges
    if frames_val > 29 {
        // 30fps non-drop frame: frames 0-29
        return Err(MtcError::InvalidValue);
    }
    if seconds_val > 59 {
        return Err(MtcError::InvalidValue);
    }
    if minutes_val > 59 {
        return Err(MtcError::InvalidValue);
    }
    if hours_val > 23 {
        return Err(MtcError::InvalidValue);
    }

    // Extract and validate frame rate from frame 7 (bits 1-2)
    let frame_rate_type = (frames[7].value >> 1) & 0x03;
    if frame_rate_type != 0x03 {
        // Only 30fps non-drop (11 binary = 0x03) is supported
        return Err(MtcError::InvalidValue);
    }

    Ok(MtcFullFrame {
        hours: hours_val,
        minutes: minutes_val,
        seconds: seconds_val,
        frames: frames_val,
    })
}
