use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    Parse(ParseError),
    Mtc(MtcError),
    NetsyncError(NetsyncError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "Parse error: {}", e),
            Error::Mtc(e) => write!(f, "MTC error: {}", e),
            Error::NetsyncError(e) => write!(f, "Netsync flow error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parse(e) => Some(e),
            Error::Mtc(e) => Some(e),
            Error::NetsyncError(e) => Some(e),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

/// Error types for MTC quarter frame processing.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MtcError {
    /// Fewer than 8 quarter frames provided
    IncompleteFrame,
    /// Frame type outside valid range (0-7)
    InvalidFrameType,
    /// Value outside valid range or resulting timecode invalid
    InvalidValue,
}

impl std::fmt::Display for MtcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MtcError::IncompleteFrame => {
                write!(f, "Incomplete MTC frame: need exactly 8 quarter frames")
            }
            MtcError::InvalidFrameType => write!(f, "Invalid frame type: must be 0-7"),
            MtcError::InvalidValue => write!(
                f,
                "Invalid value: exceeds valid range or creates invalid timecode"
            ),
        }
    }
}

impl std::error::Error for MtcError {}
impl std::error::Error for NetsyncError {}

// Convenience From implementations
impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::Parse(err)
    }
}

impl From<MtcError> for Error {
    fn from(err: MtcError) -> Self {
        Error::Mtc(err)
    }
}

impl From<NetsyncError> for Error {
    fn from(err: NetsyncError) -> Self {
        Error::NetsyncError(err)
    }
}

/// Error types for MTC quarter frame processing.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NetsyncError {
    /// Invalid Master Event Entered
    InvalidMasterEvent,
    InvalidSlaveEvent,
}

impl std::fmt::Display for NetsyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetsyncError::InvalidMasterEvent => {
                write!(f, "InvalidMasterEvent")
            }
            NetsyncError::InvalidSlaveEvent => {
                write!(f, "InvalidSlaveEvent")
            }
        }
    }
}
