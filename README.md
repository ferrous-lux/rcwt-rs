# rcwt-rs

A Rust library for reading and writing CCExtractor RCWT (Raw Captions With Time) binary caption files.

Zero dependencies. Streaming by design — modeled after the `tar` crate.

## Features

- Parse RCWT binary files into a lazy streaming iterator of entries
- Write RCWT binary files via a streaming builder
- Entry data accessible via `Read` impl
- No in-memory buffering of the full archive
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
let mut archive = Archive::new(file)?;
println!("{:?}", archive.header.magic_number);

for entry in archive.entries() {
    let entry: Entry = entry?;
    println!("fts={} data={:02X?}", entry.time_header.fts.0, entry.data());
}
```

### Writing

```rust
use rcwt_rs::*;

let header = FileHeader {
    magic_number: [b'C', b'C', b'C'],
    creating_program: 0xCC,
    program_version: 80,
    file_format_version: 1,
    reserved: [0, 0, 0],
};

let mut buf = Vec::new();
let mut builder = Builder::new(&mut buf, &header)?;
builder.append(&TimeHeader { fts: FTS(100), num_blocks: 1 }, &[0xFD, 0x01, 0x85])?;
```

## Test Files

Test files in `tests/files/` are included for research and testing purposes only.

## License

MIT
