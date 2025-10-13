/*

utils.rs: some generic utility functions

*/
// external imports
use std::io::Read;

// internal imports
use crate::RcwtError;

pub fn bytes_to_hex<const N: usize>(bytes: [u8; N]) -> String {
    bytes.iter().map(|b| format!("{:02X}", b)).collect()
}

// pub fn hex_to_bytes<const N: usize>(hex: &str) -> Result<[u8; N], String> {
//     if hex.len() != N * 2 {
//         return Err(format!("Expected {} hex digits, got {}", N * 2, hex.len()));
//     }

//     let mut bytes = [0u8; N];
//     for i in 0..N {
//         let byte_str = &hex[i * 2..i * 2 + 2];
//         bytes[i] = u8::from_str_radix(byte_str, 16)
//             .map_err(|e| format!("Invalid hex at position {}: {}", i, e))?;
//     }
//     Ok(bytes)
// }

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err(format!("Hex string must have even length, got {}", hex.len()));
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at position {}: {}", i, e))
        })
        .collect()
}


pub fn parse_hex<const N: usize>(hex: &str) -> Result<[u8; N], RcwtError> {
    let bytes = hex::decode(hex).map_err(|_| RcwtError::InvalidHeader)?;
    if bytes.len() != N {
        return Err(RcwtError::InvalidHeader);
    }
    let mut array = [0u8; N];
    array.copy_from_slice(&bytes);
    Ok(array)
}
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
