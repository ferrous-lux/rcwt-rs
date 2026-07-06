use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::process;

use rcwt_rs::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: rcwt-shift <input.rcwt> <output.rcwt> <delta_ms>");
        process::exit(1);
    }

    let input = &args[1];
    let output = &args[2];

    let delta_ms: i64 = args[3].parse().unwrap_or_else(|_| {
        eprintln!("Error: delta_ms must be an integer");
        process::exit(1);
    });

    let mut stream = if *input == "-" {
        let reader: Box<dyn std::io::Read> = Box::new(std::io::stdin().lock());
        RcwtStream::new(reader).unwrap_or_else(|e| {
            eprintln!("Error reading stdin: {}", e);
            process::exit(1);
        })
    } else {
        let file = File::open(input).unwrap_or_else(|e| {
            eprintln!("Error opening {}: {}", input, e);
            process::exit(1);
        });
        let reader: Box<dyn std::io::Read> = Box::new(BufReader::new(file));
        RcwtStream::new(reader).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", input, e);
            process::exit(1);
        })
    };

    let mut header = stream.header.clone();
    header.creating_program = RCWT_CREATING_PROGRAM;
    header.program_version = RCWT_PROGRAM_VERSION;

    let write_buf = if *output == "-" {
        let w: Box<dyn std::io::Write> = Box::new(std::io::stdout().lock());
        w
    } else {
        let file = File::create(output).unwrap_or_else(|e| {
            eprintln!("Error creating {}: {}", output, e);
            process::exit(1);
        });
        let w: Box<dyn std::io::Write> = Box::new(BufWriter::new(file));
        w
    };

    let mut builder = Builder::new(write_buf, &header).unwrap_or_else(|e| {
        eprintln!("Error writing header: {}", e);
        process::exit(1);
    });

    let mut count: usize = 0;
    for result in stream.entries() {
        let entry = result.unwrap_or_else(|e| {
            eprintln!("Error reading entry: {}", e);
            process::exit(1);
        });
        let new_fts = entry.time_header.fts.shift(delta_ms);
        let shifted_header = TimeHeader {
            fts: new_fts,
            num_blocks: entry.time_header.num_blocks,
        };
        builder
            .append(&shifted_header, entry.data())
            .unwrap_or_else(|e| {
                eprintln!("Error writing entry {}: {}", count + 1, e);
                process::exit(1);
            });
        count += 1;
    }

    eprintln!(
        "Shifted {} entries from {} to {} (delta={})",
        count, input, output, delta_ms
    );
}
