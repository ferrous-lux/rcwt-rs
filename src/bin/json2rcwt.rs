use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use rcwt_rs::formats::json::RcwtJson;
use rcwt_rs::error::RcwtError;

fn main() -> Result<(), RcwtError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: json2rcwt <input.json> <output.rcwt>");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    // Read and parse JSON
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let rcwt_json: RcwtJson = serde_json::from_reader(reader)?;

    // Convert to in-memory RCWT representation
    let cc_stream = rcwt_json.from_json()?;

    // Write to RCWT binary
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    cc_stream.write_rcwt(&mut writer)?;

    println!("Successfully wrote RCWT to {}", output_path);
    Ok(())
}


