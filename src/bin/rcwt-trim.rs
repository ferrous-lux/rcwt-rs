use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::process;

use rcwt_rs::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: rcwt-trim <input.rcwt> <output.rcwt> <max_entries>");
        process::exit(1);
    }

    let max: usize = args[3].parse().unwrap_or_else(|_| {
        eprintln!("Error: max_entries must be a number");
        process::exit(1);
    });

    let mut stream = RcwtStream::open(&args[1]).unwrap_or_else(|e| {
        eprintln!("Error opening {}: {}", args[1], e);
        process::exit(1);
    });

    let mut header = stream.header.clone();
    header.creating_program = RCWT_CREATING_PROGRAM;
    header.program_version = RCWT_PROGRAM_VERSION;

    let file = File::create(&args[2]).unwrap_or_else(|e| {
        eprintln!("Error creating {}: {}", args[2], e);
        process::exit(1);
    });
    let writer = BufWriter::new(file);
    let mut builder = Builder::new(writer, &header).unwrap_or_else(|e| {
        eprintln!("Error writing header: {}", e);
        process::exit(1);
    });

    let mut count: usize = 0;
    for result in stream.entries() {
        if count >= max {
            break;
        }
        let entry = result.unwrap_or_else(|e| {
            eprintln!("Error reading entry {}: {}", count + 1, e);
            process::exit(1);
        });
        builder
            .append(&entry.time_header, entry.data())
            .unwrap_or_else(|e| {
                eprintln!("Error writing entry {}: {}", count + 1, e);
                process::exit(1);
            });
        count += 1;
    }

    eprintln!("Copied {} entries from {} to {}", count, args[1], args[2]);
}
