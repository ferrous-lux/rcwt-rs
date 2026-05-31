use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process;

use rcwt_rs::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: rcwt-report <input.rcwt>");
        process::exit(1);
    }

    let path = &args[1];
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening {}: {}", path, e);
            process::exit(1);
        }
    };
    let mut reader = BufReader::new(file);

    println!("RCWT File Report");
    println!("=================");
    println!();

    let header = match FileHeader::parse(&mut reader) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error reading file header: {}", e);
            process::exit(1);
        }
    };

    println!("File Header:");
    println!("  Magic number:     {:02X?}", header.magic_number);
    println!("  Creating program: 0x{:02X}", header.creating_program);
    println!("  Program version:  {}", header.program_version);
    println!("  Format version:   {}", header.file_format_version);
    println!("  Reserved:         {:02X?}", header.reserved);
    println!();

    let entries = Entries::new(&mut reader);

    let mut count: usize = 0;
    let mut first_fts: Option<FTS> = None;
    let mut last_fts: Option<FTS> = None;
    let mut biggest_bytes: usize = 0;
    let mut biggest_idx: usize = 0;

    for result in entries {
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
