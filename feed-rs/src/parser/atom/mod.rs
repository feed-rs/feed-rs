use std::io::BufRead;

use mime::Mime;

use crate::model::{Category, Content, Entry, Feed, FeedType, Generator, Image, Link, MediaObject, Person, Text};
use crate::parser::mediarss;
use crate::parser::mediarss::handle_media_element;
use crate::parser::util::{if_some_then, timestamp_rfc3339_lenient};
use crate::parser::{ParseErrorKind, ParseFeedError, ParseFeedResult};
use crate::xml::{Element, NS};

#[cfg(test)]
mod tests;

/// Parses an Atom feed into our model
pub(crate) fn parse_feed<R: BufRead>(root: Element<R>) -> ParseFeedResult<Feed> {
    let mut feed = Feed::new(FeedType::Atom);
    for child in root.children() {
        let child = child?;
        match child.ns_and_tag() {
            (None, "id") => if_some_then(child.child_as_text()?, |id| feed.id = id),

            (None, "title") => feed.title = handle_text(child)?,

            (None, "updated") => if_some_then(child.child_as_text()?, |text| feed.updated = timestamp_rfc3339_lenient(&text)),

            (None, "author") => if_some_then(handle_person(child)?, |person| feed.authors.push(person)),

            (None, "link") => if_some_then(handle_link(child)?, |link| feed.links.push(link)),

            (None, "category") => if_some_then(handle_category(child)?, |category| feed.categories.push(category)),

            (None, "contributor") => if_some_then(handle_person(child)?, |person| feed.contributors.push(person)),

            (None, "generator") => feed.generator = handle_generator(child)?,

            (None, "icon") => feed.icon = handle_image(child)?,

            (None, "logo") => feed.logo = handle_image(child)?,

            (None, "rights") => feed.rights = handle_text(child)?,

            (None, "subtitle") => feed.description = handle_text(child)?,

            (None, "entry") => if_some_then(handle_entry(child)?, |entry| feed.entries.push(entry)),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(feed)
}

/// Parses an Atom entry into our model
///
/// Note that the entry is wrapped in an empty Feed to keep the API consistent
pub(crate) fn parse_entry<R: BufRead>(root: Element<R>) -> ParseFeedResult<Feed> {
    let mut feed = Feed::new(FeedType::Atom);

    if_some_then(handle_entry(root)?, |entry| feed.entries.push(entry));

    Ok(feed)
}

// Handles an Atom <category>
fn handle_category<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Category>> {
    // Always need a term
    if let Some(term) = element.attr_value("term") {
        let mut category = Category::new(&term);

        for attr in element.attributes {
            match attr.name.as_str() {
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
fn handle_content<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Content>> {
    // Extract the content type so we can parse the body
    let content_type = element.attr_value("type");

    // from http://www.atomenabled.org/developers/syndication/#contentElement
    match content_type.as_deref() {
        // Should be handled as a text element per "In the most common case, the type attribute is either text, html, xhtml, in which case the content element is defined identically to other text constructs"
        Some("text") | Some("html") | Some("xhtml") | None => {
            handle_text(element)?
                .map(|text| {
                    let mut content = Content::default();
                    content.body = Some(text.content);
                    content.content_type = text.content_type;
                    Some(content)
                })
                // The text is required for a text or HTML element
                .ok_or(ParseFeedError::ParseError(ParseErrorKind::MissingContent("content.text")))
        }

        // XML per "Otherwise, if the type attribute ends in +xml or /xml, then an xml document of this type is contained inline."
        Some(ct) if ct.ends_with(" +xml") || ct.ends_with("/xml") => {
            handle_text(element)?
                .map(|body| {
                    let mut content = Content::default();
                    content.body = Some(body.content);
                    content.content_type = mime::TEXT_XML;
                    Some(content)
                })
                // The XML is required for an XML content element
                .ok_or(ParseFeedError::ParseError(ParseErrorKind::MissingContent("content.xml")))
        }

        // Escaped text per "Otherwise, if the type attribute starts with text, then an escaped document of this type is contained inline." and
        // also handles base64 encoded document of the indicated mime type per "Otherwise, a base64 encoded document of the indicated media type is contained inline."
        Some(ct) => {
            if let Ok(mime) = ct.parse::<Mime>() {
                element
                    .child_as_text()?
                    .map(|body| {
                        let content = Content {
                            body: Some(body),
                            content_type: mime,
                            ..Default::default()
                        };
                        Some(content)
                    })
                    // The text is required for an inline text or base64 element
                    .ok_or(ParseFeedError::ParseError(ParseErrorKind::MissingContent("content.inline")))
            } else {
                Err(ParseFeedError::ParseError(ParseErrorKind::UnknownMimeType(ct.into())))
            }
        }
    }
}

// Handles an Atom <entry>
fn handle_entry<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Entry>> {
    // Create a default MediaRSS content object for non-grouped elements
    let mut media_obj = MediaObject::new();

    // Parse the entry
    let mut entry = Entry::default();
    for child in element.children() {
        let child = child?;
        match child.ns_and_tag() {
            // Extract the fields from the spec
            (None, "id") => if_some_then(child.child_as_text()?, |id| entry.id = id),

            (None, "title") => entry.title = handle_text(child)?,

            (None, "updated") => if_some_then(child.child_as_text()?, |text| entry.updated = timestamp_rfc3339_lenient(&text)),

            (None, "author") => if_some_then(handle_person(child)?, |person| entry.authors.push(person)),

            (None, "content") => entry.content = handle_content(child)?,

            (None, "link") => if_some_then(handle_link(child)?, |link| entry.links.push(link)),

            (None, "summary") => entry.summary = handle_text(child)?,

            (None, "category") => if_some_then(handle_category(child)?, |category| entry.categories.push(category)),

            (None, "contributor") => if_some_then(handle_person(child)?, |person| entry.contributors.push(person)),

            (None, "published") => if_some_then(child.child_as_text()?, |text| entry.published = timestamp_rfc3339_lenient(&text)),

            (None, "rights") => entry.rights = handle_text(child)?,

            // MediaRSS group creates a new object for this group of elements
            (Some(NS::MediaRSS), "group") => if_some_then(mediarss::handle_media_group(child)?, |obj| entry.media.push(obj)),

            // MediaRSS tags that are not grouped are parsed into the default object
            (Some(NS::MediaRSS), _) => handle_media_element(child, &mut media_obj)?,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // If a media:content item was found in this entry, then attach it
    if media_obj.content.is_some() {
        entry.media.push(media_obj);
    }

    Ok(Some(entry))
}

// Handles an Atom <generator>
fn handle_generator<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Generator>> {
    let generator = element.child_as_text()?.map(|content| {
        let mut generator = Generator::new(&content);

        for attr in element.attributes {
            match attr.name.as_str() {
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
fn handle_image<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Image>> {
    Ok(element.child_as_text()?.map(Image::new))
}

// Handles an Atom <link>
fn handle_link<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Link>> {
    // Always need an href
    let link = element.attr_value("href").map(|href| {
        let mut link = Link::new(href);

        for attr in element.attributes {
            match attr.name.as_str() {
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

        link
    });

    Ok(link)
}

// Handles an Atom <author> or <contributor>
fn handle_person<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Person>> {
    let mut person = Person::new("unknown");

    for child in element.children() {
        let child = child?;
        let tag_name = child.name.as_str();
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
fn handle_text<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Text>> {
    // Find type, defaulting to "text" if not present
    let type_attr = element.attributes.iter().find(|a| &a.name == "type").map_or("text", |a| a.value.as_str());

    let mime = match type_attr {
        "text" => Ok(mime::TEXT_PLAIN),
        "html" | "xhtml" => Ok(mime::TEXT_HTML),

        // Unknown content type
        _ => Err(ParseFeedError::ParseError(ParseErrorKind::UnknownMimeType(type_attr.into()))),
    }?;

    element
        .children_as_string()?
        .map(|content| {
            let mut text = Text::new(content);
            text.content_type = mime;
            Some(text)
        })
        // Need the text for a text element
        .ok_or(ParseFeedError::ParseError(ParseErrorKind::MissingContent("text")))
}
