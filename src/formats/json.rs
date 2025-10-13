// external imports
use serde::{Deserialize, Serialize};

// internal imports
use crate::rcwt::record::CaptionRecord;
use crate::rcwt::stream::CcStream;
use crate::rcwt::header::FileHeader;
use crate::error::RcwtError;
use crate::utils::bytes_to_hex;
use crate::rcwt::record::CaptionPacket;

#[derive(Debug, Deserialize, Serialize)]
pub struct CaptionRecordJson {
    pub index: usize,
    pub fts: u64,
    pub blocks: u16,
    pub rawdata_hex: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RcwtJson {
    pub json_schema_version: String,
    pub creating_program: String,
    pub file_format_version: u16,
    pub magic_number: String,
    pub program_version: u16,
    pub reserved: String,
    pub cc_records: Vec<CaptionRecordJson>,
}

impl RcwtJson {
    pub fn from_cc_stream(stream: &CcStream) -> Self {
        RcwtJson {
            json_schema_version: "0.7.0".to_string(), // ugh hard-coded version
            creating_program: format!("{:02X}", stream.file_header.creating_program),
            file_format_version: stream.file_header.file_format_version,
            magic_number: bytes_to_hex(stream.file_header.magic_number),
            program_version: stream.file_header.program_version,
            reserved: bytes_to_hex(stream.file_header.reserved),
            cc_records: stream.records.iter().map(|r| {
                let raw_bytes = match &r.cc_packet {
                    CaptionPacket::Raw(raw) => &raw.raw_cc_packet,
                };
                CaptionRecordJson {
                    index: r.index,
                    fts: r.time_header.fts.0,
                    blocks: r.time_header.num_blocks,
                    rawdata_hex: hex::encode_upper(raw_bytes),
                }
            }).collect(),
        }
    }
    pub fn from_json(&self) -> Result<CcStream, RcwtError> {
        let file_header = FileHeader::from_json(self)?;
        let records = self.cc_records
            .iter()
            .map(CaptionRecord::from_json)
            .collect::<Result<Vec<_>, RcwtError>>()?;
        Ok(CcStream { file_header, records })
    }
}