// external imports
use serde::{Serialize, Deserialize};
use std::io::{Read, Write};

// internal imports
use crate::RcwtError;
use crate::rcwt::header::FileHeader;
use crate::rcwt::record::CaptionRecord;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CcStream {
    pub file_header: FileHeader,
    pub records: Vec<CaptionRecord>,
}

impl CcStream {
    pub fn write_rcwt<W: Write>(&self, writer: &mut W) -> Result<(), RcwtError> {
        self.file_header.write_rcwt(writer)?;
        for record in &self.records {
            record.write_rcwt(writer)?;
        }
        Ok(())
    }
}

pub fn parse_rcwt_stream<R: Read>(reader: &mut R) -> Result<CcStream, RcwtError> {
    let file_header = FileHeader::parse(reader)?;

    let mut cc_stream = CcStream {
        file_header,
        records: Vec::new(),
    };

    let mut index = 1;

    loop {
        match CaptionRecord::parse_with_index(reader, index) {
            Ok(record) => {
                cc_stream.records.push(record);
                index += 1;
            }
            Err(e) if matches!(e, RcwtError::Eof) => {
                println!("Reached EOF cleanly.");
                break;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(cc_stream)
}