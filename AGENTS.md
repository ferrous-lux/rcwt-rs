# AGENTS.md — rcwt-rs

## Build, Lint, & Test Commands

```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Check (fast, no binary output)
cargo check

# Run all tests
cargo test

# Single test (by name filter)
cargo test <test_name>
# e.g. cargo test file_header_roundtrip
# e.g. cargo test roundtrip

# Run only integration tests
cargo test --test roundtrip

# Run only unit tests
cargo test --lib

# Lint (if clippy installed)
cargo clippy --all-targets -- -D warnings

# Format (if rustfmt installed)
cargo fmt --check
```

## Dependencies

- Zero runtime dependencies (no `[dependencies]` in Cargo.toml)

## Module Structure

```
src/
├── lib.rs         # Crate root: re-exports all public API
├── rcwt_stream.rs # RcwtStream<R: Read> — lazy streaming reader (tar-like)
├── builder.rs     # Builder<W: Write> + EntryWriter<'a, W: Write> — streaming writer
├── entries.rs     # Entries<'a, R: Read> (Iterator) + Entry (impl Read)
├── header.rs      # FileHeader + TimeHeader — binary parse/write
├── fts.rs         # FTS(pub u64) — File Timestamp in milliseconds
├── error.rs       # RcwtError — Io, InvalidHeader, UnexpectedEOF, Eof
└── utils.rs       # read_exact_or_eof helper

tests/
├── roundtrip.rs   # Integration tests: Builder → RcwtStream round-trips
└── files/         # Test fixture data
```

## Code Style

### Imports
- stdlib imports first, then blank line, then crate imports
- No wildcard imports (`use foo::*`)
- Group by module, sorted by path depth

### Derives
Data structs derive: `Debug, Clone, PartialEq, Eq`
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
```

### Naming
- **Structs/Enums**: UpperCamelCase
- **Methods/fields**: snake_case
- **Type params**: short uppercase (`R`, `W`, `N`)
- **Binary parsing**: `.parse<R: Read>(reader: &mut R)`, `.write_rcwt<W: Write>(&self, writer: &mut W)`
- **Reader pattern**: `<R: Read>` as generic on the struct (RcwtStream) or method (parse)

### Error Handling
- `RcwtError` enum with `From<io::Error>` impl
- Use `matches!` for checking error variants in control flow
- `RcwtError::Eof` signals clean end-of-stream for the iterator
- Propagate errors with `?`

### Testing
- Unit tests inline: `#[cfg(test)] mod tests` in each source file
- Integration tests in `tests/*.rs`
- Round-trip patterns: write with Builder → read with RcwtStream → assert_eq

### Binaries

```
rcwt-report <input.rcwt>   # Prints file header info, entry count, first/last FTS, largest entry
rcwt-csv <input.rcwt>      # Dumps all entries as CSV: index, fts_ms, fts_iso, size_bytes, data_hex
```

Run with: `cargo run --bin rcwt-report -- <file>`

## API Patterns
```rust
// Reading (from file path)
let mut stream = RcwtStream::open("input.rcwt")?;
println!("{:?}", stream.header.magic_number);

// Reading (from any Read)
let mut stream = RcwtStream::new(reader)?;
for entry in stream.entries() {
    let entry: Entry = entry?;
    let data: &[u8] = entry.data();
}

// Reading without RcwtStream (header parsed separately)
let header = FileHeader::parse(&mut reader)?;
let entries = Entries::new(&mut reader);

// Writing
let mut builder = Builder::new(writer, &file_header)?;
builder.append(&time_header, &data)?;

// Streaming write
let mut writer = builder.append_writer(fts);
writer.write_all(&data)?;
writer.finish()?;
```
