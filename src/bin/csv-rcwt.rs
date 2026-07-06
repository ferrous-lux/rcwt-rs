use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process;

use rcwt_rs::*;

fn parse_hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) {
        eprintln!("Error: hex data must have even length, got '{}'", s);
        process::exit(1);
    }
    let bytes: Vec<u8> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|e| {
            eprintln!("Error parsing hex data '{}': {}", s, e);
            process::exit(1);
        });
    if !bytes.len().is_multiple_of(3) {
        eprintln!(
            "Error: hex data length ({} bytes) must be a multiple of 3",
            bytes.len()
        );
        process::exit(1);
    }
    bytes
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: csv-rcwt <input.csv> <output.rcwt>");
        process::exit(1);
    }

    let input = &args[1];
    let output = &args[2];

    let reader: Box<dyn BufRead> = if *input == "-" {
        Box::new(std::io::stdin().lock())
    } else {
        let file = File::open(input).unwrap_or_else(|e| {
            eprintln!("Error opening {}: {}", input, e);
            process::exit(1);
        });
        Box::new(BufReader::new(file))
    };

    let write_buf: Box<dyn Write> = if *output == "-" {
        Box::new(std::io::stdout().lock())
    } else {
        let file = File::create(output).unwrap_or_else(|e| {
            eprintln!("Error creating {}: {}", output, e);
            process::exit(1);
        });
        Box::new(BufWriter::new(file))
    };

    let header = FileHeader {
        magic_number: [0xCC, 0xCC, 0xED],
        creating_program: RCWT_CREATING_PROGRAM,
        program_version: RCWT_PROGRAM_VERSION,
        file_format_version: 1,
        reserved: [0, 0, 0],
    };

    let mut builder = Builder::new(write_buf, &header).unwrap_or_else(|e| {
        eprintln!("Error writing header: {}", e);
        process::exit(1);
    });

    let mut count: usize = 0;
    let mut format: Option<&str> = None;

    for line in reader.lines() {
        let line = line.unwrap_or_else(|e| {
            eprintln!("Error reading input: {}", e);
            process::exit(1);
        });

        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();

        // Auto-detect format from the first data line
        if format.is_none() {
            format = match fields.len() {
                2 => Some("minimal"),
                5 => {
                    // Skip header if it looks like the rcwt-csv header
                    if line.starts_with("index,") || line.starts_with("fts_ms,") {
                        continue;
                    }
                    Some("rcwt-csv")
                }
                _ => {
                    eprintln!(
                        "Error: expected 2 or 5 comma-separated fields, got {}",
                        fields.len()
                    );
                    process::exit(1);
                }
            };
        }

        let (fts_ms, hex_data) = match format.unwrap() {
            "minimal" => {
                let fts_ms: u64 = fields[0].trim().parse().unwrap_or_else(|_| {
                    eprintln!("Error: invalid FTS value '{}'", fields[0]);
                    process::exit(1);
                });
                (fts_ms, fields[1])
            }
            "rcwt-csv" => {
                let fts_ms: u64 = fields[1].trim().parse().unwrap_or_else(|_| {
                    eprintln!(
                        "Error: invalid FTS value '{}' on line {}",
                        fields[1],
                        count + 1
                    );
                    process::exit(1);
                });
                (fts_ms, fields[4])
            }
            _ => unreachable!(),
        };

        let data = parse_hex(hex_data);
        let num_blocks = (data.len() / 3) as u16;
        let time_header = TimeHeader {
            fts: FTS(fts_ms),
            num_blocks,
        };

        builder.append(&time_header, &data).unwrap_or_else(|e| {
            eprintln!("Error writing entry {}: {}", count + 1, e);
            process::exit(1);
        });
        count += 1;
    }

    eprintln!("Wrote {} entries to {}", count, output);
}
