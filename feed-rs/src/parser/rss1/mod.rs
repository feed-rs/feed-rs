use std::io::Read;

use crate::model::{Entry, Feed, Image, Link, Text, FeedType};
use crate::parser::ParseFeedResult;
use crate::util::element_source::Element;

#[cfg(test)]
mod tests;

/// Parses an RSS 1.0 feed into our model
pub fn parse<R: Read>(root: Element<R>) -> ParseFeedResult<Feed> {
    let mut feed = Feed::new(FeedType::RSS1);

    for child in root.children() {
        let child = child?;
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "channel" => handle_channel(&mut feed, child)?,
            "image" => feed.logo = handle_image(child)?,
            "item" => if let Some(entry) = handle_item(child)? { feed.entries.push(entry) },

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(feed)
}

// Handles the <channel> element
fn handle_channel<R: Read>(feed: &mut Feed, channel: Element<R>) -> ParseFeedResult<()> {
    for child in channel.children() {
        let child = child?;
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "title" => feed.title = handle_text(child)?,
            "link" => if let Some(link) = handle_link(child)? { feed.links.push(link) },
            "description" => feed.description = handle_text(child)?,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(())
}

// Handles <image>
fn handle_image<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Image>> {
    let mut image = Image::new("".to_owned());

    for child in element.children() {
        let child = child?;
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "url" => if let Some(url) = child.child_as_text()? { image.uri = url },
            "title" => image.title = child.child_as_text()?,
            "link" => if let Some(uri) = child.child_as_text()? { image.link = Some(Link::new(uri)) },

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
fn handle_item<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Entry>> {
    let mut entry = Entry::default();

    for child in element.children() {
        let child = child?;
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "title" => entry.title = handle_text(child)?,
            "link" => if let Some(link) = handle_link(child)? { entry.links.push(link) },
            "description" => entry.summary = handle_text(child)?,

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
fn handle_link<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Link>> {
    Ok(element.child_as_text()?.map(Link::new))
}

// Handles <title>, <description>
fn handle_text<R: Read>(element: Element<R>) -> ParseFeedResult<Option<Text>> {
    Ok(element.child_as_text()?.map(Text::new))
}
