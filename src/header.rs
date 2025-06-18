//! MIDI Payload Header Parsing and Serialization
//!
//! This module implements the a basic verison of the MIDI Command Section header as defined in RFC 6295, Section 3.
//!
//! ## Protocol Structure
//!
//! The MIDI command section header follows this bit layout:
//!
//! ```text
//! 0                   1                   2                   3
//! 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |B|J|Z|P|LEN... |  MIDI list ...                                |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!
//!                Figure 2 -- MIDI Command Section
//! ```
//!
//! ## Flag Definitions
//!
//! - **B**: If set to 1, the header is two octets long, and LEN becomes a 12-bit field
//! - **J**: If set to 1, a journal section MUST appear after the MIDI command section
//! - **Z**: If set to 1, the MIDI list starts with an explicit delta-time followed by the first MIDI command
//! - **P**: If set to 1, delta times are encoded in a "packed" form
//!
//! ## Implementation Note
//!
//! For this implementation, all flags are set to 0 and are not actively used.

/// Represents a MIDI payload header containing flags and length information.
///
/// The header is packed into a single byte where:
/// - The upper 4 bits contain the flags (B, J, Z, P)
/// - The lower 4 bits contain the length field

#[derive(Debug, PartialEq)]
pub struct PayloadHeader {
    /// The 4-bit flags field containing B, J, Z, P flags as defined in RFC 6295.
    /// Currently, all flags are expected to be set to 0.
    pub flags: u8,

    /// The 4-bit length field indicating the length of the MIDI command section.
    ///
    /// When the B flag is 0 (single octet header), this field can represent
    /// values from 0 to 15. When B is 1, this becomes part of a 12-bit length field.
    pub len: u8, // Length (4 bits)
}

impl PayloadHeader {
    /// Parses a payload header from a byte buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - A byte slice containing at least 1 byte of header data
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the parsed `PayloadHeader` on success,
    /// or a `ParseError` if the buffer is insufficient.
    ///
    /// # Errors
    ///
    /// Returns `ParseError::InsufficientHeaderData` if the buffer contains
    /// fewer than 1 byte.
    pub fn parse(buf: &[u8]) -> Result<Self, super::error::ParseError> {
        if buf.len() < 1 {
            return Err(super::error::ParseError::InsufficientHeaderData);
        }
        let byte = buf[0];
        Ok(PayloadHeader {
            flags: byte >> 4,
            len: byte & 0x0F,
        })
    }

    /// Serializes the payload header into a single byte.
    ///
    /// The flags are placed in the upper 4 bits and the length
    /// is placed in the lower 4 bits of the resulting byte.
    ///
    /// # Returns
    ///
    /// Returns a 1-byte array containing the serialized header.
    pub fn serialize(&self) -> [u8; 1] {
        [(self.flags << 4) | (self.len & 0x0F)]
    }

    /// Creates a new `PayloadHeader` with the specified flags and length.
    ///
    /// # Arguments
    ///
    /// * `flags` - The 4-bit flags value (values > 15 will be masked)
    /// * `len` - The 4-bit length value (values > 15 will be masked)
    pub fn new(flags: u8, len: u8) -> Self {
        Self {
            flags: flags & 0x0F, // Ensure only 4 bits are used
            len: len & 0x0F,     // Ensure only 4 bits are used
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_roundtrip() {
        let hdr = PayloadHeader {
            flags: 0b1010,
            len: 0x0F,
        };
        let bytes = hdr.serialize();
        let parsed = PayloadHeader::parse(&bytes).unwrap();
        assert_eq!(parsed, hdr);
    }

    #[test]
    fn new_constructor() {
        let header = PayloadHeader::new(5, 10);
        assert_eq!(header.flags, 5);
        assert_eq!(header.len, 10);

        // Test masking of values > 15
        let header_masked = PayloadHeader::new(20, 18);
        assert_eq!(header_masked.flags, 4); // 20 & 0x0F = 4
        assert_eq!(header_masked.len, 2); // 18 & 0x0F = 2
    }

    #[test]
    fn parse_insufficient_data() {
        let empty_buf: &[u8] = &[];
        let result = PayloadHeader::parse(empty_buf);
        assert!(result.is_err());
    }

    #[test]
    fn serialize_all_flags_set() {
        let header = PayloadHeader {
            flags: 0b1111,
            len: 0b1111,
        };
        let bytes = header.serialize();
        assert_eq!(bytes[0], 0xFF);
    }

    #[test]
    fn serialize_no_flags_set() {
        let header = PayloadHeader {
            flags: 0b0000,
            len: 0b0101,
        };
        let bytes = header.serialize();
        assert_eq!(bytes[0], 0x05);
    }
}
