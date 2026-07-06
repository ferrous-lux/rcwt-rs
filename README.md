# rcwt-rs

A Rust library for reading and writing CCExtractor RCWT (Raw Captions With Time) binary caption files.

Only file format version 1 is supported at this time.

Zero dependencies. Streaming by design - modeled after the `tar` crate.

## Features

- Parse RCWT binary files into a lazy streaming iterator of entries
- Write RCWT binary files via a streaming builder
- Entry data accessible via `Read` impl
- No in-memory buffering of the full file/stream
- CLI tools for inspection

## CLI Tools

```bash
# Print a summary of an RCWT file
cargo run --bin rcwt-report -- input.rcwt

# Dump all entries as CSV
cargo run --bin rcwt-csv -- input.rcwt

# Split RCWT file into separate channels
cargo run --bin rcwt-split -- input.rcwt

# Trim RCWT to first <max_entries> CC entries
cargo run --bin rcwt-trim -- input.rcwt output.rcwt <max_entries>

# Crop RCWT to entries within an FTS range
cargo run --bin rcwt-crop -- input.rcwt output.rcwt <start_fts> <end_fts>

# Shift all FTS values by a delta (positive or negative)
cargo run --bin rcwt-shift -- input.rcwt output.rcwt <delta_ms>

# Concatenate multiple RCWT files with FTS overlap check
cargo run --bin rcwt-concat -- <input1.rcwt> [<input2.rcwt> ...] <output.rcwt>

# Strip FileHeader from an RCWT file (raw entry stream to stdout or file)
cargo run --bin rcwt-header-free -- <input.rcwt> <output.rcwt>

# Prepend a default FileHeader to a raw entry stream
cargo run --bin rcwt-add-header -- <input.raw> <output.rcwt>

# Build an RCWT file from CSV (fts_ms,data_hex or rcwt-csv format)
cargo run --bin csv-rcwt -- <input.csv> <output.rcwt>
```

### csv-rcwt Input Formats

`csv-rcwt` accepts two comma-separated formats and auto-detects which one is being used:

**Minimal (2 columns):**
```csv
fts_ms,data_hex
100,FD0185
200,FD0186FD0187
```

**rcwt-csv output (5 columns):**
```csv
index,fts_ms,fts_iso,size_bytes,data_hex
1,100,00:00:00.100,3,FD0185
2,200,00:00:00.200,6,FD0186FD0187
```

The header row (`index,fts_ms,...`) is detected and skipped automatically. Lines with 2 fields are parsed as `fts_ms,data_hex`. Lines with 5 fields use the same columns as `rcwt-csv` output. The hex data length must be a multiple of 3 (each CC block is 3 bytes).

## Usage in Your Project

Add this to your `Cargo.toml`:

```toml
[dependencies]
rcwt-rs = "0.1.0"
```

### Reading

```rust
use std::fs::File;
use std::io::BufReader;
use rcwt_rs::*;

let file = BufReader::new(File::open("input.rcwt")?);
let mut stream = RcwtStream::new(file)?;
println!("{:?}", stream.header.magic_number);

for entry in stream.entries() {
    let entry: Entry = entry?;
    println!("fts={} data={:02X?}", entry.time_header.fts.0, entry.data());
}
```

### Writing

```rust
use rcwt_rs::*;

let header = FileHeader {
    magic_number: [0xCC, 0xCC, 0xED],
    creating_program: 0xCC,   // 0xCC = CCExtractor, 0xFF = FFmpeg
    program_version: 1,       // set your own program version
    file_format_version: 1,
    reserved: [0, 0, 0],
};

let mut buf = Vec::new();
let mut builder = Builder::new(&mut buf, &header)?;
builder.append(&TimeHeader { fts: FTS(100), num_blocks: 1 }, &[0xFD, 0x01, 0x85])?;
```

When writing RCWT files, set `creating_program` and `program_version` to identify
your program. This helps track which software produced the file if there are bugs
in the future. Known values: `0xCC` for CCExtractor, `0xFF` for FFmpeg. The included
rcwt-split, rcwt-trim, rcwt-crop, rcwt-shift, rcwt-concat, rcwt-header-free, rcwt-add-header, and csv-rcwt set `creating_program` to ASCII 'r' (0x72) and 
`program_version` to 1.

## Specification

The RCWT binary format specification is in [`docs/rcwt-spec.txt`](docs/rcwt-spec.txt).

## Test Files

Test files in `tests/files/` are included for research and testing purposes only.

## License

MIT
