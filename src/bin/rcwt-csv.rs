use std::env;
use std::process;

use rcwt_rs::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: rcwt-csv <input.rcwt>");
        process::exit(1);
    }

    let mut stream = match RcwtStream::open(&args[1]) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error opening {}: {}", args[1], e);
            process::exit(1);
        }
    };

    println!("index,fts_ms,fts_iso,size_bytes,data_hex");
    for result in stream.entries() {
        let entry = match result {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading entry: {}", e);
                process::exit(1);
            }
        };
        let hex: String = entry.data().iter().map(|b| format!("{:02X}", b)).collect();
        println!(
            "{},{},{},{},{}",
            entry.index,
            entry.time_header.fts.0,
            entry.time_header.fts.iso_format(),
            entry.data().len(),
            hex,
        );
    }
}
