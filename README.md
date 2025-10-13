# rcwt-rs

rcwt-rs is a Rust library for reading and writing CCExtractor binary caption files, also known as Raw Captions With Time (RCWT). It provides tools for parsing, serializing, and converting RCWT data to and from structured formats like JSON.

## Features

- Parse RCWT binary files into structured Rust types.
- Serialize RCWT data to JSON format.
- Convert JSON back into RCWT binaries.
- Includes two CLI tools for conversion and testing

CLI Tools:

These binaries are included as examples and utilities:

rcwt2json

Converts an RCWT binary file to JSON.

```bash
rcwt2json <input_bin> <output_json>
```

There currently is no checking if the output exists it will silently overwrite. YOU HAVE BEEN WARNED!

json2rcwt: 

Converts a JSON-formatted file back into RCWT binary format.

```bash
json2rcwt: json2bin <input_json> <output_bin>
```

There currently is no checking if the output exists it will silently overwrite. YOU HAVE BEEN WARNED!

Example JSON Output

{
  "json_schema_version": "0.7.0",
  "creating_program": "CC",
  "file_format_version": 1,
  "magic_number": "CCCCED",
  "program_version": 80,
  "reserved": "000000",
  "cc_records": [
    {
      "index": 1,
      "fts": 100,
      "blocks": 1,
      "rawdata_hex": "FD0185"
    }
  ]
}


## Usage in Your Project

Add this to your `Cargo.toml`:

```toml
[dependencies]
rcwt-rs = "0.1.0"
```

Coming Soon

- XML format support
- Caption decoding and display
- Round-trip validation tools
- Format transcription engine (e.g. XML → JSON → RCWT)

Contributing

Pull requests, issues, and feedback are welcome! This crate is designed for modularity, clarity, and extensibility—perfect for archival tooling, caption analysis, and format conversion.
