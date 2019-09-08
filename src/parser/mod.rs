use std::io::Read;

use crate::model::Feed;
use crate::util::attr_value;
use crate::util::element_source::ElementSource;

mod atom;
mod rss0;
mod rss1;
mod rss2;

// TODO improve tests with Coverage analysis e.g. https://github.com/mozilla/grcov

/// Parse the XML input (Atom or a flavour of RSS) into our model
// TODO change this to Result (and the downstream parsers too) to allow error checking upstream
pub fn parse<R: Read>(input: R) -> Option<Feed> {
    // Set up the source of XML elements from the input
    let source = ElementSource::new(input);
    let root = source.root().unwrap();

    // Dispatch to the correct parser
    let version = attr_value(&root.attributes, "version");
    match (root.name.local_name.as_str(), version) {
        ("feed", _) => atom::parse(root),
        ("rss", Some("2.0")) => rss2::parse(root),
        ("rss", Some("0.91")) | ("rss", Some("0.92")) => rss0::parse(root),
        ("RDF", _) => rss1::parse(root),
        _ => None
    }
}
