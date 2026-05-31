use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::entries::Entries;
use crate::error::RcwtError;
use crate::header::FileHeader;

pub struct RcwtStream<R> {
    pub header: FileHeader,
    inner: R,
}

impl<R: Read> RcwtStream<R> {
    pub fn new(mut reader: R) -> Result<Self, RcwtError> {
        let header = FileHeader::parse(&mut reader)?;
        Ok(RcwtStream {
            header,
            inner: reader,
        })
    }

    pub fn into_inner(self) -> R {
        self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn entries(&mut self) -> Entries<'_, R> {
        Entries::new(&mut self.inner)
    }
}

impl RcwtStream<BufReader<File>> {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RcwtError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let header = FileHeader::parse(&mut reader)?;
        Ok(RcwtStream {
            header,
            inner: reader,
        })
    }
}
