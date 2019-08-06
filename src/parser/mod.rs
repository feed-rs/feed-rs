use std::io::Read;

use crate::model::Feed;
use crate::util::element_source::ElementSource;
use crate::util::attr_value;

mod atom;
mod rss2;

/// Parse the XML input (Atom or a flavour of RSS) into our model
pub fn parse<R: Read>(input: R) -> Option<Feed> {
    // Set up the source of XML elements from the input
    let source = ElementSource::new(input);
    let root = source.root().unwrap();

    // Dispatch to the correct parser
    let version = attr_value(&root.attributes, "version");
    match (root.name.local_name.as_str(), version) {
        ("feed", _) => atom::parse(root),
        ("rss", Some("2.0")) => rss2::parse(root),
        _ => None
    }
}
