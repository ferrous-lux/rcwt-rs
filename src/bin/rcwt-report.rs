use std::env;
use std::process;

use rcwt_rs::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: rcwt-report <input.rcwt>");
        process::exit(1);
    }

    let mut stream = match RcwtStream::open(&args[1]) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error opening {}: {}", args[1], e);
            process::exit(1);
        }
    };

    println!("RCWT File Report");
    println!("=================");
    println!();
    println!("File Header:");
    println!("  Magic number:     {:02X?}", stream.header.magic_number);
    println!(
        "  Creating program: 0x{:02X}",
        stream.header.creating_program
    );
    println!("  Program version:  {}", stream.header.program_version);
    println!("  Format version:   {}", stream.header.file_format_version);
    println!("  Reserved:         {:02X?}", stream.header.reserved);
    println!();

    let mut count: usize = 0;
    let mut first_fts: Option<FTS> = None;
    let mut last_fts: Option<FTS> = None;
    let mut biggest_bytes: usize = 0;
    let mut biggest_idx: usize = 0;

    for result in stream.entries() {
        let entry = match result {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading entry {}: {}", count + 1, e);
                process::exit(1);
            }
        };

        let data_len = entry.data().len();
        if first_fts.is_none() {
            first_fts = Some(entry.time_header.fts);
        }
        last_fts = Some(entry.time_header.fts);

        if data_len > biggest_bytes {
            biggest_bytes = data_len;
            biggest_idx = count;
        }

        count += 1;
    }

    if count == 0 {
        println!("No caption records found.");
    } else {
        println!("Caption Records: {}", count);
        println!();
        if let Some(fts) = first_fts {
            println!("First entry FTS:   {} ms ({})", fts.0, fts.iso_format());
        }
        if let Some(fts) = last_fts {
            println!("Last entry FTS:    {} ms ({})", fts.0, fts.iso_format());
        }
        println!(
            "Largest entry:     {} bytes (entry #{})",
            biggest_bytes, biggest_idx
        );
    }
}
