#[macro_use]
extern crate serde_derive;

pub mod entry;
pub mod feed;
pub mod parser;

#[cfg(test)]
mod tests {
    use std::fs::File;

    #[test]
    fn it_works() {
        println!("----------------------- rss1 ---------------------");
        let mut f = File::open("fixture/rss_1.0.xml").unwrap();
        if let Some(feed) = super::parser::parse(&mut f) {
            println!("{:?}", feed);
        }

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
