use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

/// Return the path to our test data directory
pub fn fixture_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("fixture");
    dir
}

/// Return the path to a file within the test data directory
pub fn fixture_filename(filename: &str) -> String {
    let mut dir = fixture_dir();
    dir.push(filename);
    dir.to_str().unwrap().to_owned()
}

/// Load a test file and return it as a String
pub fn fixture_as_string(resource: &str) -> String {
    let path = fixture_filename(resource);
    fs::read_to_string(path).unwrap()
}
