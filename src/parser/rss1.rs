use chrono::prelude::*;
use xml5ever::rcdom::{NodeData, Handle};
use feed::Feed;
use entry::{Entry, Link};
use super::{attr, text, timestamp};

pub fn handle_rss1(handle: Handle) -> Option<Feed> {
    let node = handle;
    let mut feed = None;
    let mut entries = vec![];
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, ref attrs, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "channel" => feed = handle_channel(child.clone()),
                    "item"    => {
                        if let Some(about) = attr("about", &attrs.borrow()) {
                            let f = feed.clone().unwrap();
                            let items = &mut f.entries.iter()
                                .filter(|e| e.id == about.to_string())
                                .map(|e| handle_item(child.clone(), e.id.to_string()))
                                .collect::<Vec<_>>();
                            entries.append(items);
                        }
                    },
                    _         => (),
                }
            },
            _ => {},
        }
    }
    if let Some(ref mut f) = feed {
        f.entries  = entries;
    }
    feed
}

pub fn handle_channel(handle: Handle) -> Option<Feed> {
    let mut feed = Feed::new();
    let node = handle;
    match node.data {
        NodeData::Element { ref attrs, .. } => {
            if let Some(about) = attr("about", &attrs.borrow()) {
                feed.id = about;
            }
        },
        _ => (),
    }
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "title" => feed.title = text(child.clone()),
                    "description" => feed.description = text(child.clone()),
                    "link" => feed.website = text(child.clone()),
                    "image" => (),
                    "textinput" => (),
                    "items" => handle_items(child.clone(), &mut feed),
                    "date" => feed.last_updated = timestamp(child.clone()),
                    "language" => feed.language = text(child.clone()),
                    _ => (),
                }
            },
            _ => {},
        }
    }
    Some(feed)
}

pub fn handle_items(handle: Handle, feed: &mut Feed) {
    let node = handle;
    let mut seq = None;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, .. } => {
                let tag_name = name.local.as_ref();
                if tag_name == "Seq" {
                    seq = Some(child.clone());
                }
            }
            _ => (),
        }
    }
    if seq.is_none() {
        return
    }
    let seq = seq.unwrap();
    for child in seq.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, ref attrs, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "li" => {
                        if let Some(resource) = attr("resource", &attrs.borrow()) {
                            let mut entry = Entry::new();
                            entry.id      = resource;
                            feed.entries.push(entry);
                        }
                    },
                    _    => (),
                }
            }
            _ => (),
        }
    }
}

pub fn handle_item(handle: Handle, id: String) -> Entry {
    let mut entry: Entry = Entry::new();
    entry.id = id;
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "title" => entry.title = text(child.clone()),
                    "description" => entry.summary = text(child.clone()),
                    "link" => {
                        entry.alternate = text(child.clone())
                            .map(|s| vec![Link::new("text/html", s)])
                            .unwrap_or(vec![])
                    },
                    // dc
                    "date" => entry.published = timestamp(child.clone()).unwrap_or(Utc::now().naive_utc()),
                    "creator" => entry.author = text(child.clone()),
                    "subject" => if let Some(s) = text(child.clone()) {
                        entry.keywords.push(s)
                    },
                    _ => (),
                }
            }
            _ => (),
        }
    }
    entry
}
