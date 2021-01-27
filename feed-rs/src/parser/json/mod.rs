use std::io::Read;

use mime::Mime;

use crate::model::{Category, Content, Entry, Feed, FeedType, Image, Link, Person, Text};
use crate::parser::util::{timestamp_rfc3339_lenient, if_some_then};
use crate::parser::{ParseFeedError, ParseFeedResult};

#[cfg(test)]
mod tests;

/// Parses a JSON feed into our model
pub(crate) fn parse<R: Read>(stream: R) -> ParseFeedResult<Feed> {
    let parsed = serde_json::from_reader(stream);
    if let Ok(json_feed) = parsed {
        convert(json_feed)
    } else {
        // Unable to parse the JSON
        Err(ParseFeedError::JsonSerde(parsed.err().unwrap()))
    }
}

// Convert the JSON Feed into our standard model
fn convert(jf: JsonFeed) -> ParseFeedResult<Feed> {
    let mut feed = Feed::new(FeedType::JSON);

    // Convert feed level fields
    feed.title = Some(Text::new(jf.title));

    if_some_then(jf.home_page_url, |uri| feed.links.push(Link::new(uri)));

    if_some_then(jf.feed_url, |uri| feed.links.push(Link::new(uri)));

    if_some_then(jf.description, |text| feed.description = Some(Text::new(text)));

    if_some_then(jf.icon, |uri| feed.logo = Some(Image::new(uri)));

    if_some_then(jf.favicon, |uri| feed.icon = Some(Image::new(uri)));

    if_some_then(handle_person(jf.author), |person| feed.authors.push(person));

    // Convert items within the JSON feed
    jf.items.into_iter().for_each(|ji| {
        if let Ok(entry) = handle_item(ji) {
            feed.entries.push(entry);
        }
    });

    Ok(feed)
}

// Handles an attachment
fn handle_attachment(attachment: JsonAttachment) -> Link {
    let mut link = Link::new(attachment.url);

    link.media_type = Some(attachment.mime_type);
    link.title = attachment.title;
    link.length = attachment.size_in_bytes;

    link
}

// Handles HTML or plain text content
fn handle_content(content: Option<String>, content_type: Mime) -> Option<Content> {
    content.map(|body| {
        let mut content = Content::default();
        content.length = Some(body.as_bytes().len() as u64);
        content.body = Some(body.trim().into());
        content.content_type = content_type;
        content
    })
}

// Converts a JSON feed item into our model
fn handle_item(ji: JsonItem) -> ParseFeedResult<Entry> {
    let mut entry = Entry::default();

    entry.id = ji.id;

    if_some_then(ji.url, |uri| entry.links.push(Link::new(uri)));

    if_some_then(ji.external_url, |uri| entry.links.push(Link::new(uri)));

    if_some_then(ji.title, |text| entry.title = Some(Text::new(text)));

    // Content HTML, content text and summary are mapped across to our model with the preference toward HTML and explicit summary fields
    entry.content = handle_content(ji.content_html, mime::TEXT_HTML);
    entry.summary = ji.summary.map(Text::new);
    if let Some(content_text) = handle_content(ji.content_text, mime::TEXT_PLAIN) {
        // If we don't have HTML content, use the text content as the entry content
        // otherwise, if the summary was not provided, we push the text there

        if entry.content.is_none() {
            entry.content = Some(content_text);
        } else if entry.summary.is_none() {
            entry.summary = content_text.body.map(Text::new);
        }
    }

    if_some_then(ji.date_published, |published| entry.published = timestamp_rfc3339_lenient(&published));

    if_some_then(ji.date_modified, |modified| entry.updated = timestamp_rfc3339_lenient(&modified));

    if_some_then(handle_person(ji.author), |person| entry.authors.push(person));

    if_some_then(ji.tags, |tags| tags.into_iter().map(|t| Category::new(&t)).for_each(|category| entry.categories.push(category)));

    if_some_then(ji.attachments, |attachments| attachments.into_iter().map(handle_attachment).for_each(|link| entry.links.push(link)));

    Ok(entry)
}

// Converts an author object into our model
fn handle_person(author: Option<JsonAuthor>) -> Option<Person> {
    if let Some(ja) = author {
        if ja.name.is_some() {
            let mut person = Person::new(&ja.name.unwrap());

            person.uri = ja.url;

            return Some(person);
        }
    }

    None
}

#[derive(Debug, Deserialize)]
struct JsonFeed {
    pub version: String,
    pub title: String,
    pub home_page_url: Option<String>,
    pub feed_url: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub favicon: Option<String>,
    pub author: Option<JsonAuthor>,
    pub items: Vec<JsonItem>,
}

#[derive(Debug, Deserialize)]
struct JsonAttachment {
    url: String,
    mime_type: String,
    title: Option<String>,
    size_in_bytes: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct JsonAuthor {
    name: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JsonItem {
    pub id: String,
    pub url: Option<String>,
    pub external_url: Option<String>,
    pub title: Option<String>,
    pub content_html: Option<String>,
    pub content_text: Option<String>,
    pub summary: Option<String>,
    pub date_published: Option<String>,
    pub date_modified: Option<String>,
    pub author: Option<JsonAuthor>,
    pub tags: Option<Vec<String>>,
    pub attachments: Option<Vec<JsonAttachment>>,
}
