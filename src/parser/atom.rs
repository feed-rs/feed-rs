use std::io::Read;

use crate::model::{Category, Entry, Feed, Generator, Link, Person, Image, Text};
use crate::util::{attr_value, timestamp_from_rfc3339};
use crate::util::element_source::Element;

/// Parses an Atom feed into our model
pub fn parse<R: Read>(root: Element<R>) -> Option<Feed> {
    let mut feed = Feed::new();
    for child in root.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "id" => if let Some(id) = child.child_as_text() { feed.id = id },
            "title" => feed.title = handle_text(child),
            "updated" => if let Some(text) = child.child_as_text() { if let Some(ts) = timestamp_from_rfc3339(&text) { feed.updated = ts } },

            "author" => if let Some(person) = handle_person(child) { feed.authors.push(person) },
            "link" => if let Some(link) = handle_link(child) { feed.links.push(link) },

            "category" => if let Some(category) = handle_category(child) { feed.categories.push(category) },
            "contributor" => if let Some(person) = handle_person(child) { feed.contributors.push(person) },
            "generator" => feed.generator = handle_generator(child),
            "icon" => feed.icon = handle_image(child),
            "logo" => feed.logo = handle_image(child),
            "rights" => feed.rights = handle_text(child),
            "subtitle" => feed.description = handle_text(child),

            "entry" => feed.entries.push(handle_entry(child)),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Some(feed)
}

// Handles an Atom <category>
fn handle_category<R: Read>(element: Element<R>) -> Option<Category> {
    // Always need a term
    if let Some(term) = attr_value(&element.attributes, "term") {
        let mut category = Category::new(term.to_owned());

        for attr in &element.attributes {
            match attr.name.local_name.as_str() {
                // TODO can we avoid the clone() with a mutable/moved vec iterator?
                "scheme" => category.scheme = Some(attr.value.clone()),
                "label" => category.label = Some(attr.value.clone()),

                // Nothing required for unknown attributes
                _ => {}
            }
        }

        Some(category)
    } else {
        None
    }
}

// Handles an Atom <entry>
fn handle_entry<R: Read>(element: Element<R>) -> Entry {
    let mut entry = Entry::new();

    for child in element.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            // Extract the fields from the spec
            "id" => if let Some(id) = child.child_as_text() { entry.id = id },
            "title" => entry.title = child.child_as_text().map(|title| Text::new(title)),
            "updated" => if let Some(text) = child.child_as_text() { if let Some(ts) = timestamp_from_rfc3339(&text) { entry.updated = ts } },

            "author" => if let Some(person) = handle_person(child) { entry.authors.push(person) },
            "content" => entry.content = handle_text(child),
            "link" => if let Some(link) = handle_link(child) { entry.links.push(link) },
            "summary" => entry.summary = child.child_as_text().map(|summary| Text::new(summary)),

            "category" => if let Some(category) = handle_category(child) { entry.categories.push(category) },
            "contributor" => if let Some(person) = handle_person(child) { entry.contributors.push(person) },
            "published" => if let Some(text) = child.child_as_text() { entry.published = timestamp_from_rfc3339(&text) },
            // TODO source (need to roll up the raw XML)
            // "source" => feed.rights = handle_text(child),
            "rights" => entry.rights = handle_text(child),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    entry
}

// Handles an Atom <generator>
fn handle_generator<R: Read>(element: Element<R>) -> Option<Generator> {
    element.child_as_text().map(|content| {
        let mut generator = Generator::new(content);

        for attr in &element.attributes {
            match attr.name.local_name.as_str() {
                // TODO can we avoid the clone
                "uri" => generator.uri = Some(attr.value.clone()),
                "version" => generator.version = Some(attr.value.clone()),
                // Nothing required for unknown attributes
                _ => {}
            }
        }

        generator
    })
}

// Handles an Atom <icon> or <logo>
fn handle_image<R: Read>(element: Element<R>) -> Option<Image> {
    element.child_as_text().map(|uri| Image::new(uri))
}

// Handles an Atom <link>
fn handle_link<R: Read>(element: Element<R>) -> Option<Link> {
    // Always need an href
    if let Some(href) = attr_value(&element.attributes, "href") {
        let mut link = Link::new(href.to_owned());

        for attr in &element.attributes {
            match attr.name.local_name.as_str() {
                // TODO can we avoid the clone
                "rel" => link.rel = Some(attr.value.clone()),
                "type" => link.media_type = Some(attr.value.clone()),
                "hreflang" => link.hreflang = Some(attr.value.clone()),
                "title" => link.title = Some(attr.value.clone()),
                "length" => link.length = attr.value.parse::<u64>().ok(),

                // Nothing required for unrecognised attributes
                _ => {}
            }
        }
        // Default "rel" to "alternate" if not set
        if link.rel.is_none() {
            link.rel = Some(String::from("alternate"));
        }

        Some(link)
    } else {
        None
    }
}

// Handles an Atom <author> or <contributor>
fn handle_person<R: Read>(element: Element<R>) -> Option<Person> {
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

    Some(person)
}

// Handles an Atom <title>, <summary>, <content>, <rights> or <subtitle> element
fn handle_text<R: Read>(element: Element<R>) -> Option<Text> {
    element.child_as_text().map(|content| {
        let mut text = Text::new(content);

        if let Some(content_type) = attr_value(&element.attributes, "type") {
            text.content_type = content_type.to_owned();
        }

        text
    })
}

#[cfg(test)]
mod tests {
    use crate::model::{Entry, Person, Text, Link};
    use crate::parser;
    use crate::util;
    use crate::util::test;

    // Verify we can parse a more complete example
    #[test]
    fn test_example_1() {
        // Parse the feed
        let test_data = test::fixture_as_string("atom_example_1.xml");
        let feed = parser::parse(test_data.as_bytes()).unwrap();

        // Mandatory fields
        assert_eq!(feed.id, "tag:example.org,2003:3");
        assert_eq!(feed.title, Some(Text::new("dive into mark".to_owned())));
        assert_eq!(feed.updated, util::timestamp_from_rfc3339("2005-07-31T12:29:29Z").unwrap());

        // Expected entry
        let entry = Entry::new()
            .id("tag:example.org,2003:3.2397")
            .title("Atom draft-07 snapshot")
            .updated("2005-07-31T12:29:29Z")
            .author(Person::new("Mark Pilgrim".to_owned())
                .uri("http://example.org/")
                .email("f8dy@example.com"))
            .link(Link::new("http://example.org/2005/04/02/atom".to_owned())
                .rel("alternate")
                .media_type("text/html"))
            .link(Link::new("http://example.org/audio/ph34r_my_podcast.mp3".to_owned())
                .rel("enclosure")
                .media_type("audio/mpeg")
                .length(1337))
            .contributor(Person::new("Sam Ruby".to_owned()))
            .contributor(Person::new("Joe Gregorio".to_string()))
            .published("2003-12-13T08:29:29-04:00");

        assert_eq!(feed.entries, vec!(entry));
    }

    // Verify we can parse the example contained in the Atom specification
    #[test]
    fn test_spec_1() {
        // Parse the feed
        let test_data = test::fixture_as_string("atom_spec_1.xml");
        let feed = parser::parse(test_data.as_bytes()).unwrap();

        // Mandatory fields
        assert_eq!(feed.id, "urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6");
        assert_eq!(feed.title, Some(Text::new("Example Feed".to_owned())));
        assert_eq!(feed.updated, util::timestamp_from_rfc3339("2003-12-13T18:30:02Z").unwrap());

        // Optional fields
        assert_eq!(feed.authors, vec!(Person::new(String::from("John Doe"))));

        // Entries
        let entry = Entry::new()
            .id("urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a")
            .title("Atom-Powered Robots Run Amok")
            .updated("2003-12-13T18:30:02Z")
            .summary("Some text.")
            .link(Link::new("http://example.org/2003/12/13/atom03".to_owned())
                .rel("alternate"));
        assert_eq!(feed.entries, vec!(entry));
    }
}
