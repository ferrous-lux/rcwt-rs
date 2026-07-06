use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process;

use rcwt_rs::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: rcwt-header-free <input.rcwt> <output.rcwt>");
        process::exit(1);
    }

    let input = &args[1];
    let output = &args[2];

    let mut reader: Box<dyn Read> = if *input == "-" {
        Box::new(std::io::stdin().lock())
    } else {
        let file = File::open(input).unwrap_or_else(|e| {
            eprintln!("Error opening {}: {}", input, e);
            process::exit(1);
        });
        Box::new(BufReader::new(file))
    };

    // Validate the FileHeader (parses magic, version, etc.)
    FileHeader::parse(&mut reader).unwrap_or_else(|e| {
        eprintln!("Error reading header from {}: {}", input, e);
        process::exit(1);
    });

    let mut writer: Box<dyn Write> = if *output == "-" {
        Box::new(std::io::stdout().lock())
    } else {
        let file = File::create(output).unwrap_or_else(|e| {
            eprintln!("Error creating {}: {}", output, e);
            process::exit(1);
        });
        Box::new(BufWriter::new(file))
    };

    std::io::copy(&mut reader, &mut writer).unwrap_or_else(|e| {
        eprintln!("Error copying data: {}", e);
        process::exit(1);
    });
}
