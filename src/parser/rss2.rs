use chrono::prelude::*;
use xml5ever::rcdom::{NodeData, Handle};
use feed::Feed;
use entry::{Entry, Link};
use super::{attr, text, uuid_gen, timestamp_from_rfc2822};

pub fn handle_rss2(handle: Handle) -> Option<Feed> {
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "channel" => return handle_channel(child.clone()),
                    _ => (),
                }
            },
            _ => {},
        }
    }
    None
}

pub fn handle_channel(handle: Handle) -> Option<Feed> {
    let mut feed = Feed::new();
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "title" => feed.title = text(child.clone()),
                    "description" => feed.description = text(child.clone()),
                    "link" => feed.website = text(child.clone()),
                    "language" => feed.language = text(child.clone()),
                    "lastBuildDate" => feed.last_updated = timestamp_from_rfc2822(child.clone()),
                    "pubDate" => (),
                    "managingEditor" => (),
                    "webMaster" => (),
                    "copyright" => (),
                    "docs" => (),
                    "cloud" => (),
                    "ttl" => (),
                    "image" => (),
                    "textInput" => (),
                    "skipHours" => (),
                    "skipDays" => (),
                    "category" => {
                        //TODO
                    },
                    "item" => {
                        if let Some(entry) = handle_item(child.clone()) {
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

pub fn handle_item(handle: Handle) -> Option<Entry> {
    let mut entry = Entry::new();
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, ref attrs, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "title" => entry.title = text(child.clone()),
                    "description" => entry.summary = text(child.clone()),
                    "link" => {
                        entry.alternate = text(child.clone())
                            .map(|s| vec![Link::new("text/html", s)])
                            .unwrap_or(vec![])
                    },
                    "author" => entry.author = text(child.clone()),
                    "category" => if let Some(s) = text(child.clone()) {
                        entry.keywords.push(s)
                    },
                    "comments" => {}, // TODO
                    "enclosure" => {
                        let attributes = &attrs.borrow();
                        let mime_type  = attr("type", attributes);
                        let length     = attr("length", attributes).and_then(|s| s.parse::<i64>().ok());
                        let url        = attr("url", attributes);
                        match (mime_type, length, url) {
                            (Some(mime_type), Some(length), Some(url)) => {
                                entry.enclosure.push(Link::enc(mime_type, length, url))
                            },
                            _ => (),
                        }
                    },
                    "guid" => entry.id = text(child.clone()).unwrap_or(uuid_gen()),
                    "pubDate" =>
                        entry.published = timestamp_from_rfc2822(child.clone()).unwrap_or(UTC::now().naive_utc()),
                    "source" => {}, // TODO
                    _ => (),
                }
            }
            _ => (),
        }
    }
    Some(entry)
}
