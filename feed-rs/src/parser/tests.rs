use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::model::Feed;
use crate::parser;
use crate::util::test;

// Regression test for the default ID generator
#[test]
fn id_generator_default() {
    let test_data = test::fixture_as_raw("rss2/rss_2.0_kdist.xml");
    let feed = parser::parse(test_data.as_slice()).unwrap();
    assert_eq!("354331764be7571efc15c7a1bad13d54", feed.id);
}

// Verifies failure uncovered by fuzzing is now fixed
#[test]
fn fuzz_parse() {
    let data: Vec<u8> = vec![
        0xdb, 0x3b, 0x3c, 0x66, 0x65, 0x65, 0x64, 0x3e, 0x00, 0xfe, 0xff, 0x00, 0x00, 0x00, 0x3c, 0x1b, 0x3b, 0x64, 0x22, 0x22, 0x0d, 0x78, 0x6d, 0x6c, 0x3a,
        0x62, 0x61, 0x73, 0x65, 0x0d, 0x0d, 0x3d, 0x0a, 0x22, 0x0a, 0x0d, 0x0a, 0x0a, 0x0d, 0x66, 0x69, 0x6c, 0x65, 0x3a, 0xff, 0x3b, 0xbf, 0x5b, 0xbf, 0xbf,
        0xbc, 0xff, 0xff, 0x0a, 0x53, 0x53, 0x2b, 0x78, 0x3b, 0x22, 0x3c, 0x64, 0x3e, 0x2b, 0x00, 0x00, 0x2b, 0x3c, 0xdb, 0x3b, 0x32, 0x65, 0x64, 0x22, 0x22,
        0x0d, 0x78, 0x6d, 0x6c, 0x3a, 0x62, 0x61, 0x73, 0x65, 0x0d, 0x0d, 0x3d, 0x22, 0x75, 0x7c, 0x3f, 0x0a, 0x34, 0x0a, 0xff, 0x22, 0x34, 0x3a, 0xb5, 0x2f,
        0x3c, 0x66, 0x65, 0x64, 0x3e, 0x2b, 0x3c, 0xdb, 0x3b, 0x32, 0x65, 0x0d, 0x78, 0x6d, 0x6c, 0x3a, 0x62, 0x61, 0x73, 0x65, 0x0d, 0x0d, 0x3d, 0x22, 0x2e,
        0x2e, 0x3f, 0x0a, 0x3c, 0x3f, 0xff, 0x22, 0x34, 0x3a, 0xb5, 0x2f, 0x2f, 0xff, 0xff, 0xfe, 0x01, 0xdb, 0x3b, 0x3c, 0x66, 0x65,
    ];

    let result = parser::parse(data.as_slice());
    assert!(result.is_err());
}

// Verifies that a round-trip through the parser + serde works correctly over time
#[test]
fn serde_regression() {
    let fixture_root_dir = test::fixture_dir();
    find_fixture_files(&fixture_root_dir, |source_path, json_path| {
        // Parse the original fixture file
        let data = fs::read(&source_path).unwrap();
        let parser = parser::Builder::default().sanitize_content(false).build();
        let mut feed = parser.parse(data.as_slice()).unwrap();

        // Parse the previously serialised form
        let serde_data = fs::read(&json_path).unwrap();
        let mut serde_feed = serde_json::from_slice(&serde_data).unwrap();

        // Basic check, and then try with replaced IDs too
        if feed != serde_feed {
            // Replace the IDs in the serialised form if UUIDs
            replace_ids(&mut feed, &mut serde_feed);

            assert_eq!(feed, serde_feed);
        }
    });
}

fn find_fixture_files(fixture_root: &PathBuf, callback: fn(&Path, &Path)) {
    fs::read_dir(fixture_root).unwrap().map(|entry| entry.unwrap()).for_each(|entry| {
        let source_path = entry.path();
        if source_path.is_dir() {
            find_fixture_files(&source_path, callback);
        } else {
            // Process the xml + json base files
            let path_str = source_path.to_str().unwrap();
            if path_str.ends_with(".xml") || path_str.ends_with(".json") {
                // Ignore if no serde companion file
                let json_path = source_path.with_extension("serde.json");
                if json_path.exists() {
                    callback(&source_path, &json_path);
                }
            }
        }
    });
}

fn replace_ids(expected: &mut Feed, actual: &mut Feed) {
    if Uuid::parse_str(&expected.id).is_ok() {
        actual.id = expected.id.clone();
    }

    for (expected_entry, actual_entry) in expected.entries.iter().zip(actual.entries.iter_mut()) {
        if Uuid::parse_str(&expected_entry.id).is_ok() {
            actual_entry.id = expected_entry.id.clone();
        }
    }
}
