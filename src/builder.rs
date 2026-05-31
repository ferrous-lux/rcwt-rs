use std::io::Write;

use crate::error::RcwtError;
use crate::header::{FileHeader, TimeHeader};
use crate::FTS;

pub struct Builder<W: Write> {
    inner: Option<W>,
    header_written: bool,
}

impl<W: Write> Builder<W> {
    pub fn new(writer: W, file_header: &FileHeader) -> Result<Self, RcwtError> {
        let mut builder = Builder {
            inner: Some(writer),
            header_written: false,
        };
        if let Some(ref mut w) = builder.inner {
            file_header.write_rcwt(w)?;
        }
        builder.header_written = true;
        Ok(builder)
    }

    pub fn into_inner(mut self) -> Result<W, RcwtError> {
        self.finish()?;
        Ok(self.inner.take().unwrap())
    }

    pub fn get_ref(&self) -> &W {
        self.inner.as_ref().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut W {
        self.inner.as_mut().unwrap()
    }

    fn inner(&mut self) -> &mut W {
        self.inner.as_mut().unwrap()
    }

    pub fn append(&mut self, header: &TimeHeader, data: &[u8]) -> Result<(), RcwtError> {
        if !self.header_written {
            return Err(RcwtError::InvalidHeader);
        }
        header.write_rcwt(self.inner())?;
        self.inner().write_all(data)?;
        Ok(())
    }

    pub fn append_writer(&mut self, fts: FTS) -> EntryWriter<'_, W> {
        EntryWriter {
            inner: self.inner(),
            buf: Vec::new(),
            fts,
        }
    }

    pub fn finish(&mut self) -> Result<(), RcwtError> {
        Ok(())
    }
}

impl<W: Write> Drop for Builder<W> {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

pub struct EntryWriter<'a, W: Write> {
    inner: &'a mut W,
    buf: Vec<u8>,
    fts: FTS,
}

impl<'a, W: Write> EntryWriter<'a, W> {
    pub fn finish(mut self) -> Result<(), RcwtError> {
        let data = std::mem::take(&mut self.buf);
        let num_blocks = (data.len() / 3) as u16;
        let header = TimeHeader {
            fts: self.fts,
            num_blocks,
        };
        header.write_rcwt(self.inner)?;
        self.inner.write_all(&data)?;
        Ok(())
    }
}

impl<'a, W: Write> Write for EntryWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
