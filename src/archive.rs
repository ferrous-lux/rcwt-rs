use std::io::Read;

use crate::entries::Entries;
use crate::error::RcwtError;
use crate::header::FileHeader;

pub struct Archive<R> {
    inner: R,
    header: Option<FileHeader>,
}

impl<R: Read> Archive<R> {
    pub fn new(reader: R) -> Self {
        Archive {
            inner: reader,
            header: None,
        }
    }

    pub fn into_inner(self) -> R {
        self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn header(&self) -> Option<&FileHeader> {
        self.header.as_ref()
    }

    pub fn entries(&mut self) -> Result<Entries<'_, R>, RcwtError> {
        if self.header.is_none() {
            let h = FileHeader::parse(&mut self.inner)?;
            self.header = Some(h);
        }
        Ok(Entries::new(&mut self.inner))
    }
}
