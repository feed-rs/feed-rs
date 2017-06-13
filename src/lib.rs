#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;
extern crate xml5ever;
extern crate uuid;

pub mod entry;
pub mod feed;
pub mod parser;

pub use self::entry::Entry;
pub use self::feed::Feed;

#[cfg(test)]
mod tests {
    use std::fs::File;

    #[test]
    fn it_works() {
        println!("----------------------- rss2 ---------------------");
        let mut f = File::open("fixture/rss_2.0.xml").unwrap();
        if let Some(feed) = super::parser::parse(&mut f) {
            println!("{:?}", feed);
        }

        println!("----------------------- atom ---------------------");
        let mut f = File::open("fixture/atom.xml").unwrap();
        if let Some(feed) = super::parser::parse(&mut f) {
            println!("{:?}", feed);
        }
    }
}
