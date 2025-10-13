use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader};
//use serde::Serialize;


use rcwt_rs::error::RcwtError;
use rcwt_rs::formats::json::RcwtJson;
//use rcwt_rs::rcwt::record::CaptionRecord;
//use rcwt_rs::rcwt::header::FileHeader;
use rcwt_rs::rcwt::stream::parse_rcwt_stream;
//use rcwt_rs::utils::{bytes_to_hex, hex_to_bytes};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: rcwt2json <input.json> <output.rcwt>");
        std::process::exit(1);
    }

    let input_binary_file = &args[1];
    let output_path = &args[2];

    let file = File::open(input_binary_file)?;
    let mut reader = BufReader::new(file);

    println!("Starting parsing");

    let stream = match parse_rcwt_stream(&mut reader) {
        Ok(s) => s,
        Err(RcwtError::Eof) => {
            eprintln!("Reached EOF cleanly. No more caption records.");
            return Ok(());
        }
        Err(e) => {
            eprintln!("Error during parsing: {:?}", e);
            return Err(e.into());
        }
    };

    println!("Finished parsing, trying to make JSON");
    let json_output = RcwtJson::from_cc_stream(&stream);

    // let mut output_json_file = output_path;

    let mut file = File::create(output_path)?;
    serde_json::to_writer_pretty(&mut file, &json_output)?;

    Ok(())
}
