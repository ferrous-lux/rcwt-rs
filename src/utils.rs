use std::io::Read;

use crate::RcwtError;

pub fn read_exact_or_eof<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<bool, RcwtError> {
    let mut total = 0;
    while total < buf.len() {
        match reader.read(&mut buf[total..])? {
            0 => return Ok(false), // EOF
            n => total += n,
        }
    }
    Ok(true)
}
