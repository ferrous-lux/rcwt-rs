// external imports
use serde::{Deserialize, Serialize};
use std::io;
use std::io::{Read, Write};

// internal imports
use crate::RcwtError;
use crate::formats::json::CaptionRecordJson;
use crate::rcwt::header::TimeHeader;
use crate::utils::hex_to_bytes;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CaptionPacket {
    Raw(RawCaptionPacket),
    // Decoded(DecodedCaptionPacket), // Not yet implemented
}

impl CaptionPacket {
    pub fn write_rcwt<W: Write>(&self, writer: &mut W) -> Result<(), RcwtError> {
        match self {
            CaptionPacket::Raw(packet) => packet.write_rcwt(writer),
            // Add other variants here if needed
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawCaptionPacket {
	pub raw_cc_packet: Vec<u8>, // Raw binary caption data in triplets
}

impl RawCaptionPacket {
    pub fn read<R: Read>(reader: &mut R, num_blocks: usize) -> io::Result<Self> {
        let mut cc_buf = vec![0u8; num_blocks * 3];
        reader.read_exact(&mut cc_buf)?;
        Ok(Self {
            raw_cc_packet: cc_buf,
        })
    }
    pub fn write_rcwt<W: Write>(&self, writer: &mut W) -> Result<(), RcwtError> {
        writer.write_all(&self.raw_cc_packet)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaptionRecord {
    pub index: usize,
    pub time_header: TimeHeader,
    pub cc_packet: CaptionPacket,
}

impl CaptionRecord {
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self, RcwtError> {
        let time_header = TimeHeader::parse(reader)?;
        let raw = RawCaptionPacket::read(reader, time_header.num_blocks.into())?;
        let cc_packet = CaptionPacket::Raw(raw);

        Ok(CaptionRecord {
            index: 0, // Temporary placeholder
            time_header,
            cc_packet,
        })
    }
    pub fn parse_with_index<R: Read>(reader: &mut R, index: usize) -> Result<Self, RcwtError> {
        let mut record = Self::parse(reader)?;
        record.index = index;
        Ok(record)
    }
    pub fn from_json(json: &CaptionRecordJson) -> Result<Self, RcwtError> {
        let raw_bytes = hex_to_bytes(&json.rawdata_hex)
        .map_err(|_| RcwtError::InvalidHeader)?;
        Ok(CaptionRecord {
            index: json.index,
            time_header: TimeHeader::from_json(json),
            cc_packet: CaptionPacket::Raw(RawCaptionPacket {
                raw_cc_packet: raw_bytes.to_vec(),
            }),
        })
    }
    pub fn write_rcwt<W: Write>(&self, writer: &mut W) -> Result<(), RcwtError> {
        self.time_header.write_rcwt(writer)?;
        self.cc_packet.write_rcwt(writer)?;
        Ok(())
    }
}