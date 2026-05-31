# rcwt-rs

A Rust library for reading and writing CCExtractor RCWT (Raw Captions With Time) binary caption files.

Zero dependencies. Streaming by design — modeled after the `tar` crate.

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
```

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
in the future. Known values: `0xCC` for CCExtractor, `0xFF` for FFmpeg.

## Specification

The RCWT binary format specification is in [`docs/rcwt-spec.txt`](docs/rcwt-spec.txt).

## Test Files

Test files in `tests/files/` are included for research and testing purposes only.

## License

MIT
