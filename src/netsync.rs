//! # Netsync main rust orchestrator
//! This module provides the core functionality for RTP-MIDI netsync, implementing bidirectional conversion between MIDI events and network payload format.
//!
//!
//! The master flow converts MIDI synchronization events (MTC, MMC) into network payloads suitable for transmission over RTP-MIDI. This involves:
//! The slave flow reconstructs MIDI events from received network payloads by:
//!
//! Supported Events:
//!
//! - Quarter-Frame MTC (MIDI Time Code) -- But not used in practice
//! - Full-Frame MTC
//! - MMC (MIDI Machine Control) Start/Stop commands
//! - MMC Locate commands

use crate::error::NetsyncError;
use crate::header::PayloadHeader;
use crate::midi::{build_midi_list, parse_midi_list, MidiEvent, MmcCommand};
use crate::midi::{
    MMC_LOCATE_LENGTH, MMC_START_STOP_LENGTH, MTC_FULL_FRAME_LENGTH, MTC_QUARTER_FRAME_LENGTH,
};

/// Converts a MIDI synchronization event into a network payload for master transmission.
///
/// This function takes a structured MIDI event and converts it into a byte payload
/// suitable for transmission over the network. The payload includes a header with
/// length information followed by the serialized MIDI data.
///
/// # Arguments
///
/// * `event` - The MIDI synchronization event to convert. Must be a valid sync event
///             (MTC or MMC), not `MidiEvent::Other`.
///
/// # Returns
///
/// Returns a `Vec<u8>` containing the complete network payload on success.
///
/// # Errors
///
/// * `NetsyncError::InvalidMasterEvent` - If the event is `MidiEvent::Other` or
///   another unsupported event type for master synchronization.
pub fn master_netsync_flow(event: &MidiEvent) -> Result<Vec<u8>, NetsyncError> {
    // Determine header length based on event type
    let header_len = match event {
        MidiEvent::Mmc(MmcCommand::Play | MmcCommand::Stop) => MMC_START_STOP_LENGTH,
        MidiEvent::Mmc(MmcCommand::Locate { .. }) => MMC_LOCATE_LENGTH,
        MidiEvent::MtcFull { .. } => MTC_FULL_FRAME_LENGTH,
        MidiEvent::MtcQuarter { .. } => MTC_QUARTER_FRAME_LENGTH,
        MidiEvent::Other(_) => return Err(NetsyncError::InvalidMasterEvent),
    };

    // Build MIDI data
    let midi_data = build_midi_list(event);

    // Create payload with proper capacity
    let mut payload = Vec::with_capacity(1 + midi_data.len());

    // Add header (currently always 1 byte)
    let header = PayloadHeader::new(0x0, header_len as u8);
    payload.extend_from_slice(&header.serialize());

    // Add MIDI data
    payload.extend_from_slice(&midi_data);

    Ok(payload)
}

/// Reconstructs a MIDI synchronization event from a received network payload.
///
/// This function parses a network payload received from a master device and
/// reconstructs the original MIDI synchronization event. It validates the
/// payload structure and extracts the MIDI data based on the header information.
///
/// # Arguments
///
/// * `buf` - The received network payload bytes. Must contain at least a header
///           and the corresponding MIDI data.
///
/// # Returns
///
/// Returns the reconstructed `MidiEvent` on successful parsing.
///
/// # Errors
///
/// * `NetsyncError::InvalidSlaveEvent` - If the buffer is too small to contain
///   a valid payload (less than 3 bytes for the smallest valid message).
/// * `ParseError::InsufficientHeaderData` - If the buffer doesn't contain enough
///   data to parse the header.
/// * `ParseError::BufferTooSmall` - If the buffer is smaller than the length
///   specified in the header.
/// * `ParseError::InvalidMidiData` - If the MIDI data in the payload is malformed.
/// ```
pub fn slave_netsync_flow(buf: &[u8]) -> Result<MidiEvent, NetsyncError> {
    // Check minimum payload size (1 byte header + 2 bytes MTC Quarter Frame = 3 bytes minimum)
    const MIN_PAYLOAD_SIZE: usize = 3;
    if buf.len() < MIN_PAYLOAD_SIZE {
        return Err(NetsyncError::InvalidSlaveEvent);
    }

    // Parse header
    let header = PayloadHeader::parse(&buf[..1]).map_err(|_| NetsyncError::InvalidSlaveEvent)?;

    // Parse MIDI data with proper error propagation
    let midi_event = parse_midi_list(&buf[1..], header.len as usize)
        .map_err(|_| NetsyncError::InvalidSlaveEvent)?;

    Ok(midi_event)
}
