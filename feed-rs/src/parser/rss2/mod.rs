use std::io::Read;

use chrono::{DateTime, Utc};
use mime::Mime;

use crate::model::{Category, Content, Entry, Feed, Generator, Image, Link, Person, Text};
use crate::parser::{ParseFeedResult, ParseFeedError, ParseErrorKind};
use crate::util::{attr_value};
use crate::util::element_source::Element;
use crate::parser::util::timestamp_rfc2822_lenient;

#[cfg(test)]
mod tests;

/// Parses an RSS 2.0 feed into our model
pub fn parse<R: Read>(root: Element<R>) -> ParseFeedResult<Feed> {
    // Only expecting a channel element
    if let Some(channel) = root.children().find(|e| &e.name.local_name == "channel") {
        handle_channel(channel)
    } else {
        Err(ParseFeedError::ParseError(ParseErrorKind::NoFeedRoot))
    }
}

// Handles the <channel> element
fn handle_channel<R: Read>(channel: Element<R>) -> ParseFeedResult<Feed> {
    let mut feed = Feed::default();

    for child in channel.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "title" => feed.title = handle_text(child)?,
            "link" => if let Some(link) = handle_link(child)? { feed.links.push(link) },
            "description" => feed.description = handle_text(child)?,

            "language" => feed.language = child.child_as_text()?.map(|text| text.to_lowercase()),
            "copyright" => feed.rights = handle_text(child)?,
            "managingEditor" => if let Some(person) = handle_contact("managingEditor", child)? { feed.contributors.push(person) },
            "webMaster" => if let Some(person) = handle_contact("webMaster", child)? { feed.contributors.push(person) },
            "pubDate" => feed.published = handle_timestamp(child)?,

            // Some feeds have "updated" instead of "lastBuildDate"
            "lastBuildDate" | "updated" => if let Some(ts) = handle_timestamp(child)? { feed.updated = ts },

            "category" => if let Some(category) = handle_category(child)? { feed.categories.push(category) },
            "generator" => feed.generator = handle_generator(child)?,
            "ttl" => if let Some(text) = child.child_as_text()? { feed.ttl = text.parse::<u32>().ok() },
            "image" => feed.logo = handle_image(child)?,
            "item" => if let Some(item) = handle_item(child)? { feed.entries.push(item) },

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // RSS 2.0 defines <lastBuildDate> on an item as optional so for completeness we set them to the updated date of the feed
    for entry in feed.entries.iter_mut() {
        entry.updated = feed.updated;
    }

    Ok(feed)
}

// Handles <category>
fn handle_category<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Category>> {
    Ok(element.child_as_text()?.map(|text| {
        let mut category = Category::new(text);
        category.scheme = attr_value(&element.attributes, "domain").map(|s| s.to_owned());
        category
    }))
}

// Handles <managingEditor> and <webMaster>
fn handle_contact<R: Read>(role: &str, element: Element<R>) -> ParseFeedResult<Option<Person>> {
    Ok(element.child_as_text()?.map(|email| {
        let mut person = Person::new(role.to_owned());
        person.email = Some(email);
        person
    }))
}

fn handle_generator<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Generator>> {
    let result = element.child_as_text()?.map(|c| {
        let mut generator = Generator::new(c);

        for attr in &element.attributes {
            let tag_name = attr.name.local_name.as_str();
            if tag_name == "uri" {
                generator.uri = Some(attr.value.clone());
            }
        }

        generator
    });

    Ok(result)
}

// Handles <enclosure>
fn handle_enclosure<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Content>> {
    let mut content = Content::default();

    for attr in &element.attributes {
        let tag_name = attr.name.local_name.as_str();
        match tag_name {
            "url" => content.src = Some(Link::new(attr.value.clone())),
            "length" => content.length = attr.value.parse::<u64>().ok(),
            "type" => if let Ok(mime) = attr.value.parse::<Mime>() { content.content_type = mime },

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // No point returning the enclosure if we don't have a URL
    Ok(if content.src.is_some() {
        Some(content)
    } else {
        None
    })
}

// Handles <image>
fn handle_image<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Image>> {
    let mut image = Image::new("".to_owned());

    for child in element.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "url" => if let Some(url) = child.child_as_text()? { image.uri = url },
            "title" => image.title = child.child_as_text()?,
            "link" => if let Some(uri) = child.child_as_text()? { image.link = Some(Link::new(uri)) },
            "width" => if let Some(width) = child.child_as_text()? { if let Ok(width) = width.parse::<u32>() { if width > 0 && width <= 144 { image.width = Some(width) } } },
            "height" => if let Some(height) = child.child_as_text()? { if let Ok(height) = height.parse::<u32>() { if height > 0 && height <= 400 { image.height = Some(height) } } },
            "description" => image.description = child.child_as_text()?,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // If we don't have a URI there is no point returning an image
    Ok(if !image.uri.is_empty() {
        Some(image)
    } else {
        None
    })
}

// Handles <item>
fn handle_item<R: Read>(item: Element<R>) -> ParseFeedResult<Option<Entry>> {
    let mut entry = Entry::default();

    for child in item.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "title" => entry.title = handle_text(child)?,
            "link" => if let Some(link) = handle_link(child)? { entry.links.push(link) },
            "description" => entry.summary = handle_text(child)?,
            "author" => if let Some(person) = handle_contact("author", child)? { entry.authors.push(person) },
            "category" => if let Some(category) = handle_category(child)? { entry.categories.push(category) },
            "guid" => if let Some(guid) = child.child_as_text()? { entry.id = guid },
            "enclosure" => entry.content = handle_enclosure(child)?,
            "pubDate" => entry.published = handle_timestamp(child)?,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(Some(entry))
}

// Handles <link>
fn handle_link<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Link>> {
    Ok(element.child_as_text()?.map(Link::new))
}

// Handles <title>, <description>, <encoded>
fn handle_text<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Text>> {
    Ok(element.child_as_text()?.map(Text::new))
}

// Handles date/time
fn handle_timestamp<R: Read>(element: Element<R>) -> ParseFeedResult<Option<DateTime<Utc>>> {
    element.child_as_text()?
        .map(|text| timestamp_rfc2822_lenient(&text))
        .transpose()
}
