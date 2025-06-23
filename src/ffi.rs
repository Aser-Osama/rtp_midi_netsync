//! # FFI Layer for RTP-MIDI Netsync
//!
//! This module provides C-compatible interfaces for the RTP-MIDI netsync functionality.
//! It uses C-provided buffers to avoid heap allocation and libc dependency, making it
//! suitable for embedded systems and real-time applications.
//!
//! ## Overview
//!
//! The FFI layer enables C/C++ applications to:
//! - Convert MIDI timing events to RTP-MIDI network payloads (master mode)
//! - Parse RTP-MIDI network payloads back to MIDI events (slave mode)
//! - Handle MTC (MIDI Time Code) and MMC (MIDI Machine Control) messages
//!
//! ## Usage Pattern
//!
//! For master (sender) applications:
//! 1. Create a `CMidiEvent` using helper functions
//! 2. Call `master_netsync_flow_ffi()` to generate network payload
//! 3. Send the payload over the network
//!
//! For slave (receiver) applications:
//! 1. Receive network payload
//! 2. Call `slave_netsync_flow_ffi()` to parse into `CMidiEvent`
//! 3. Process the MIDI event as needed
//!
//! ## Memory Safety
//!
//! All functions use C-provided buffers and avoid dynamic allocation.
//! Callers must ensure proper buffer sizing using `get_max_payload_size()`.

use std::ffi::{c_char, c_int};
use std::slice;

use crate::midi::{MidiEvent, MmcCommand};
use crate::netsync::{master_netsync_flow, slave_netsync_flow};

// ============================================================================
// FFI TYPE DEFINITIONS
// ============================================================================

/// C-compatible MIDI event types supported by the netsync system
///
/// These correspond to timing-related MIDI messages:
/// - MTC: MIDI Time Code for synchronization
/// - MMC: MIDI Machine Control for transport commands
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CMidiEventType {
    /// MTC Quarter Frame message (incremental time updates)
    MtcQuarter = 0,
    /// MTC Full Frame message (absolute time position)
    MtcFull = 1,
    /// MMC Stop command
    MmcStop = 2,
    /// MMC Play command
    MmcPlay = 3,
    /// MMC Locate command (jump to specific time)
    MmcLocate = 4,
}

/// C-compatible MIDI event structure
///
/// This structure can hold any supported MIDI event type with a fixed-size
/// data buffer to avoid dynamic allocation.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CMidiEvent {
    /// Type of MIDI event
    pub event_type: CMidiEventType,
    /// Raw event data (interpretation depends on event_type)
    pub data: [u8; 8], // Maximum size needed for any supported event
    /// Number of valid bytes in the data array
    pub data_len: u8,
}

/// Error codes returned by FFI functions
///
/// All FFI functions return these error codes as c_int values.
/// Use `get_error_message()` to get human-readable descriptions.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CErrorCode {
    /// Operation completed successfully
    Success = 0,
    /// Invalid MIDI event provided to master flow
    InvalidMasterEvent = 1,
    /// Invalid network payload provided to slave flow
    InvalidSlaveEvent = 2,
    /// Provided buffer is too small for the operation
    BufferTooSmall = 3,
    /// Null pointer passed where valid pointer expected
    NullPointer = 4,
    /// Unsupported or malformed event type
    InvalidEventType = 5,
}

// ============================================================================
// INTERNAL CONVERSION FUNCTIONS
// ============================================================================

/// Convert C-compatible CMidiEvent to internal Rust MidiEvent
///
/// Validates the event data and ensures proper field mapping.
/// Used for master instance
/// # Arguments
/// * `c_event` - C event structure to convert
///
/// # Returns
/// * `Ok(MidiEvent)` - Successfully converted event
/// * `Err(CErrorCode)` - Conversion failed due to invalid data
fn c_to_midi_event(c_event: &CMidiEvent) -> Result<MidiEvent, CErrorCode> {
    // Safely extract the raw event type value without triggering UB
    // We read the memory as a raw u32 to avoid enum validation
    let event_type_raw: u32 =
        unsafe { std::ptr::read(&c_event.event_type as *const CMidiEventType as *const u32) };

    match event_type_raw {
        0 => {
            // CMidiEventType::MtcQuarter
            // MTC Quarter Frame: msg_type (0-7) + value (0-15)
            // Must have exactly 2 bytes
            if c_event.data_len != 2 {
                return Err(CErrorCode::InvalidEventType);
            }
            Ok(MidiEvent::MtcQuarter {
                msg_type: c_event.data[0],
                value: c_event.data[1],
            })
        }
        1 => {
            // CMidiEventType::MtcFull
            // MTC Full Frame: hour + minute + second + frame
            // Must have exactly 4 bytes
            if c_event.data_len != 4 {
                return Err(CErrorCode::InvalidEventType);
            }
            Ok(MidiEvent::MtcFull {
                hour: c_event.data[0],
                minute: c_event.data[1],
                second: c_event.data[2],
                frame: c_event.data[3],
            })
        }
        2 => {
            // CMidiEventType::MmcStop
            // MMC Stop: no additional data needed
            // Must have exactly 0 bytes
            if c_event.data_len != 0 {
                return Err(CErrorCode::InvalidEventType);
            }
            Ok(MidiEvent::Mmc(MmcCommand::Stop))
        }
        3 => {
            // CMidiEventType::MmcPlay
            // MMC Play: no additional data needed
            // Must have exactly 0 bytes
            if c_event.data_len != 0 {
                return Err(CErrorCode::InvalidEventType);
            }
            Ok(MidiEvent::Mmc(MmcCommand::Play))
        }
        4 => {
            // CMidiEventType::MmcLocate
            // MMC Locate: hour + minute + second + frame (subframe set to 0)
            // Must have exactly 4 bytes
            if c_event.data_len != 4 {
                return Err(CErrorCode::InvalidEventType);
            }
            Ok(MidiEvent::Mmc(MmcCommand::Locate {
                hour: c_event.data[0],
                minute: c_event.data[1],
                second: c_event.data[2],
                frame: c_event.data[3],
                subframe: 0, // Always 0 for FFI compatibility
            }))
        }
        _ => {
            // Invalid event type, accessed when enum value is passed in as an unsuppprted int value
            //  (error-case exists in the testfile "test_ffi_stress.c")
            Err(CErrorCode::InvalidEventType)
        }
    }
}

/// Convert internal Rust MidiEvent to C-compatible CMidiEvent
///
/// Maps Rust event structures to the fixed C layout.
/// Used for slave instance
///
/// # Arguments
/// * `event` - Rust event to convert
///
/// # Returns
/// * `Ok(CMidiEvent)` - Successfully converted event
/// * `Err(CErrorCode)` - Event type not supported in C interface
fn midi_event_to_c(event: &MidiEvent) -> Result<CMidiEvent, CErrorCode> {
    match event {
        MidiEvent::MtcQuarter { msg_type, value } => Ok(CMidiEvent {
            event_type: CMidiEventType::MtcQuarter,
            data: {
                let mut data = [0u8; 8];
                data[0] = *msg_type;
                data[1] = *value;
                data
            },
            data_len: 2,
        }),
        MidiEvent::MtcFull {
            hour,
            minute,
            second,
            frame,
        } => Ok(CMidiEvent {
            event_type: CMidiEventType::MtcFull,
            data: {
                let mut data = [0u8; 8];
                data[0] = *hour;
                data[1] = *minute;
                data[2] = *second;
                data[3] = *frame;
                data
            },
            data_len: 4,
        }),
        MidiEvent::Mmc(MmcCommand::Stop) => Ok(CMidiEvent {
            event_type: CMidiEventType::MmcStop,
            data: [0u8; 8],
            data_len: 0,
        }),
        MidiEvent::Mmc(MmcCommand::Play) => Ok(CMidiEvent {
            event_type: CMidiEventType::MmcPlay,
            data: [0u8; 8],
            data_len: 0,
        }),
        MidiEvent::Mmc(MmcCommand::Locate {
            hour,
            minute,
            second,
            frame,
            subframe: _, // Ignored in C interface
        }) => Ok(CMidiEvent {
            event_type: CMidiEventType::MmcLocate,
            data: {
                let mut data = [0u8; 8];
                data[0] = *hour;
                data[1] = *minute;
                data[2] = *second;
                data[3] = *frame;
                data
            },
            data_len: 4,
        }),
        // Other event types are not supported in the C interface
        MidiEvent::Other(_) => Err(CErrorCode::InvalidEventType),
    }
}

// ============================================================================
// CORE FFI FUNCTIONS
// ============================================================================

/// Master netsync flow: Convert MIDI event to RTP-MIDI network payload
///
/// This function is used by master (sender) applications to convert timing
/// events into network payloads that can be transmitted to slave devices.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers. Callers must ensure:
/// - `event` points to a valid, properly initialized `CMidiEvent`
/// - `buffer` points to a writable buffer of at least `buffer_size` bytes
/// - `actual_size` points to a writable `usize` location
/// - All pointers remain valid for the duration of the call
///
/// # Arguments
/// * `event` - Pointer to the MIDI event to convert
/// * `buffer` - Destination buffer for the network payload
/// * `buffer_size` - Size of the destination buffer in bytes
/// * `actual_size` - Output: actual number of bytes written to buffer
///
/// # Returns
/// * `0` (Success) - Payload generated successfully
/// * Non-zero - Error code (see `CErrorCode` enum)
///
/// # Example Usage (C)
/// ```c
/// CMidiEvent event = create_mtc_quarter_event(0, 5);
/// uint8_t buffer[16];
/// size_t actual_size;
/// int result = master_netsync_flow_ffi(&event, buffer, sizeof(buffer), &actual_size);
/// if (result == 0) {
///     // Send buffer[0..actual_size] over network
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn master_netsync_flow_ffi(
    event: *const CMidiEvent,
    buffer: *mut u8,
    buffer_size: usize,
    actual_size: *mut usize,
) -> c_int {
    // Validate all pointers before use
    if event.is_null() || buffer.is_null() || actual_size.is_null() {
        return CErrorCode::NullPointer as c_int;
    }

    // Initialize output parameter to safe default
    *actual_size = 0;

    // Safely dereference the event pointer
    let c_event = &*event;

    // Convert C event structure to internal Rust representation
    let rust_event = match c_to_midi_event(c_event) {
        Ok(event) => event,
        Err(error_code) => return error_code as c_int,
    };

    // Generate the network payload using core netsync logic
    let payload = match master_netsync_flow(&rust_event) {
        Ok(payload) => payload,
        Err(_) => return CErrorCode::InvalidMasterEvent as c_int,
    };

    // Ensure the provided buffer is large enough
    if payload.len() > buffer_size {
        return CErrorCode::BufferTooSmall as c_int;
    }

    // Copy payload data to the C-provided buffer
    let buffer_slice = slice::from_raw_parts_mut(buffer, buffer_size);
    buffer_slice[..payload.len()].copy_from_slice(&payload);

    // Report the actual number of bytes written
    *actual_size = payload.len();

    CErrorCode::Success as c_int
}

/// Slave netsync flow: Parse RTP-MIDI network payload to MIDI event
///
/// This function is used by slave (receiver) applications to parse incoming
/// network payloads back into MIDI timing events.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers. Callers must ensure:
/// - `buffer` points to a readable buffer of at least `buffer_len` bytes
/// - `event` points to a writable `CMidiEvent` structure
/// - Both pointers remain valid for the duration of the call
///
/// # Arguments
/// * `buffer` - Source buffer containing the network payload
/// * `buffer_len` - Length of the source buffer in bytes
/// * `event` - Output: parsed MIDI event structure
///
/// # Returns
/// * `0` (Success) - Event parsed successfully
/// * Non-zero - Error code (see `CErrorCode` enum)
///
/// # Example Usage (C)
/// ```c
/// uint8_t payload[] = { /* received network data */ };
/// CMidiEvent event;
/// int result = slave_netsync_flow_ffi(payload, sizeof(payload), &event);
/// if (result == 0) {
///     // Process the parsed event
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn slave_netsync_flow_ffi(
    buffer: *const u8,
    buffer_len: usize,
    event: *mut CMidiEvent,
) -> c_int {
    // Validate all pointers before use
    if buffer.is_null() || event.is_null() {
        return CErrorCode::NullPointer as c_int;
    }

    // Create a safe slice from the raw buffer pointer
    let buf = slice::from_raw_parts(buffer, buffer_len);

    // Parse the network payload using core netsync logic
    let midi_event = match slave_netsync_flow(buf) {
        Ok(event) => event,
        Err(_) => return CErrorCode::InvalidSlaveEvent as c_int,
    };

    // Convert the parsed event to C-compatible format
    let c_event = match midi_event_to_c(&midi_event) {
        Ok(event) => event,
        Err(error_code) => return error_code as c_int,
    };

    // Write the result to the output parameter
    *event = c_event;

    CErrorCode::Success as c_int
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Get the maximum buffer size needed for network payloads
///
/// Use this function to allocate appropriately sized buffers for
/// `master_netsync_flow_ffi()`. The returned size is guaranteed to
/// accommodate any event type supported by this library.
///
/// # Returns
/// Maximum buffer size in bytes needed for any payload
///
/// # Example Usage (C)
/// ```c
/// size_t max_size = get_max_payload_size();
/// uint8_t* buffer = malloc(max_size);
/// ```
#[no_mangle]
pub extern "C" fn get_max_payload_size() -> usize {
    // MMC Locate has the largest payload:
    // - 1 byte header + 13 bytes MMC data = 14 bytes
    // - Adding padding for safety and future expansion
    16
}

/// Get human-readable error message for an error code
///
/// Converts numeric error codes returned by FFI functions into
/// descriptive English text for debugging and user feedback.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the program
/// and points to a null-terminated C string. Do not free the pointer.
///
/// # Arguments
/// * `error_code` - Error code from any FFI function
///
/// # Returns
/// Pointer to null-terminated error message string
///
/// # Example Usage (C)
/// ```c
/// int result = master_netsync_flow_ffi(...);
/// if (result != 0) {
///     const char* message = get_error_message(result);
///     printf("Error: %s\n", message);
/// }
/// ```
#[no_mangle]
pub extern "C" fn get_error_message(error_code: c_int) -> *const c_char {
    let message = match error_code {
        x if x == CErrorCode::Success as c_int => "Success\0",
        x if x == CErrorCode::InvalidMasterEvent as c_int => "Invalid master event\0",
        x if x == CErrorCode::InvalidSlaveEvent as c_int => "Invalid slave event\0",
        x if x == CErrorCode::BufferTooSmall as c_int => "Buffer too small\0",
        x if x == CErrorCode::NullPointer as c_int => "Null pointer passed\0",
        x if x == CErrorCode::InvalidEventType as c_int => "Invalid event type\0",
        _ => "Unknown error\0",
    };
    message.as_ptr() as *const c_char
}

// ============================================================================
// HELPER FUNCTIONS FOR EVENT CREATION
// ============================================================================

/// Create MTC Quarter Frame event
///
/// MTC Quarter Frame messages provide incremental time updates,
/// sending 2 nibbles of time information per message in sequence.
///
/// # Arguments
/// * `msg_type` - Message type/piece number (0-7, indicates which time component)
/// * `value` - Data nibble value (0-15, the actual time data)
///
/// # Returns
/// Initialized `CMidiEvent` structure for MTC Quarter Frame
///
/// # Message Type Values
/// * 0: Frame count LS nibble
/// * 1: Frame count MS nibble
/// * 2: Seconds count LS nibble
/// * 3: Seconds count MS nibble
/// * 4: Minutes count LS nibble
/// * 5: Minutes count MS nibble
/// * 6: Hours count LS nibble
/// * 7: Hours count MS nibble + SMPTE type
#[no_mangle]
pub extern "C" fn create_mtc_quarter_event(msg_type: u8, value: u8) -> CMidiEvent {
    CMidiEvent {
        event_type: CMidiEventType::MtcQuarter,
        data: {
            let mut data = [0u8; 8];
            data[0] = msg_type;
            data[1] = value;
            data
        },
        data_len: 2,
    }
}

/// Create MTC Full Frame event
///
/// MTC Full Frame messages provide complete absolute time position
/// in a single message, typically used for initial synchronization.
///
/// # Arguments
/// * `hour` - Hours (0-23)
/// * `minute` - Minutes (0-59)
/// * `second` - Seconds (0-59)
/// * `frame` - Frame number (0-29, depending on frame rate)
///
/// # Returns
/// Initialized `CMidiEvent` structure for MTC Full Frame
#[no_mangle]
pub extern "C" fn create_mtc_full_event(hour: u8, minute: u8, second: u8, frame: u8) -> CMidiEvent {
    CMidiEvent {
        event_type: CMidiEventType::MtcFull,
        data: {
            let mut data = [0u8; 8];
            data[0] = hour;
            data[1] = minute;
            data[2] = second;
            data[3] = frame;
            data
        },
        data_len: 4,
    }
}

/// Create MMC Stop event
///
/// MMC Stop commands instruct synchronized devices to halt playback/recording.
///
/// # Returns
/// Initialized `CMidiEvent` structure for MMC Stop
#[no_mangle]
pub extern "C" fn create_mmc_stop_event() -> CMidiEvent {
    CMidiEvent {
        event_type: CMidiEventType::MmcStop,
        data: [0u8; 8],
        data_len: 0,
    }
}

/// Create MMC Play event
///
/// MMC Play commands instruct synchronized devices to begin playback/recording.
///
/// # Returns
/// Initialized `CMidiEvent` structure for MMC Play
#[no_mangle]
pub extern "C" fn create_mmc_play_event() -> CMidiEvent {
    CMidiEvent {
        event_type: CMidiEventType::MmcPlay,
        data: [0u8; 8],
        data_len: 0,
    }
}

/// Create MMC Locate event
///
/// MMC Locate commands instruct synchronized devices to jump to
/// a specific time position before starting playback/recording.
///
/// # Arguments
/// * `hour` - Target hours (0-23)
/// * `minute` - Target minutes (0-59)
/// * `second` - Target seconds (0-59)
/// * `frame` - Target frame (0-29, depending on frame rate)
///
/// # Returns
/// Initialized `CMidiEvent` structure for MMC Locate
#[no_mangle]
pub extern "C" fn create_mmc_locate_event(
    hour: u8,
    minute: u8,
    second: u8,
    frame: u8,
) -> CMidiEvent {
    CMidiEvent {
        event_type: CMidiEventType::MmcLocate,
        data: {
            let mut data = [0u8; 8];
            data[0] = hour;
            data[1] = minute;
            data[2] = second;
            data[3] = frame;
            data
        },
        data_len: 4,
    }
}
