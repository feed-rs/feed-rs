use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;

// Tool to convert the base fixture data (e.g. XML) into JSON via serde for regression tests
fn main() {
    let mut fixture_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixture_root_dir.push("../feed-rs/fixture");

    find_fixture_files(&fixture_root_dir, |entry| {
        let parser = feed_rs::parser::Builder::new().build();

        let path = entry.path();
        println!("Processing {}", &path.display());

        let data = fs::read(&path).unwrap();
        if let Ok(feed) = parser.parse(data.as_slice()) {
            let json = serde_json::to_string(&feed).unwrap();

            let output_path = path.with_extension("serde.json");
            fs::write(output_path, json).unwrap();
        }
    })
}

fn find_fixture_files(fixture_root: &PathBuf, callback: fn(&DirEntry)) {
    fs::read_dir(fixture_root).unwrap().map(|entry| entry.unwrap()).for_each(|entry| {
        let path = entry.path();
        if path.is_dir() {
            find_fixture_files(&path, callback);
        } else {
            // Ignore any files ending in our serialisation test extension
            if !entry.path().ends_with(".serde.json") {
                callback(&entry);
            }
        }
    });
}
