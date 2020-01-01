use std::io::Read;

use mime::Mime;

use crate::model::{Category, Content, Entry, Feed, Generator, Image, Link, Person, Text};
use crate::util::{attr_value, timestamp_from_rfc3339};
use crate::util::element_source::Element;
use crate::parser::{ParseFeedResult, ParseFeedError, ParseErrorKind};

#[cfg(test)]
mod tests;

/// Parses an Atom feed into our model
pub fn parse<R: Read>(root: Element<R>) -> ParseFeedResult<Feed> {
    let mut feed = Feed::default();
    for child in root.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "id" => if let Some(id) = child.child_as_text()? { feed.id = id },
            "title" => feed.title = handle_text(child)?,
            "updated" => if let Some(text) = child.child_as_text()? { if let Ok(ts) = timestamp_from_rfc3339(&text) { feed.updated = ts } },

            "author" => if let Some(person) = handle_person(child)? { feed.authors.push(person) }
            "link" => if let Some(link) = handle_link(child)? { feed.links.push(link) },

            "category" => if let Some(category) = handle_category(child)? { feed.categories.push(category) },
            "contributor" => if let Some(person) = handle_person(child)? { feed.contributors.push(person) },
            "generator" => feed.generator = handle_generator(child)?,
            "icon" => feed.icon = handle_image(child)?,
            "logo" => feed.logo = handle_image(child)?,
            "rights" => feed.rights = handle_text(child)?,
            "subtitle" => feed.description = handle_text(child)?,

            "entry" => if let Some(entry) = handle_entry(child)? { feed.entries.push(entry) },

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(feed)
}

// Handles an Atom <category>
fn handle_category<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Category>> {
    // Always need a term
    if let Some(term) = attr_value(&element.attributes, "term") {
        let mut category = Category::new(term.to_owned());

        for attr in &element.attributes {
            match attr.name.local_name.as_str() {
                "scheme" => category.scheme = Some(attr.value.clone()),
                "label" => category.label = Some(attr.value.clone()),

                // Nothing required for unknown attributes
                _ => {}
            }
        }

        Ok(Some(category))
    } else {
        // A missing category isn't fatal
        Ok(None)
    }
}

// Handles an Atom <content> element
// TODO idiomatic treatment of options, errors etc
fn handle_content<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Content>> {
    // Extract the content type so we can parse the body
    let content_type = element.attributes.iter()
        .find(|a| &a.name.local_name == "type")
        .map(|oa| oa.value.as_str());

    if let Some(ct) = content_type {
        // from http://www.atomenabled.org/developers/syndication/#contentElement
        match ct {
            // Should be handled as a text element per "In the most common case, the type attribute is either text, html, xhtml, in which case the content element is defined identically to other text constructs"
            "text" | "html" | "xhtml" => {
                handle_text(element)?.map(|text| {
                    let mut content = Content::default();
                    content.body = Some(text.content);
                    content.content_type = text.content_type;
                    Some(content)
                })
                    // The text is required for a text or HTML element
                    .ok_or(ParseFeedError::ParseError(ParseErrorKind::MissingContent("content.text")))
            }

            // XML per "Otherwise, if the type attribute ends in +xml or /xml, then an xml document of this type is contained inline."
            ct if ct.ends_with(" +xml") || ct.ends_with("/xml") => {
                element.child_as_text()?.map(|body| {
                    let mut content = Content::default();
                    content.body = Some(body);
                    content.content_type = mime::TEXT_XML;
                    Some(content)
                })
                    // The XML is required for an XML content element
                    .ok_or(ParseFeedError::ParseError(ParseErrorKind::MissingContent("content.xml")))
            }

            // Escaped text per "Otherwise, if the type attribute starts with text, then an escaped document of this type is contained inline."
            ct if ct.starts_with("text") => {
                if let Ok(mime) = ct.parse::<Mime>() {
                    element.child_as_text()?.map(|body| {
                        let mut content = Content::default();
                        content.body = Some(body);
                        content.content_type = mime;
                        Some(content)
                    })
                        // The text is required for an inline text element
                        .ok_or(ParseFeedError::ParseError(ParseErrorKind::MissingContent("content.inline")))
                } else {
                    Err(ParseFeedError::ParseError(ParseErrorKind::UnknownMimeType(ct.into())))
                }
            }

            // Unknown content type
            _ => Err(ParseFeedError::ParseError(ParseErrorKind::UnknownMimeType(ct.into())))
        }
    } else {
        // We can't parse without a content type
        Err(ParseFeedError::ParseError(ParseErrorKind::MissingContent("content.type")))
    }
}

// Handles an Atom <entry>
fn handle_entry<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Entry>> {
    let mut entry = Entry::default();

    for child in element.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            // Extract the fields from the spec
            "id" => if let Some(id) = child.child_as_text()? { entry.id = id },
            "title" => entry.title = handle_text(child)?,
            "updated" => if let Some(text) = child.child_as_text()? { if let Ok(ts) = timestamp_from_rfc3339(&text) { entry.updated = ts } },

            "author" => if let Some(person) = handle_person(child)? { entry.authors.push(person) },
            "content" => entry.content = handle_content(child)?,
            "link" => if let Some(link) = handle_link(child)? { entry.links.push(link) },
            "summary" => entry.summary = handle_text(child)?,

            "category" => if let Some(category) = handle_category(child)? { entry.categories.push(category) },
            "contributor" => if let Some(person) = handle_person(child)? { entry.contributors.push(person) },
            "published" => if let Some(text) = child.child_as_text()? { entry.published = timestamp_from_rfc3339(&text).ok() },
            "rights" => entry.rights = handle_text(child)?,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(Some(entry))
}

// Handles an Atom <generator>
fn handle_generator<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Generator>> {
    let generator = element.child_as_text()?.map(|content| {
        let mut generator = Generator::new(content);

        for attr in &element.attributes {
            match attr.name.local_name.as_str() {
                "uri" => generator.uri = Some(attr.value.clone()),
                "version" => generator.version = Some(attr.value.clone()),
                // Nothing required for unknown attributes
                _ => {}
            }
        }

        generator
    });

    Ok(generator)
}

// Handles an Atom <icon> or <logo>
fn handle_image<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Image>> {
    Ok(element.child_as_text()?.map(Image::new))
}

// Handles an Atom <link>
fn handle_link<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Link>> {
    // Always need an href
    let link = attr_value(&element.attributes, "href").and_then(|href| {
        let mut link = Link::new(href.to_owned());

        for attr in &element.attributes {
            match attr.name.local_name.as_str() {
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
    });

    Ok(link)
}

// Handles an Atom <author> or <contributor>
fn handle_person<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Person>> {
    let mut person = Person::new(String::from("unknown"));

    for child in element.children() {
        let tag_name = child.name.local_name.as_str();
        let child_text = child.child_as_text()?;
        match (tag_name, child_text) {
            // Extract the fields from the spec
            ("name", Some(name)) => person.name = name,
            ("uri", uri) => person.uri = uri,
            ("email", email) => person.email = email,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(Some(person))
}

// Directly handles an Atom <title>, <summary>, <rights> or <subtitle> element
fn handle_text<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Text>> {
    // Find type, defaulting to "text" if not present
    let type_attr = element.attributes.iter()
        .find(|a| &a.name.local_name == "type")
        .map_or("text", |a| a.value.as_str());

    let mime = match type_attr {
        "text" => Ok(mime::TEXT_PLAIN),
        "html" | "xhtml" => Ok(mime::TEXT_HTML),

        // Unknown content type
        _ => Err(ParseFeedError::ParseError(ParseErrorKind::UnknownMimeType(type_attr.into())))
    }?;

    element.child_as_text()?.map(|content| {
        let mut text = Text::new(content);
        text.content_type = mime;
        Some(text)
    })
        // Need the text for a text element
        .ok_or(ParseFeedError::ParseError(ParseErrorKind::MissingContent("text")))
}
