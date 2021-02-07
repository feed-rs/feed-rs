use std::io::BufRead;

use chrono::{DateTime, Utc};
use mime::Mime;

use crate::model::{Category, Content, Entry, Feed, FeedType, Generator, Image, Link, MediaContent, MediaObject, Person, Text};
use crate::parser::itunes::handle_itunes_element;
use crate::parser::mediarss;
use crate::parser::mediarss::handle_media_element;
use crate::parser::util::{if_ok_then_some, if_some_then, timestamp_rfc2822_lenient};
use crate::parser::{util, ParseErrorKind, ParseFeedError, ParseFeedResult};
use crate::xml::{Element, NS};

#[cfg(test)]
mod tests;

/// Parses an RSS 2.0 feed into our model
pub(crate) fn parse<R: BufRead>(root: Element<R>) -> ParseFeedResult<Feed> {
    // Only expecting a channel element
    let found_channel = root.children().find(|result| match result {
        Ok(element) => &element.name == "channel",
        Err(_) => true,
    });
    if let Some(channel) = found_channel {
        handle_channel(channel?)
    } else {
        Err(ParseFeedError::ParseError(ParseErrorKind::NoFeedRoot))
    }
}

// Handles the <channel> element
fn handle_channel<R: BufRead>(channel: Element<R>) -> ParseFeedResult<Feed> {
    let mut feed = Feed::new(FeedType::RSS2);

    for child in channel.children() {
        let child = child?;
        match child.ns_and_tag() {
            (None, "title") => feed.title = handle_text(child)?,

            (None, "link") => if_some_then(handle_link(child)?, |link| feed.links.push(link)),

            (None, "description") => feed.description = handle_text(child)?,

            (None, "language") => feed.language = child.child_as_text()?.map(|text| text.to_lowercase()),

            (None, "copyright") => feed.rights = handle_text(child)?,

            (None, "managingEditor") => if_some_then(handle_contact("managingEditor", child)?, |person| feed.contributors.push(person)),

            (None, "webMaster") => if_some_then(handle_contact("webMaster", child)?, |person| feed.contributors.push(person)),

            (None, "pubDate") => feed.published = handle_timestamp(child),

            // Some feeds have "updated" instead of "lastBuildDate"
            (None, "lastBuildDate") | (None, "updated") => feed.updated = handle_timestamp(child),

            (None, "category") => if_some_then(handle_category(child)?, |category| feed.categories.push(category)),

            (None, "generator") => feed.generator = handle_generator(child)?,

            (None, "ttl") => if_some_then(child.child_as_text()?, |text| if_ok_then_some(text.parse::<u32>(), |ttl| feed.ttl = ttl)),

            (None, "image") => feed.logo = handle_image(child)?,

            (None, "item") => if_some_then(handle_item(child)?, |item| feed.entries.push(item)),

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
fn handle_category<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Category>> {
    Ok(element.child_as_text()?.map(|text| {
        let mut category = Category::new(&text);
        category.scheme = element.attr_value("domain");
        category
    }))
}

// Handles <managingEditor> and <webMaster>
fn handle_contact<R: BufRead>(role: &str, element: Element<R>) -> ParseFeedResult<Option<Person>> {
    Ok(element.child_as_text()?.map(|email| {
        let mut person = Person::new(role);
        person.email = Some(email);
        person
    }))
}

fn handle_generator<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Generator>> {
    let result = element.child_as_text()?.map(|c| {
        let mut generator = Generator::new(&c);

        for attr in element.attributes {
            let tag_name = attr.name.as_str();
            if tag_name == "uri" {
                generator.uri = Some(attr.value.clone());
            }
        }

        generator
    });

    Ok(result)
}

// Handles <enclosure>
fn handle_enclosure<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<MediaContent>> {
    let mut content = MediaContent::new();

    for attr in element.attributes {
        let tag_name = attr.name.as_str();
        match tag_name {
            "url" => content.url = Some(attr.value),
            "length" => content.size = attr.value.parse::<u64>().ok(),
            "type" => if_ok_then_some(attr.value.parse::<Mime>(), |mime| content.content_type = mime),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // No point returning the enclosure if we don't have a URL
    Ok(if content.url.is_some() { Some(content) } else { None })
}

// Handles <image>
fn handle_image<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Image>> {
    let mut image = Image::new("".to_owned());

    for child in element.children() {
        let child = child?;
        match child.ns_and_tag() {
            (None, "url") => if_some_then(child.child_as_text()?, |url| image.uri = url),

            (None, "title") => image.title = child.child_as_text()?,

            (None, "link") => if_some_then(child.child_as_text()?, |uri| image.link = Some(Link::new(uri))),

            (None, "width") => if_some_then(child.child_as_text()?, |width| {
                if let Ok(width) = width.parse::<u32>() {
                    if width > 0 && width <= 144 {
                        image.width = Some(width)
                    }
                }
            }),

            (None, "height") => if_some_then(child.child_as_text()?, |height| {
                if let Ok(height) = height.parse::<u32>() {
                    if height > 0 && height <= 400 {
                        image.height = Some(height)
                    }
                }
            }),

            (None, "description") => image.description = child.child_as_text()?,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // If we don't have a URI there is no point returning an image
    Ok(if !image.uri.is_empty() { Some(image) } else { None })
}

// Handles <content:encoded>
fn handle_content<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Content>> {
    Ok(element.children_as_string()?.map(|string| Content {
        body: Some(string),
        content_type: mime::TEXT_PLAIN,
        ..Default::default()
    }))
}

// Handles <item>
fn handle_item<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Entry>> {
    let mut entry = Entry::default();

    // Create a default media content object for <itunes> elements.
    let mut media_itunes = MediaObject::new();

    // Create another default object for actual MediaRSS elements.
    let mut media_rss = MediaObject::new();

    // <enclosure> is used as media element and content fallback.
    let mut enclosure = None;

    for child in element.children() {
        let child = child?;
        match child.ns_and_tag() {
            (None, "title") => entry.title = handle_text(child)?,

            (None, "link") => if_some_then(handle_link(child)?, |link| entry.links.push(link)),

            (None, "description") => entry.summary = util::handle_encoded(child)?,

            (None, "author") => if_some_then(handle_contact("author", child)?, |person| entry.authors.push(person)),

            (None, "category") => if_some_then(handle_category(child)?, |category| entry.categories.push(category)),

            (None, "guid") => if_some_then(child.child_as_text()?, |guid| entry.id = guid),

            (None, "enclosure") => enclosure = handle_enclosure(child)?,

            (None, "pubDate") => entry.published = handle_timestamp(child),

            (Some(NS::Content), "encoded") => entry.content = handle_content(child)?,

            (Some(NS::DublinCore), "creator") => if_some_then(child.child_as_text()?, |name| entry.authors.push(Person::new(&name))),

            (Some(NS::Itunes), _) => handle_itunes_element(child, &mut media_itunes)?,

            // MediaRSS group creates a new object for this group of elements
            (Some(NS::MediaRSS), "group") => if_some_then(mediarss::handle_media_group(child)?, |obj| entry.media.push(obj)),

            // MediaRSS tags that are not grouped are parsed into the default object
            (Some(NS::MediaRSS), _) => handle_media_element(child, &mut media_rss)?,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // If we have an enclosure insert it as media with optional additonal itunes data.
    if let Some(enclosure) = enclosure {
        let content = media_itunes.content.get_or_insert_with(MediaContent::new);
        content.content_type = enclosure.content_type.clone();
        content.url = enclosure.url.clone();
        content.size = enclosure.size;
        entry.media.push(media_itunes);

        // Also use the enclosue as content when none exist so far.
        if entry.content.is_none() {
            entry.content = Some(Content {
                src: Some(Link::new(enclosure.url.unwrap())),
                content_type: enclosure.content_type.unwrap_or(mime::TEXT_PLAIN),
                length: enclosure.size,
                ..Default::default()
            });
        }
    }

    // If a media:content item with content exists, then attach it as well.
    if media_rss.content.is_some() {
        entry.media.push(media_rss);
    }

    Ok(Some(entry))
}

// Handles <link>
fn handle_link<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Link>> {
    Ok(element.child_as_text()?.map(Link::new))
}

// Handles <title>, <description> etc
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
