use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    Parse(ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parse(e) => Some(e),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// Buffer is empty or length is zero
    EmptyBuffer,
    /// Requested length exceeds buffer size, the requested length comes from the payload header
    BufferTooSmall { requested: usize, available: usize },
    /// Insufficient data to parse payload header (need at least 1 byte)
    InsufficientHeaderData,
    /// Invalid or corrupted MIDI data at specified position
    InvalidMidiData { position: usize, byte: u8 },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyBuffer => {
                write!(f, "Cannot parse MIDI data from empty buffer")
            }
            ParseError::BufferTooSmall {
                requested,
                available,
            } => {
                write!(
                    f,
                    "Requested {} bytes but only {} available in buffer",
                    requested, available
                )
            }
            ParseError::InsufficientHeaderData => {
                write!(f, "Need at least 1 byte to parse payload header")
            }
            ParseError::InvalidMidiData { position, byte } => {
                write!(
                    f,
                    "Invalid MIDI data at position {}: 0x{:02X}",
                    position, byte
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}
