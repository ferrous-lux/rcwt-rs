use std::env;
use std::process;

use rcwt_rs::*;

struct Report {
    path: String,
    header: FileHeader,
    entry_count: usize,
    first_fts: Option<(u64, String)>,
    last_fts: Option<(u64, String)>,
    largest_entry_bytes: usize,
    largest_entry_index: usize,
    total_blocks: usize,
    ch1f1: usize,
    ch2f1: usize,
    ch1f2: usize,
    ch2f2: usize,
    cea708: usize,
    null_padding: usize,
    invalid: usize,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let json_output = args.iter().any(|a| a == "--json");
    let file_args: Vec<&String> = args.iter().filter(|a| *a != "--json").collect();

    if file_args.len() != 2 {
        eprintln!("Usage: rcwt-report [--json] <input.rcwt>");
        process::exit(1);
    }

    let path = file_args[1];
    let mut stream = match RcwtStream::open(path) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error opening {}: {}", path, e);
            process::exit(1);
        }
    };

    let mut report = Report {
        path: path.clone(),
        header: stream.header.clone(),
        entry_count: 0,
        first_fts: None,
        last_fts: None,
        largest_entry_bytes: 0,
        largest_entry_index: 0,
        total_blocks: 0,
        ch1f1: 0,
        ch2f1: 0,
        ch1f2: 0,
        ch2f2: 0,
        cea708: 0,
        null_padding: 0,
        invalid: 0,
    };

    let mut classifier = ChannelClassifier::new();

    for result in stream.entries() {
        let entry = match result {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading entry {}: {}", report.entry_count + 1, e);
                process::exit(1);
            }
        };

        let data_len = entry.data().len();
        if report.first_fts.is_none() {
            report.first_fts = Some((entry.time_header.fts.0, entry.time_header.fts.iso_format()));
        }
        report.last_fts = Some((entry.time_header.fts.0, entry.time_header.fts.iso_format()));

        if data_len > report.largest_entry_bytes {
            report.largest_entry_bytes = data_len;
            report.largest_entry_index = report.entry_count;
        }

        for block in entry.blocks() {
            report.total_blocks += 1;
            if !block.cc_valid {
                report.invalid += 1;
            } else if block.is_null() {
                report.null_padding += 1;
            } else if let Some(channel) = classifier.classify(block) {
                match channel {
                    Channel::Ch1F1 => report.ch1f1 += 1,
                    Channel::Ch2F1 => report.ch2f1 += 1,
                    Channel::Ch1F2 => report.ch1f2 += 1,
                    Channel::Ch2F2 => report.ch2f2 += 1,
                    Channel::Cea708 => report.cea708 += 1,
                }
            }
        }

        report.entry_count += 1;
    }

    if json_output {
        print_json(&report);
    } else {
        print_human(&report);
    }
}

fn print_human(r: &Report) {
    if r.entry_count == 0 {
        println!("No caption records found.");
        return;
    }

    println!("RCWT File Report");
    println!("=================");
    println!();
    println!("File Header:");
    println!("  Magic number:     {:02X?}", r.header.magic_number);
    println!("  Creating program: 0x{:02X}", r.header.creating_program);
    println!("  Program version:  {}", r.header.program_version);
    println!("  Format version:   {}", r.header.file_format_version);
    println!("  Reserved:         {:02X?}", r.header.reserved);
    println!();
    println!("Caption Records: {}", r.entry_count);
    println!();
    if let Some((ms, iso)) = &r.first_fts {
        println!("First entry FTS:   {} ms ({})", ms, iso);
    }
    if let Some((ms, iso)) = &r.last_fts {
        println!("Last entry FTS:    {} ms ({})", ms, iso);
    }
    println!(
        "Largest entry:     {} bytes (entry #{})",
        r.largest_entry_bytes, r.largest_entry_index
    );
    println!();
    println!("CC Block Summary:");
    println!("  Total blocks:    {}", r.total_blocks);
    println!("  Channel 1 F1:    {}", r.ch1f1);
    println!("  Channel 2 F1:    {}", r.ch2f1);
    println!("  Channel 1 F2:    {}", r.ch1f2);
    println!("  Channel 2 F2:    {}", r.ch2f2);
    println!("  CEA-708:         {}", r.cea708);
    println!("  Null padding:    {}", r.null_padding);
    println!("  Invalid:         {}", r.invalid);
}

fn print_json(r: &Report) {
    use std::fmt::Write;

    let magic = r.header.magic_number;
    let path = esc(&r.path);
    let first_ms = r
        .first_fts
        .as_ref()
        .map_or("null".to_string(), |(ms, _)| ms.to_string());
    let first_iso = r
        .first_fts
        .as_ref()
        .map_or("null".to_string(), |(_, iso)| esc(iso));
    let last_ms = r
        .last_fts
        .as_ref()
        .map_or("null".to_string(), |(ms, _)| ms.to_string());
    let last_iso = r
        .last_fts
        .as_ref()
        .map_or("null".to_string(), |(_, iso)| esc(iso));

    let mut buf = String::new();
    write!(
        buf,
        r#"{{"file":"{}","file_header":{{"magic_number":[{},{},{}],"creating_program":{},"program_version":{},"format_version":{},"reserved":[{},{},{}]}},"caption_records":{},"first_fts_ms":{},"first_fts_iso":"{}","last_fts_ms":{},"last_fts_iso":"{}","largest_entry_bytes":{},"largest_entry_index":{},"cc_blocks":{{"total":{},"ch1f1":{},"ch2f1":{},"ch1f2":{},"ch2f2":{},"cea708":{},"null_padding":{},"invalid":{}}}}}"#,
        path,
        magic[0], magic[1], magic[2], r.header.creating_program,
        r.header.program_version, r.header.file_format_version,
        r.header.reserved[0], r.header.reserved[1], r.header.reserved[2],
        r.entry_count, first_ms, first_iso, last_ms, last_iso,
        r.largest_entry_bytes, r.largest_entry_index,
        r.total_blocks, r.ch1f1, r.ch2f1, r.ch1f2, r.ch2f2,
        r.cea708, r.null_padding, r.invalid,
    )
    .ok();
    print!("{}", buf);
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
