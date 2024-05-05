use std::io::{self, BufRead};

use feed_rs::parser;

// Fetch each URL and try to parse it
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        print!("{}  ... ", line);
        let xml = reqwest::blocking::get(&line)?.bytes()?;

        match parser::parse_with_uri(xml.as_ref(), Some(&line)) {
            Ok(_feed) => println!("ok"),
            Err(error) => println!("failed: {:?}\n{:?}\n-------------------------------------------------------------", error, xml),
        }
    }

    Ok(())
}
