use std::io::Read;

use crate::model::{Category, Entry, Feed, Generator, Link, Person, Image, Text, Content};
use crate::util::{attr_value, timestamp_from_rfc3339};
use crate::util::element_source::Element;

#[cfg(test)]
mod tests;

// TODO expand test coverage to verify all elements + attributes are parsed

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

// Handles an Atom <content> element
fn handle_content<R: Read>(element: Element<R>) -> Option<Content> {
    element.child_as_text().map(|content| {
        let mut content = Content::new(content);

        if let Some(content_type) = attr_value(&element.attributes, "type") {
            content.content_type = Some(content_type.to_owned());
        }

        content
    })
}

// Handles an Atom <entry>
fn handle_entry<R: Read>(element: Element<R>) -> Entry {
    let mut entry = Entry::new();

    for child in element.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            // Extract the fields from the spec
            "id" => if let Some(id) = child.child_as_text() { entry.id = id },
            "title" => entry.title = handle_text(child),
            "updated" => if let Some(text) = child.child_as_text() { if let Some(ts) = timestamp_from_rfc3339(&text) { entry.updated = ts } },

            "author" => if let Some(person) = handle_person(child) { entry.authors.push(person) },
            "content" => entry.content = handle_content(child),
            "link" => if let Some(link) = handle_link(child) { entry.links.push(link) },
            "summary" => entry.summary = handle_text(child),

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
                "hreflang" => link.href_lang = Some(attr.value.clone()),
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

// Handles an Atom <title>, <summary>, <rights> or <subtitle> element
fn handle_text<R: Read>(element: Element<R>) -> Option<Text> {
    element.child_as_text().map(|content| {
        let mut text = Text::new(content);

        if let Some(content_type) = attr_value(&element.attributes, "type") {
            text.content_type = content_type.to_owned();
        }

        text
    })
}
