use std::io::Read;

use chrono::NaiveDateTime;

use crate::model::{Category, Feed, Generator, Link, Person, Text, Entry, Image, Content};
use crate::util::{attr_value, timestamp_from_rfc2822};
use crate::util::element_source::Element;
use mime::Mime;

#[cfg(test)]
mod tests;

/// Parses an RSS 2.0 feed into our model
pub fn parse<R: Read>(root: Element<R>) -> Option<Feed> {
    // Only expecting a channel element
    for child in root.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "channel" => return Some(handle_channel(child)),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    None
}

// Handles the <channel> element
fn handle_channel<R: Read>(channel: Element<R>) -> Feed {
    let mut feed = Feed::new();

    for child in channel.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "title" => feed.title = handle_text(child),
            "link" => if let Some(link) = handle_link(child) { feed.links.push(link) },
            "description" => feed.description = handle_text(child),

            "language" => feed.language = child.child_as_text(),
            "copyright" => feed.rights = handle_text(child),
            "managingEditor" => if let Some(person) = handle_contact("managingEditor", child) { feed.contributors.push(person) },
            "webMaster" => if let Some(person) = handle_contact("webMaster", child) { feed.contributors.push(person) },
            "pubDate" => feed.published = handle_date_rfc2822(child),
            "lastBuildDate" => if let Some(ts) = handle_date_rfc2822(child) { feed.updated = ts },
            "category" => if let Some(category) = handle_category(child) { feed.categories.push(category) },
            "generator" => feed.generator = child.child_as_text().map(|content| Generator::new(content)),
            "ttl" => if let Some(text) = child.child_as_text() { feed.ttl = text.parse::<u32>().ok() },
            "image" => feed.logo = handle_image(child),
            "item" => feed.entries.push(handle_item(child)),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // RSS 2.0 defines <lastBuildDate> on an item as optional so for completeness we set them to the updated date of the feed
    for entry in feed.entries.iter_mut() {
        entry.updated = feed.updated;
    }

    feed
}

// Handles <category>
fn handle_category<R: Read>(element: Element<R>) -> Option<Category> {
    element.child_as_text().map(|text| {
        let mut category = Category::new(text);
        category.scheme = attr_value(&element.attributes, "domain").map(|s| s.to_owned());
        category
    })
}

// Handles <managingEditor> and <webMaster>
fn handle_contact<R: Read>(role: &str, element: Element<R>) -> Option<Person> {
    element.child_as_text().map(|email| {
        let mut person = Person::new(role.to_owned());
        person.email = Some(email);
        person
    })
}

// Handles an RFC 2822 (822) date
fn handle_date_rfc2822<R: Read>(element: Element<R>) -> Option<NaiveDateTime> {
    element.child_as_text()
        .map_or(None, |text| timestamp_from_rfc2822(&text))
}

// Handles <enclosure>
fn handle_enclosure<R: Read>(element: Element<R>) -> Option<Content> {
    let mut content = Content::new();

    for attr in &element.attributes {
        let tag_name = attr.name.local_name.as_str();
        match tag_name {
            // TODO can we avoid the clone
            "url" => content.src = Some(attr.value.clone()),
            "length" => content.length = attr.value.parse::<u64>().ok(),
            "type" => if let Ok(mime) = attr.value.parse::<Mime>() { content.content_type = mime },

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // No point returning the enclosure if we don't have a URL
    if content.src.is_none() {
        None
    } else {
        Some(content)
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
            "width" => if let Some(width) = child.child_as_text() { if let Ok(width) = width.parse::<u32>() { if width > 0 && width <= 144 { image.width = Some(width) } } },
            "height" => if let Some(height) = child.child_as_text() { if let Ok(height) = height.parse::<u32>() { if height > 0 && height <= 400 { image.height = Some(height) } } },
            "description" => image.description = child.child_as_text(),

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
fn handle_item<R: Read>(item: Element<R>) -> Entry {
    let mut entry = Entry::new();

    for child in item.children() {
        let tag_name = child.name.local_name.as_str();
        match tag_name {
            "title" => entry.title = handle_text(child),
            "link" => if let Some(link) = handle_link(child) { entry.links.push(link) },
            "description" => entry.summary = handle_text(child),
            "author" => if let Some(person) = handle_contact("author", child) { entry.authors.push(person) },
            "category" => if let Some(category) = handle_category(child) { entry.categories.push(category) },
            // TODO add isPermaLink to link - https://validator.w3.org/feed/docs/rss2.html#ltguidgtSubelementOfLtitemgt
            "guid" => if let Some(guid) = child.child_as_text() { entry.id = guid },
            "enclosure" => entry.content = handle_enclosure(child),
            "pubDate" => entry.published = handle_date_rfc2822(child),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    entry
}

// Handles <link>
fn handle_link<R: Read>(element: Element<R>) -> Option<Link> {
    element.child_as_text().map(|uri| Link::new(uri))
}

// Handles <title>, <description>
fn handle_text<R: Read>(element: Element<R>) -> Option<Text> {
    element.child_as_text().map(|content| Text::new(content))
}
