use std::io::Read;

use crate::model::Feed;
use crate::util::element_source::ElementSource;
use crate::util::attr_value;

mod atom;

/// Parse the XML input (Atom or a flavour of RSS) into our model
pub fn parse<R: Read>(input: R) -> Option<Feed> {
    // Set up the source of XML elements from the input
    let source = ElementSource::new(input);
    let root = source.root().unwrap();

    // Dispatch to the correct parser
    let version = attr_value(&root.attributes, "version");
    match (root.name.local_name.as_str(), version) {
        ("feed", _) => atom::parse(root),
        _ => None
    }
}
/*

fn walk(handle: Handle) -> Option<Feed> {
    let node = handle;
    match node.data {
        NodeData::Document => (),
        NodeData::Element { ref name, ref attrs, .. } => {
            let tag_name = name.local.as_ref();
            let version  = attr("version", &attrs.borrow()).unwrap_or("".to_string());
            match (tag_name, version.as_ref()) {
                ("feed", _)    => return atom::handle_atom(node.clone()),
                ("rss", "2.0") => return rss2::handle_rss2(node.clone()),
                ("RDF", _)     => return rss1::handle_rss1(node.clone()),
                _ => (),
            }
        },
        _ => {},
    }
    for child in node.children.borrow().iter() {
        if let Some(feed) = walk(child.clone()) {
            return Some(feed)
        }
    }
    None
}

pub fn attr(attr_name: &str, attrs: &Vec<Attribute>) -> Option<String> {
    for attr in attrs.iter() {
        if attr.name.local.as_ref() == attr_name {
            return Some(attr.value.to_string())
        }
    }
    None
}

pub fn text(handle: Handle) -> Option<String> {
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Text { ref contents } =>
                return Some(contents.borrow().to_string()),
            _ => (),
        }
    }
    return None
}

pub fn timestamp_from_rfc3339(handle: Handle) -> Option<NaiveDateTime> {
    text(handle)
        .and_then(|s| DateTime::parse_from_rfc3339(&s.trim()).ok())
        .map(|n| n.naive_utc())
}

pub fn timestamp_from_rfc2822(handle: Handle) -> Option<NaiveDateTime> {
    text(handle)
        .and_then(|s| DateTime::parse_from_rfc2822(&s.trim()).ok())
        .map(|n| n.naive_utc())
}

pub fn timestamp(handle: Handle) -> Option<NaiveDateTime> {
    text(handle)
        .and_then(|s| DateTime::parse_from_rfc2822(&s.trim()).ok().or(
            DateTime::parse_from_rfc3339(&s.trim()).ok()
        )).map(|n| n.naive_utc())
}
*/
