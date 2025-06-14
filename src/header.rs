/* 3.  MIDI Command Section
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|B|J|Z|P|LEN... |  MIDI list ...                                |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

               Figure 2 -- MIDI Command Section
*/

#[derive(Debug, PartialEq)]
pub struct PayloadHeader {
    pub flags: u8, // Flags (4 bits)
    pub len: u8,   // Length (4 bits)
}

impl PayloadHeader {
    pub fn parse(buf: &[u8]) -> Result<Self, super::error::ParseError> {
        if buf.len() < 1 {
            return Err(super::error::ParseError::Insufficient);
        }
        let byte = buf[0];
        Ok(PayloadHeader {
            flags: byte >> 4,
            len: byte & 0x0F,
        })
    }

    pub fn serialize(&self) -> [u8; 1] {
        [(self.flags << 4) | (self.len & 0x0F)]
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
}
