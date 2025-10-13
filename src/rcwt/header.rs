// external imports
use serde::{Serialize, Deserialize};
use std::io;
use std::io::{Read, Write};
use byteorder::ReadBytesExt;

// internal imports
use crate::FTS;
use crate::RcwtError;
use crate::formats::json::{CaptionRecordJson, RcwtJson};
use crate::utils::{parse_hex, read_exact_or_eof};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileHeader {
    pub magic_number: [u8; 3],      // CCCCED
    pub creating_program: u8,       // CC
    pub program_version: u16,       // 0052
    pub file_format_version: u16,   // 0001
    pub reserved: [u8; 3],          // 000000
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
    pub fn from_json(json: &RcwtJson) -> Result<Self, RcwtError> {
        Ok(FileHeader {
            creating_program: parse_hex::<1>(&json.creating_program)?[0],
            file_format_version: json.file_format_version,
            magic_number: parse_hex::<3>(&json.magic_number)?,
            program_version: json.program_version,
            reserved: parse_hex::<3>(&json.reserved)?,
        })
    }
    pub fn write_rcwt<W: Write>(&self, writer: &mut W) -> Result<(), RcwtError> {
        writer.write_all(&self.magic_number)?;                         // [u8; 3]
        writer.write_all(&[self.creating_program])?;                   // u8
        // required output is little-endian
        writer.write_all(&self.program_version.to_be_bytes())?;       // u16 → [u8; 2]
        writer.write_all(&self.file_format_version.to_be_bytes())?;   // u16 → [u8; 2]
        writer.write_all(&self.reserved)?;                             // [u8; 3]
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeHeader {
    pub fts: FTS,
    pub num_blocks: u16,
}

impl TimeHeader {
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self, RcwtError> {

        let mut fts_buf = [0; 8];
        if !read_exact_or_eof(reader, &mut fts_buf)? {
            return Err(RcwtError::Eof.into()); // clean end of stream
        }

        let fts = FTS(u64::from_le_bytes(fts_buf));

        let mut num_blocks_buf = [0; 2];
        reader.read_exact(&mut num_blocks_buf)?;
        let num_blocks = u16::from_le_bytes(num_blocks_buf);

        Ok(TimeHeader { fts, num_blocks })
    }
    pub fn from_json(json: &CaptionRecordJson) -> Self {
        TimeHeader {
            fts: FTS(json.fts),
            num_blocks: json.blocks,
        }
    }
    pub fn write_rcwt<W: Write>(&self, writer: &mut W) -> Result<(), RcwtError> {
        writer.write_all(&self.fts.0.to_le_bytes())?;
        writer.write_all(&self.num_blocks.to_le_bytes())?;
        Ok(())
    }
}
