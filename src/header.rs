use std::io;
use std::io::{Read, Write};
use byteorder::ReadBytesExt;

use crate::FTS;
use crate::RcwtError;
use crate::utils::read_exact_or_eof;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHeader {
    pub magic_number: [u8; 3],
    pub creating_program: u8,
    pub program_version: u16,
    pub file_format_version: u16,
    pub reserved: [u8; 3],
}

impl FileHeader {
    pub fn parse<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut magic_number = [0; 3];
        reader.read_exact(&mut magic_number)?;
        let creating_program = reader.read_u8()?;

        let mut buf = [0; 2];
        reader.read_exact(&mut buf)?;
        let program_version = u16::from_be_bytes(buf);
        reader.read_exact(&mut buf)?;
        let file_format_version = u16::from_be_bytes(buf);

        let mut reserved = [0; 3];
        reader.read_exact(&mut reserved)?;

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
