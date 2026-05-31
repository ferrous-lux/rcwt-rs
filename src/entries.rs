use std::io::Read;

use crate::header::TimeHeader;
use crate::RcwtError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CcBlock {
    pub cc_valid: bool,
    pub cc_type: u8,
    pub cc_data: [u8; 2],
}

impl CcBlock {
    pub fn parse(raw: &[u8; 3]) -> Self {
        CcBlock {
            cc_valid: (raw[0] & 0x04) != 0,
            cc_type: raw[0] & 0x03,
            cc_data: [raw[1], raw[2]],
        }
    }

    pub fn is_null(&self) -> bool {
        self.cc_valid && self.cc_type <= 1 && self.cc_data[0] == 0x80 && self.cc_data[1] == 0x80
    }

    pub fn to_bytes(&self) -> [u8; 3] {
        let flags = if self.cc_valid { 0x04u8 } else { 0x00u8 };
        [
            flags | (self.cc_type & 0x03),
            self.cc_data[0],
            self.cc_data[1],
        ]
    }
}

pub struct Entry {
    pub index: usize,
    pub time_header: TimeHeader,
    pub ccblocks: Vec<CcBlock>,
    raw: Vec<u8>,
}

impl Entry {
    pub fn data(&self) -> &[u8] {
        &self.raw
    }

    pub fn blocks(&self) -> &[CcBlock] {
        &self.ccblocks
    }
}

pub struct Entries<'a, R> {
    reader: &'a mut R,
    index: usize,
    finished: bool,
}

impl<'a, R: Read> Entries<'a, R> {
    pub fn new(reader: &'a mut R) -> Self {
        Entries {
            reader,
            index: 1,
            finished: false,
        }
    }

    pub fn into_reader(self) -> &'a mut R {
        self.reader
    }
}

impl<'a, R: Read> Iterator for Entries<'a, R> {
    type Item = Result<Entry, RcwtError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match TimeHeader::parse(self.reader) {
            Ok(time_header) => {
                let num_bytes = time_header.num_blocks as usize * 3;
                let mut data = vec![0u8; num_bytes];
                if let Err(e) = self.reader.read_exact(&mut data) {
                    self.finished = true;
                    return Some(Err(RcwtError::Io(e)));
                }
                let ccblocks = data
                    .chunks_exact(3)
                    .map(|c| CcBlock::parse(&[c[0], c[1], c[2]]))
                    .collect();
                let entry = Entry {
                    index: self.index,
                    time_header,
                    ccblocks,
                    raw: data,
                };
                self.index += 1;
                Some(Ok(entry))
            }
            Err(RcwtError::Eof) => {
                self.finished = true;
                None
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CcBlock, Entry, TimeHeader};
    use crate::FTS;

    fn ccblock(cc_valid: bool, cc_type: u8, cc_data: [u8; 2]) -> CcBlock {
        CcBlock {
            cc_valid,
            cc_type,
            cc_data,
        }
    }

    #[test]
    fn entry_data() {
        let raw = vec![0xFD, 0x01, 0x85];
        let ccblocks = vec![ccblock(true, 1, [0x01, 0x85])];
        let entry = Entry {
            index: 1,
            time_header: TimeHeader {
                fts: FTS(100),
                num_blocks: 1,
            },
            ccblocks,
            raw: raw.clone(),
        };
        assert_eq!(entry.index, 1);
        assert_eq!(entry.time_header.fts, FTS(100));
        assert_eq!(entry.time_header.num_blocks, 1);
        assert_eq!(entry.data(), &raw);
    }

    #[test]
    fn cc_block_parse_field1() {
        let raw = [0x04, 0x94, 0x20];
        let block = CcBlock::parse(&raw);
        assert!(block.cc_valid);
        assert_eq!(block.cc_type, 0);
        assert_eq!(block.cc_data, [0x94, 0x20]);
    }

    #[test]
    fn cc_block_parse_field2() {
        let raw = [0x05, 0x80, 0x80];
        let block = CcBlock::parse(&raw);
        assert!(block.cc_valid);
        assert_eq!(block.cc_type, 1);
        assert_eq!(block.cc_data, [0x80, 0x80]);
    }

    #[test]
    fn cc_block_parse_invalid() {
        let raw = [0x00, 0x94, 0x20];
        let block = CcBlock::parse(&raw);
        assert!(!block.cc_valid);
        assert_eq!(block.cc_type, 0);
    }

    #[test]
    fn cc_block_is_null_padding() {
        let block = CcBlock::parse(&[0x04, 0x80, 0x80]);
        assert!(block.is_null());
    }

    #[test]
    fn cc_block_is_not_null_with_data() {
        let block = CcBlock::parse(&[0x04, 0x94, 0x20]);
        assert!(!block.is_null());
    }

    #[test]
    fn cc_block_708_is_not_null() {
        let block = CcBlock::parse(&[0x06, 0x80, 0x80]);
        assert!(!block.is_null()); // cc_type 2, not 608
    }

    #[test]
    fn cc_block_to_bytes_roundtrip() {
        let cases = [
            (
                [0x04, 0x94, 0x20],
                CcBlock {
                    cc_valid: true,
                    cc_type: 0,
                    cc_data: [0x94, 0x20],
                },
            ),
            (
                [0x05, 0x80, 0x80],
                CcBlock {
                    cc_valid: true,
                    cc_type: 1,
                    cc_data: [0x80, 0x80],
                },
            ),
            (
                [0x06, 0x41, 0x42],
                CcBlock {
                    cc_valid: true,
                    cc_type: 2,
                    cc_data: [0x41, 0x42],
                },
            ),
            (
                [0x07, 0x00, 0x00],
                CcBlock {
                    cc_valid: true,
                    cc_type: 3,
                    cc_data: [0x00, 0x00],
                },
            ),
            (
                [0x00, 0x00, 0x00],
                CcBlock {
                    cc_valid: false,
                    cc_type: 0,
                    cc_data: [0x00, 0x00],
                },
            ),
        ];
        for (expected_bytes, block) in &cases {
            assert_eq!(block.to_bytes(), *expected_bytes);
            assert_eq!(CcBlock::parse(expected_bytes), *block);
        }
    }

    #[test]
    fn entry_blocks() {
        let raw = vec![0x04, 0x94, 0x20, 0x05, 0x80, 0x80];
        let ccblocks = vec![
            ccblock(true, 0, [0x94, 0x20]),
            ccblock(true, 1, [0x80, 0x80]),
        ];
        let entry = Entry {
            index: 1,
            time_header: TimeHeader {
                fts: FTS(100),
                num_blocks: 2,
            },
            ccblocks,
            raw,
        };
        assert_eq!(entry.blocks().len(), 2);
        assert_eq!(entry.blocks()[0], ccblock(true, 0, [0x94, 0x20]));
        assert_eq!(entry.blocks()[1], ccblock(true, 1, [0x80, 0x80]));
    }
}
