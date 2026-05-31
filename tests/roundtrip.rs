use std::io::{Cursor, Read, Write};

use rcwt_rs::*;

fn test_file_header() -> FileHeader {
    FileHeader {
        magic_number: [0xCC, 0xCC, 0xED],
        creating_program: 0xCC,
        program_version: 80,
        file_format_version: 1,
        reserved: [0, 0, 0],
    }
}

#[test]
fn builder_stream_roundtrip_single() {
    let header = TimeHeader {
        fts: FTS(100),
        num_blocks: 1,
    };
    let data = vec![0xFD, 0x01, 0x85];

    let mut buf = Vec::new();
    {
        let mut builder = Builder::new(&mut buf, &test_file_header()).unwrap();
        builder.append(&header, &data).unwrap();
    }

    let cursor = Cursor::new(&buf);
    let mut stream = RcwtStream::new(cursor).unwrap();
    let entries: Vec<Entry> = stream.entries().collect::<Result<Vec<_>, _>>().unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].time_header, header);
    assert_eq!(entries[0].data(), &data);
}

#[test]
fn builder_stream_roundtrip_multiple() {
    let records = vec![
        (TimeHeader { fts: FTS(100), num_blocks: 1 }, vec![0xFD, 0x01, 0x85]),
        (
            TimeHeader { fts: FTS(200), num_blocks: 2 },
            vec![0xFD, 0x01, 0x86, 0xFD, 0x01, 0x87],
        ),
        (TimeHeader { fts: FTS(300), num_blocks: 1 }, vec![0xFC, 0x02, 0x80]),
    ];

    let mut buf = Vec::new();
    {
        let mut builder = Builder::new(&mut buf, &test_file_header()).unwrap();
        for (h, d) in &records {
            builder.append(h, d).unwrap();
        }
    }

    let cursor = Cursor::new(&buf);
    let mut stream = RcwtStream::new(cursor).unwrap();
    let entries: Vec<Entry> = stream.entries().collect::<Result<Vec<_>, _>>().unwrap();

    assert_eq!(entries.len(), 3);
    for (i, (exp_h, exp_d)) in records.iter().enumerate() {
        assert_eq!(entries[i].time_header, *exp_h, "entry {i}: time header mismatch");
        assert_eq!(entries[i].data(), exp_d.as_slice(), "entry {i}: data mismatch");
    }
}

#[test]
fn stream_entries_empty_stream() {
    let mut buf = Vec::new();
    Builder::new(&mut buf, &test_file_header()).unwrap();

    let cursor = Cursor::new(&buf);
    let mut stream = RcwtStream::new(cursor).unwrap();
    let entries: Vec<Entry> = stream.entries().collect::<Result<Vec<_>, _>>().unwrap();

    assert!(entries.is_empty());
}

#[test]
fn entry_implements_read_from_stream() {
    let header = TimeHeader {
        fts: FTS(100),
        num_blocks: 1,
    };
    let data = vec![0xFD, 0x01, 0x85];

    let mut buf = Vec::new();
    {
        let mut builder = Builder::new(&mut buf, &test_file_header()).unwrap();
        builder.append(&header, &data).unwrap();
    }

    let cursor = Cursor::new(&buf);
    let mut stream = RcwtStream::new(cursor).unwrap();
    let mut entry = stream.entries().next().unwrap().unwrap();

    let mut read_buf = [0u8; 3];
    entry.read_exact(&mut read_buf).unwrap();
    assert_eq!(&read_buf[..], &data[..]);
}

#[test]
fn entry_writer_roundtrip() {
    let fts = FTS(42);

    let mut buf = Vec::new();
    let entry_data = vec![0xFD, 0x01, 0x85, 0xFD, 0x01, 0x86];
    {
        let mut builder = Builder::new(&mut buf, &test_file_header()).unwrap();
        let mut writer = builder.append_writer(fts);
        writer.write_all(&entry_data).unwrap();
        writer.finish().unwrap();
    }

    let cursor = Cursor::new(&buf);
    let mut stream = RcwtStream::new(cursor).unwrap();
    let entry = stream.entries().next().unwrap().unwrap();

    assert_eq!(entry.time_header.fts, fts);
    assert_eq!(entry.time_header.num_blocks, 2);
    assert_eq!(entry.data(), &entry_data);
}
