use std::io::{Read, Write};

use crate::RcwtError;
use crate::FTS;

const RCWT_MAGIC: [u8; 3] = [0xCC, 0xCC, 0xED];

pub const RCWT_CREATING_PROGRAM: u8 = 0x72; // 'r' for rcwt-rs
pub const RCWT_PROGRAM_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHeader {
    pub magic_number: [u8; 3],
    pub creating_program: u8,
    pub program_version: u16,
    pub file_format_version: u16,
    pub reserved: [u8; 3],
}

impl FileHeader {
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self, RcwtError> {
        let mut magic_number = [0; 3];
        reader.read_exact(&mut magic_number)?;
        if magic_number != RCWT_MAGIC {
            return Err(RcwtError::InvalidHeader);
        }
        let mut creating_buf = [0; 1];
        reader.read_exact(&mut creating_buf)?;
        let creating_program = creating_buf[0];

        let mut buf = [0; 2];
        reader.read_exact(&mut buf)?;
        let program_version = u16::from_be_bytes(buf);
        reader.read_exact(&mut buf)?;
        let file_format_version = u16::from_be_bytes(buf);

        let mut reserved = [0; 3];
        reader.read_exact(&mut reserved)?;

        if file_format_version != 1 {
            return Err(RcwtError::UnsupportedVersion(file_format_version));
        }

        Ok(FileHeader {
            magic_number,
            creating_program,
            program_version,
            file_format_version,
            reserved,
        })
    }

    pub fn write_rcwt<W: Write>(&self, writer: &mut W) -> Result<(), RcwtError> {
        writer.write_all(&self.magic_number)?;
        writer.write_all(&[self.creating_program])?;
        writer.write_all(&self.program_version.to_be_bytes())?;
        writer.write_all(&self.file_format_version.to_be_bytes())?;
        writer.write_all(&self.reserved)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeHeader {
    pub fts: FTS,
    pub num_blocks: u16,
}

impl TimeHeader {
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self, RcwtError> {
        let mut fts_buf = [0; 8];
        if !read_exact_or_eof(reader, &mut fts_buf)? {
            return Err(RcwtError::Eof);
        }

        let fts = FTS(u64::from_le_bytes(fts_buf));

        let mut num_blocks_buf = [0; 2];
        reader.read_exact(&mut num_blocks_buf)?;
        let num_blocks = u16::from_le_bytes(num_blocks_buf);

        Ok(TimeHeader { fts, num_blocks })
    }

    pub fn write_rcwt<W: Write>(&self, writer: &mut W) -> Result<(), RcwtError> {
        writer.write_all(&self.fts.0.to_le_bytes())?;
        writer.write_all(&self.num_blocks.to_le_bytes())?;
        Ok(())
    }
}

fn read_exact_or_eof<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<bool, RcwtError> {
    let mut total = 0;
    while total < buf.len() {
        match reader.read(&mut buf[total..])? {
            0 => return Ok(false),
            n => total += n,
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_header_roundtrip() {
        let header = FileHeader {
            magic_number: [0xCC, 0xCC, 0xED],
            creating_program: 0xCC,
            program_version: 80,
            file_format_version: 1,
            reserved: [0, 0, 0],
        };
        let mut buf = Vec::new();
        header.write_rcwt(&mut buf).unwrap();

        let parsed = FileHeader::parse(&mut buf.as_slice()).unwrap();
        assert_eq!(header, parsed);
    }

    #[test]
    fn time_header_roundtrip() {
        let header = TimeHeader {
            fts: FTS(12345),
            num_blocks: 3,
        };
        let mut buf = Vec::new();
        header.write_rcwt(&mut buf).unwrap();

        let parsed = TimeHeader::parse(&mut buf.as_slice()).unwrap();
        assert_eq!(header, parsed);
    }

    #[test]
    fn time_header_eof_returns_eof_error() {
        let result = TimeHeader::parse(&mut &b""[..]);
        assert!(matches!(result, Err(RcwtError::Eof)));
    }

    #[test]
    fn file_header_rejects_version_2() {
        // Build a v2 header: magic + CC + version(0,2) + format(0,2) + reserved
        let mut buf = Vec::new();
        buf.extend_from_slice(b"\xCC\xCC\xED");
        buf.push(0xCC); // creating program
        buf.extend_from_slice(&80u16.to_be_bytes()); // program version
        buf.extend_from_slice(&2u16.to_be_bytes()); // file format version = 2
        buf.extend_from_slice(&[0, 0, 0]); // reserved
        let result = FileHeader::parse(&mut buf.as_slice());
        assert!(matches!(result, Err(RcwtError::UnsupportedVersion(2))));
    }
}
