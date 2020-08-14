use std::io::BufRead;

use chrono::{DateTime, Utc};

use crate::model::{Entry, Feed, FeedType, Image, Link, Person, Text};
use crate::parser::ParseFeedResult;
use crate::parser::util::timestamp_rfc2822_lenient;
use crate::xml::{Element, NS};

#[cfg(test)]
mod tests;

/// Parses an RSS 1.0 feed into our model
pub(crate) fn parse<R: BufRead>(root: Element<R>) -> ParseFeedResult<Feed> {
    let mut feed = Feed::new(FeedType::RSS1);

    for child in root.children() {
        let child = child?;
        match child.ns_and_tag() {
            (None, "channel") => handle_channel(&mut feed, child)?,
            (None, "image") => feed.logo = handle_image(child)?,
            (None, "item") => if let Some(entry) = handle_item(child)? { feed.entries.push(entry) },

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(feed)
}

// Handles the <channel> element
fn handle_channel<R: BufRead>(feed: &mut Feed, channel: Element<R>) -> ParseFeedResult<()> {
    for child in channel.children() {
        let child = child?;
        match child.ns_and_tag() {
            (None, "title") => feed.title = handle_text(child)?,
            (None, "link") => if let Some(link) = handle_link(child)? { feed.links.push(link) },
            (None, "description") => feed.description = handle_text(child)?,

            (Some(NS::DublinCore), "creator") => if let Some(name) = child.child_as_text()? { feed.authors.push(Person::new(&name)) },
            (Some(NS::DublinCore), "date") => feed.published = handle_timestamp(child),
            (Some(NS::DublinCore), "language") => feed.language = child.child_as_text()?,
            (Some(NS::DublinCore), "rights") => feed.rights = handle_text(child)?,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(())
}

// Handles <image>
fn handle_image<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Image>> {
    let mut image = Image::new("".to_owned());

    for child in element.children() {
        let child = child?;
        match child.ns_and_tag() {
            (None, "url") => if let Some(url) = child.child_as_text()? { image.uri = url },
            (None, "title") => image.title = child.child_as_text()?,
            (None, "link") => if let Some(uri) = child.child_as_text()? { image.link = Some(Link::new(uri)) },

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
fn handle_item<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Entry>> {
    let mut entry = Entry::default();

    for child in element.children() {
        let child = child?;
        match child.ns_and_tag() {
            (None, "title") => entry.title = handle_text(child)?,
            (None, "link") => if let Some(link) = handle_link(child)? { entry.links.push(link) },
            (None, "description") => entry.summary = handle_text(child)?,

            (Some(NS::DublinCore), "creator") => if let Some(name) = child.child_as_text()? { entry.authors.push(Person::new(&name)) },
            (Some(NS::DublinCore), "date") => entry.published = handle_timestamp(child),
            (Some(NS::DublinCore), "description") => if entry.summary.is_none() { entry.summary = handle_text(child)? },
            (Some(NS::DublinCore), "rights") => entry.rights = handle_text(child)?,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // If we found at least 1 link
    Ok(if !entry.links.is_empty() {
        Some(entry)
    } else {
        // No point returning anything if we are missing a destination
        None
    })
}

// Handles <link>
fn handle_link<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Link>> {
    Ok(element.child_as_text()?.map(Link::new))
}

// Handles <title>, <description>
fn handle_text<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Text>> {
    Ok(element.child_as_text()?.map(Text::new))
}

// Handles date/time
fn handle_timestamp<R: BufRead>(element: Element<R>) -> Option<DateTime<Utc>> {
    if let Ok(Some(text)) = element.child_as_text() {
        timestamp_rfc2822_lenient(&text)
    } else {
        None
    }
}
