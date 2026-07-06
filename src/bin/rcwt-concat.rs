use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::process;

use rcwt_rs::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: rcwt-concat <input1.rcwt> [<input2.rcwt> ...] <output.rcwt>");
        process::exit(1);
    }

    let output_path = args.last().unwrap();
    let input_count = args.len() - 2;

    // Open and stamp the first input's header, then build the output writer
    let first_input = &args[1];
    let first_file = File::open(first_input).unwrap_or_else(|e| {
        eprintln!("Error opening {}: {}", first_input, e);
        process::exit(1);
    });
    let first_reader = BufReader::new(first_file);
    let mut first_stream = RcwtStream::new(first_reader).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", first_input, e);
        process::exit(1);
    });

    let mut header = first_stream.header.clone();
    header.creating_program = RCWT_CREATING_PROGRAM;
    header.program_version = RCWT_PROGRAM_VERSION;

    let write_buf = if output_path == "-" {
        let w: Box<dyn std::io::Write> = Box::new(std::io::stdout().lock());
        w
    } else {
        let file = File::create(output_path).unwrap_or_else(|e| {
            eprintln!("Error creating {}: {}", output_path, e);
            process::exit(1);
        });
        let w: Box<dyn std::io::Write> = Box::new(BufWriter::new(file));
        w
    };

    let mut builder = Builder::new(write_buf, &header).unwrap_or_else(|e| {
        eprintln!("Error writing header: {}", e);
        process::exit(1);
    });

    let mut last_fts: Option<FTS> = None;
    let mut count: usize = 0;

    // Write all entries from the first input
    for result in first_stream.entries() {
        let entry = result.unwrap_or_else(|e| {
            eprintln!("Error reading entry from {}: {}", first_input, e);
            process::exit(1);
        });
        builder
            .append(&entry.time_header, entry.data())
            .unwrap_or_else(|e| {
                eprintln!("Error writing entry from {}: {}", first_input, e);
                process::exit(1);
            });
        count += 1;
        last_fts = Some(entry.time_header.fts);
    }

    // Process remaining inputs (their FileHeaders are parsed and discarded)
    for input_path in args[2..=input_count].iter() {
        let file = File::open(input_path).unwrap_or_else(|e| {
            eprintln!("Error opening {}: {}", input_path, e);
            process::exit(1);
        });
        let reader = BufReader::new(file);
        let mut stream = RcwtStream::new(reader).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", input_path, e);
            process::exit(1);
        });

        let mut entries = stream.entries();

        // Peek at first entry for overlap check
        if let Some(Ok(entry)) = entries.next() {
            if let Some(lf) = last_fts {
                if entry.time_header.fts < lf {
                    eprintln!(
                        "Error: {} starts at FTS {} which is before last output FTS {}",
                        input_path, entry.time_header.fts.0, lf.0,
                    );
                    process::exit(1);
                }
            }

            builder
                .append(&entry.time_header, entry.data())
                .unwrap_or_else(|e| {
                    eprintln!("Error writing entry from {}: {}", input_path, e);
                    process::exit(1);
                });
            count += 1;
            last_fts = Some(entry.time_header.fts);

            // Write remaining entries from this input
            for result in entries {
                let entry = result.unwrap_or_else(|e| {
                    eprintln!("Error reading entry from {}: {}", input_path, e);
                    process::exit(1);
                });
                builder
                    .append(&entry.time_header, entry.data())
                    .unwrap_or_else(|e| {
                        eprintln!("Error writing entry from {}: {}", input_path, e);
                        process::exit(1);
                    });
                count += 1;
                last_fts = Some(entry.time_header.fts);
            }
        }
    }

    eprintln!(
        "Concatenated {} entries from {} input(s) to {}",
        count, input_count, output_path,
    );
}
