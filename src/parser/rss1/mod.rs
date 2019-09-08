use std::io::Read;

use crate::model::{Feed, Image, Link, Text, Entry};
use crate::util::element_source::Element;

#[cfg(test)]
mod tests;

/// Parses an RSS 1.0 feed into our model
pub fn parse<R: Read>(root: Element<R>) -> Option<Feed> {
    let mut feed = Feed::default();

    for child in root.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "channel" => handle_channel(&mut feed, child),
            "image" => feed.logo = handle_image(child),
            "item" => if let Some(entry) = handle_item(child) { feed.entries.push(entry) },

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Some(feed)
}

// Handles the <channel> element
fn handle_channel<R: Read>(feed: &mut Feed, channel: Element<R>) {
    for child in channel.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "title" => feed.title = handle_text(child),
            "link" => if let Some(link) = handle_link(child) { feed.links.push(link) },
            "description" => feed.description = handle_text(child),

            // Nothing required for unknown elements
            _ => {}
        }
    }
}

// Handles <image>
fn handle_image<R: Read>(element: Element<R>) -> Option<Image> {
    let mut image = Image::new("".to_owned());

    for child in element.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "url" => if let Some(url) = child.child_as_text() { image.uri = url },
            "title" => image.title = child.child_as_text(),
            "link" => if let Some(uri) = child.child_as_text() { image.link = Some(Link::new(uri)) },

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // If we don't have a URI there is no point returning an image
    if image.uri.is_empty() {
        None
    } else {
        Some(image)
    }
}

// Handles <item>
fn handle_item<R: Read>(element: Element<R>) -> Option<Entry> {
    let mut entry = Entry::default();

    for child in element.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "title" => entry.title = handle_text(child),
            "link" => if let Some(link) = handle_link(child) { entry.links.push(link) },
            "description" => entry.summary = handle_text(child),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // No point returning anything if we are missing a destination
    if !entry.links.is_empty() {
        Some(entry)
    } else {
        None
    }
}

// Handles <link>
fn handle_link<R: Read>(element: Element<R>) -> Option<Link> {
    element.child_as_text().map(Link::new)
}

// Handles <title>, <description>
fn handle_text<R: Read>(element: Element<R>) -> Option<Text> {
    element.child_as_text().map(Text::new)
}
