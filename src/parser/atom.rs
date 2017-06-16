use chrono::prelude::*;
use chrono::{NaiveDateTime};
use xml5ever::rcdom::{NodeData, Handle};
use feed::Feed;
use entry::{Entry, Link};
use super::{attr, text, uuid_gen, timestamp};

pub fn handle_atom(handle: Handle) -> Option<Feed> {
    let node = handle;
    let mut feed = Feed::new();
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, ref attrs, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "id"    => feed.id = text(child.clone()).unwrap_or(uuid_gen()),
                    "title" => feed.title = text(child.clone()),
                    "subtitle" => feed.description = text(child.clone()),
                    "updated" => feed.last_updated = timestamp(child.clone()),
                    "link" => {
                        // rel
                        //    self
                        let attributes = &attrs.borrow();
                        let rel = attr("rel", attributes).unwrap_or("".to_string());
                        let href = attr("href", attributes);
                        match (rel.as_ref(), href) {
                            ("self", Some(href)) => feed.id = format!("feed/{}", href),
                            (_, Some(href)) => feed.website = Some(href),
                            _ => (),
                        }
                    },
                    //"author" => (),
                    "logo" => feed.visual_url = text(child.clone()),
                    "icon" => feed.icon_url = text(child.clone()),
                    "generator" => (),
                    "contributor" => (),
                    "category" => {},
                    "rights" => (),
                    "entry" => {
                        if let Some(entry) = handle_entry(child.clone()) {
                            feed.entries.push(entry)
                        }
                    },
                    _ => (),
                }
            },
            _ => {},
        }
    }
    Some(feed)
}

pub fn handle_entry(handle: Handle) -> Option<Entry> {
    let node = handle;
    let mut entry = Entry::new();
    let mut published: Option<NaiveDateTime> = None;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, ref attrs, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "id" => entry.id = text(child.clone()).unwrap_or(uuid_gen()),
                    "title" => entry.title = text(child.clone()),
                    "summary" => entry.summary = text(child.clone()),
                    "content" => {
                        //entry.content = text(child.clone()),
                        let attributes = &attrs.borrow();
                        let content_type = attr("type", attributes).unwrap_or("text".to_string());
                        let _ = attr("src", attributes);
                        match content_type.as_ref() {
                            "text" => (),
                            "html" => (),
                            "xhtml" => (),
                            _ => (),
                        }
                    },
                    "author" => {},
                    "link" => {
                        let attributes = &attrs.borrow();
                        let rel       = attr("rel", attributes).unwrap_or("alternate".to_string());
                        let mime_type = attr("type", attributes).unwrap_or("text/html".to_string());
                        let length    = attr("length", attributes).and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
                        if let Some(url) = attr("href", attributes) {
                            match rel.as_ref() {
                                "enclosure" => entry.enclosure.push(Link::enc(mime_type, length, url)),
                                "alternate" => entry.alternate.push(Link::new(&mime_type, url)),
                                rel           => {
                                    println!("unprocessed rel {}", rel);
                                },
                            }
                        }
                    },
                    "published" => published = timestamp(child.clone()),
                    "updated" => {
                        entry.updated = timestamp(child.clone());
                        if published.is_none() {
                            published = timestamp(child.clone());
                        }
                    },
                    "category" => {
                        let attributes = &attrs.borrow();
                        let term   = attr("term", attributes);
                        let scheme = attr("schema", attributes);
                        let label  = attr("label", attributes);
                        match (term, scheme, label) {
                            (Some(term), _, _) => entry.keywords.push(term),
                            _ => (),
                        }
                    },
                    "contributor" => (),
                    "rights" => (),
                    "source" => (),
                    _ => (),
                }
            },
            _ => (),
        }
    }
    entry.published = published.unwrap_or(UTC::now().naive_utc());
    Some(entry)
}
