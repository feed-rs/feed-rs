use std::io::Read;

use chrono::{DateTime, NaiveDateTime};
use uuid::Uuid;
use xml5ever::Attribute;
use xml5ever::driver::parse_document;
use xml5ever::rcdom::{Handle, NodeData, RcDom};
use xml5ever::tendril::TendrilSink;

use crate::feed::Feed;

mod rss1;
mod rss2;
mod atom;

pub fn parse<R>(input: &mut R) -> Option<Feed> where R: Read {
    let mut buf = String::new();
    let _ = input.read_to_string(&mut buf);
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut buf.replace(" rdf:", " ").as_bytes()) // FIXME
        .unwrap();
    walk(dom.document)
}

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

pub fn uuid_gen() -> String {
    Uuid::new_v4().to_string()
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
