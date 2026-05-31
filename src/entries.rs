use std::io::{Cursor, Read};

use crate::header::TimeHeader;
use crate::RcwtError;

pub struct Entry {
    pub index: usize,
    pub time_header: TimeHeader,
    cursor: Cursor<Vec<u8>>,
}

impl Entry {
    pub fn data(&self) -> &[u8] {
        self.cursor.get_ref()
    }
}

impl Read for Entry {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(buf)
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
                let entry = Entry {
                    index: self.index,
                    time_header,
                    cursor: Cursor::new(data),
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
