use crate::model::{Feed, Person, Entry, Link};
use std::io::Read;
use crate::util::element_source::Element;
use crate::util::{uuid_gen, timestamp_from_rfc3339, attr_value};

/// Parses an Atom feed into our model
pub fn parse<R: Read>(root: Element<R>) -> Option<Feed> {
    let mut feed = Feed::new();
    for child in root.children() {
        let tag_name = child.name.local_name.as_str();
        let child_text = child.child_as_text();
        match (tag_name, child_text) {
            ("id", Some(id)) => feed.id = id,
            ("title", Some(title)) => feed.title = title,
            ("updated", Some(text)) => if let Some(ts) = timestamp_from_rfc3339(&text) { feed.updated = ts },
            ("author", _) => feed.authors.push(handle_person(child)),
            ("entry", _) => feed.entries.push(handle_entry(child)),

            // Nothing required in the default case
            _ => {}
        }
    }

    Some(feed)
}

// Handles an Atom <entry>
fn handle_entry<R: Read>(element: Element<R>) -> Entry {
    let mut entry = Entry::new();

    for child in element.children() {
        let tag_name = child.name.local_name.as_str();
        let child_text = child.child_as_text();
        match (tag_name, child_text) {
            // Extract the fields from the spec
            ("id", Some(id)) => entry.id = id,
            ("title", Some(title)) => entry.title = title,
            ("updated", Some(text)) => if let Some(ts) = timestamp_from_rfc3339(&text) { entry.updated = ts },
            ("link", _) => if let Some(href) = attr_value(&child.attributes, "href") { entry.link = Some(Link::new(href.to_owned())); },
            ("summary", summary) => entry.summary = summary,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    entry
}

// Handles an Atom <author> or <contributor>
fn handle_person<R: Read>(element: Element<R>) -> Person {
    let mut person = Person::new(String::from("unknown"));

    for child in element.children() {
        let tag_name = child.name.local_name.as_str();
        let child_text = child.child_as_text();
        match (tag_name, child_text) {
            // Extract the fields from the spec
            ("name", Some(name)) => person.name = name,
            ("uri", uri) => person.uri = uri,
            ("email", email) => person.email = email,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    person
}

#[cfg(test)]
mod tests {
    use crate::util;
    use crate::util::test;
    use crate::parser;
    use crate::model::{Person, Entry};

    // Verify we can parse the example contained in the Atom specification
    #[test]
    fn test_spec_1() {
        // Parse the feed
        let test_data = test::fixture_as_string("atom_spec_1.xml");
        let feed = parser::parse(test_data.as_bytes()).unwrap();

        // Mandatory fields
        assert_eq!(feed.id, "urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6");
        assert_eq!(feed.title, "Example Feed");
        assert_eq!(feed.updated, util::timestamp_from_rfc3339("2003-12-13T18:30:02Z").unwrap());

        // Optional fields
        assert_eq!(feed.authors, vec!(Person::new(String::from("John Doe"))));

        // Entries
        let entry = Entry::new()
            .id("urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a")
            .title("Atom-Powered Robots Run Amok")
            .updated("2003-12-13T18:30:02Z")
            .summary("Some text.")
            .link("http://example.org/2003/12/13/atom03");
        assert_eq!(feed.entries, vec!(entry));
    }
}

/*
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
                                _           => {
                                    // println!("unprocessed rel {}", rel);
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
    entry.published = published.unwrap_or(Utc::now().naive_utc());
    Some(entry)
}
*/
