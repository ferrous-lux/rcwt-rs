use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process;

use rcwt_rs::channel::{Channel, ChannelClassifier};
use rcwt_rs::*;

struct Output {
    builder: Builder<BufWriter<File>>,
    count: usize,
}

fn create_output(dir: &Path, stem: &str, suffix: &str, header: &FileHeader) -> Output {
    let mut path = dir.to_path_buf();
    path.push(format!("{}.{}.rcwt", stem, suffix));
    let file = File::create(&path).unwrap_or_else(|e| {
        eprintln!("Error creating {}: {}", path.display(), e);
        process::exit(1);
    });
    let writer = BufWriter::new(file);
    let builder = Builder::new(writer, header).unwrap_or_else(|e| {
        eprintln!("Error writing header to {}: {}", path.display(), e);
        process::exit(1);
    });
    Output { builder, count: 0 }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: rcwt-split <input.rcwt>");
        process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let parent = input_path
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let stem = input_path.file_stem().unwrap().to_str().unwrap();

    let mut stream = RcwtStream::open(&args[1]).unwrap_or_else(|e| {
        eprintln!("Error opening {}: {}", args[1], e);
        process::exit(1);
    });

    let mut header = stream.header.clone();
    header.creating_program = RCWT_CREATING_PROGRAM;
    header.program_version = RCWT_PROGRAM_VERSION;

    let mut outputs: Vec<(&str, Output)> = vec![
        (
            "channel1-field1",
            create_output(&parent, stem, "ch1-f1", &header),
        ),
        (
            "channel2-field1",
            create_output(&parent, stem, "ch2-f1", &header),
        ),
        (
            "channel1-field2",
            create_output(&parent, stem, "ch1-f2", &header),
        ),
        (
            "channel2-field2",
            create_output(&parent, stem, "ch2-f2", &header),
        ),
        ("cea708", create_output(&parent, stem, "cea708", &header)),
    ];

    let mut state = ChannelClassifier::new();
    let mut entry_count: usize = 0;

    for result in stream.entries() {
        let entry = result.unwrap_or_else(|e| {
            eprintln!("Error reading entry {}: {}", entry_count + 1, e);
            process::exit(1);
        });

        let fts = entry.time_header.fts;
        let mut channel_blocks: Vec<Vec<CcBlock>> = (0..5).map(|_| Vec::new()).collect();

        for block in entry.blocks() {
            if let Some(channel) = state.classify(block) {
                let idx = match channel {
                    Channel::Ch1F1 => 0usize,
                    Channel::Ch2F1 => 1,
                    Channel::Ch1F2 => 2,
                    Channel::Ch2F2 => 3,
                    Channel::Cea708 => 4,
                };
                channel_blocks[idx].push(*block);
            }
        }

        for (i, blocks) in channel_blocks.iter().enumerate() {
            if blocks.is_empty() {
                continue;
            }
            let num_blocks = blocks.len() as u16;
            let time_header = TimeHeader { fts, num_blocks };
            let mut data = Vec::with_capacity(blocks.len() * 3);
            for block in blocks {
                data.extend_from_slice(&block.to_bytes());
            }
            let (name, output) = &mut outputs[i];
            output
                .builder
                .append(&time_header, &data)
                .unwrap_or_else(|e| {
                    eprintln!("Error writing to {}: {}", name, e);
                    process::exit(1);
                });
            output.count += 1;
        }

        entry_count += 1;
    }

    println!("Split {} entries:", entry_count);
    for (name, output) in &outputs {
        println!(
            "  {}: {} entries -> {}.{}.rcwt",
            name, output.count, stem, name
        );
    }
}
