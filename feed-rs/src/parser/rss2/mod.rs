use mediatype::{MediaTypeBuf, names};
use std::collections::HashSet;
use std::io::BufRead;

use crate::model::{Category, Content, Entry, Feed, FeedType, Generator, Image, Link, LinkTarget, MediaContent, MediaObject, MediaObjectSource, Person};
use crate::parser::itunes::{handle_itunes_channel_element, handle_itunes_item_element};
use crate::parser::podcast::{handle_podcast_channel_element, handle_podcast_item_element};
use crate::parser::util::{if_ok_then_some, if_some_then};
use crate::parser::{ParseErrorKind, ParseFeedError, ParseFeedResult};
use crate::parser::{Parser, atom};
use crate::parser::{common, util};
use crate::xml::{Element, NS};

#[cfg(test)]
mod tests;

/// Parses an RSS 2.0 feed into our model
pub(crate) fn parse<R: BufRead>(parser: &Parser, root: Element<R>) -> ParseFeedResult<Feed> {
    // Only expecting a channel element
    let found_channel = root.children().find(|result| match result {
        Ok(element) => &element.name == "channel",
        Err(_) => true,
    });
    if let Some(channel) = found_channel {
        handle_channel(parser, channel?)
    } else {
        Err(ParseFeedError::ParseError(ParseErrorKind::NoFeedRoot))
    }
}

// Handles the <channel> element
fn handle_channel<R: BufRead>(parser: &Parser, channel: Element<R>) -> ParseFeedResult<Feed> {
    let mut feed = Feed::new(FeedType::RSS2);

    for child in channel.children() {
        let child = child?;
        match child.ns_and_tag() {
            (NS::RSS, "title") => feed.title = common::handle_text(child),

            (NS::RSS, "link") => if_some_then(common::handle_link(None, child), |link| feed.links.push(link)),

            (NS::Atom, "link") => if_some_then(atom::handle_link(child), |link| feed.links.push(link)),

            (NS::RSS, "description") => feed.description = common::handle_text(child),

            (NS::RSS, "language") => feed.language = child.child_as_text().map(|text| text.to_lowercase()),

            (NS::RSS, "copyright") => feed.rights = common::handle_text(child),

            (NS::RSS, "managingEditor") => if_some_then(handle_contact("managingEditor", child), |person| feed.contributors.push(person)),

            (NS::RSS, "webMaster") => if_some_then(handle_contact("webMaster", child), |person| feed.contributors.push(person)),

            (NS::RSS, "pubDate") => feed.published = util::handle_timestamp(parser, child),

            // Some feeds have "updated" instead of "lastBuildDate"
            (NS::RSS, "lastBuildDate") | (NS::RSS, "updated") => feed.updated = util::handle_timestamp(parser, child),

            (NS::RSS, "category") => if_some_then(handle_category(child), |category| feed.categories.push(category)),

            (NS::RSS, "generator") => feed.generator = handle_generator(child),

            (NS::RSS, "ttl") => if_some_then(child.child_as_text(), |text| if_ok_then_some(text.parse::<u32>(), |ttl| feed.ttl = ttl)),

            (NS::RSS, "image") => feed.logo = handle_image(child)?,

            (NS::RSS, "item") => if_some_then(handle_item(parser, child)?, |item| feed.entries.push(item)),

            (NS::Itunes, _) => handle_itunes_channel_element(child, &mut feed)?,

            (NS::Podcast, _) => handle_podcast_channel_element(child, &mut feed)?,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(feed)
}

// Handles <category>
fn handle_category<R: BufRead>(element: Element<R>) -> Option<Category> {
    element.children_as_string().ok().flatten().map(|text| {
        let mut category = Category::new(&text);
        category.scheme = element.attr_value("domain");
        category
    })
}

// Handles <managingEditor> and <webMaster>
fn handle_contact<R: BufRead>(role: &str, element: Element<R>) -> Option<Person> {
    element.child_as_text().map(|email| {
        let mut person = Person::new(role);
        person.email = Some(email);
        person
    })
}

fn handle_generator<R: BufRead>(element: Element<R>) -> Option<Generator> {
    element.child_as_text().map(|c| {
        let mut generator = Generator::new(&c);

        for attr in element.attributes {
            let tag_name = attr.name.as_str();
            if tag_name == "uri" {
                generator.uri = Some(attr.value.clone());
            }
        }

        generator
    })
}

// Handles <enclosure>
fn handle_enclosure<R: BufRead>(element: Element<R>, media_obj: &mut MediaObject) {
    let mut content = MediaContent::new();

    for attr in &element.attributes {
        let tag_name = attr.name.as_str();
        match tag_name {
            "url" => content.url = util::parse_uri(&attr.value, element.xml_base.as_ref()),
            "length" => content.size = attr.value.parse::<u64>().ok(),
            "type" => if_ok_then_some(attr.value.parse::<MediaTypeBuf>(), |mime| content.content_type = mime),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // Wrap in a media object if we have a sufficient definition of a media object
    if content.url.is_some() {
        media_obj.content.push(content);
    }
}

// Handles <image>
fn handle_image<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Image>> {
    let mut image = Image::new("".to_owned());

    for child in element.children() {
        let child = child?;
        match child.ns_and_tag() {
            (NS::RSS, "url") => if_some_then(child.child_as_text(), |url| image.uri = url),

            (NS::RSS, "title") => image.title = child.child_as_text(),

            (NS::RSS, "link") => if_some_then(child.child_as_text(), |uri| image.link = Some(Link::new(uri, element.xml_base.as_ref()))),

            (NS::RSS, "width") => if_some_then(child.child_as_text(), |width| {
                if let Ok(width) = width.parse::<u32>() {
                    if width > 0 && width <= 144 {
                        image.width = Some(width)
                    }
                }
            }),

            (NS::RSS, "height") => if_some_then(child.child_as_text(), |height| {
                if let Ok(height) = height.parse::<u32>() {
                    if height > 0 && height <= 400 {
                        image.height = Some(height)
                    }
                }
            }),

            (NS::RSS, "description") => image.description = child.child_as_text(),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // If we don't have a URI there is no point returning an image
    Ok(if !image.uri.is_empty() { Some(image) } else { None })
}

// Handles <content:encoded>
fn handle_content_encoded<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Content>> {
    let src = element.xml_base.as_ref().map(|xml_base| Link::new(xml_base, element.xml_base.as_ref()));

    Ok(element.children_as_string()?.and_then(|string| {
        if string.is_empty() {
            None
        } else {
            Some(Content {
                body: Some(string),
                content_type: MediaTypeBuf::new(names::TEXT, names::HTML),
                src,
                ..Default::default()
            })
        }
    }))
}

// Handles <item>
//
// There is some complexity around "enclosure", "content:encoded", MediaRSS and iTunes support
// * "enclosure": the RSS spec states that <enclosure> "Describes a media object that is attached to the item." - https://validator.w3.org/feed/docs/rss2.html#ltenclosuregtSubelementOfLtitemgt
// * "content:encoded": RSS best practices state <content:encoded> "...defines the full content of an item (OPTIONAL). This element has a more precise purpose than the description element, which can be the full content, a summary or some other form of excerpt at the publisher's discretion." - https://www.rssboard.org/rss-profile#namespace-elements-content-encoded
// * The MediaRSS, iTunes and Podcast namespaces define media objects or attributes of items in the feed
//
// Handling is as follows:
// * "enclosure" is treated as if it was a MediaRSS MediaContent element and wrapped in a MediaObject
// * "content:encoded" is mapped to the content field of an Entry
// * MediaRSS elements are handle via the MediaRSSState parser extension
// * iTunes elements are added to an iTunes specific MediaObject
// * Podcast elements are added to a Podcast specific MediaObject
fn handle_item<R: BufRead>(parser: &Parser, element: Element<R>) -> ParseFeedResult<Option<Entry>> {
    let mut entry = Entry::default();

    // Instances for enclosure, iTunes + Podcasts
    let mut enclosure_media_obj = MediaObject::new(MediaObjectSource::RSS);
    let mut itunes_media_obj = MediaObject::new(MediaObjectSource::ITunes);
    let mut podcast_media_obj = MediaObject::new(MediaObjectSource::Podcast);

    // Note we have started a new MediaRSS entry scope
    parser.mediarss.entry_begin();

    for child in element.children() {
        let child = child?;
        match child.ns_and_tag() {
            (NS::RSS, "title") => entry.title = common::handle_text(child),

            (NS::RSS, "link") => if_some_then(common::handle_link(None, child), |link| entry.links.push(link)),

            (NS::RSS, "description") => entry.summary = util::handle_encoded(child)?,

            (NS::RSS, "author") => if_some_then(handle_contact("author", child), |person| entry.authors.push(person)),

            (NS::RSS, "category") => if_some_then(handle_category(child), |category| entry.categories.push(category)),

            (NS::RSS, "guid") => if_some_then(child.child_as_text(), |guid| entry.id = guid.trim().to_string()),

            (NS::RSS, "enclosure") => handle_enclosure(child, &mut enclosure_media_obj),

            (NS::RSS, "pubDate") | (NS::DublinCore, "date") => entry.published = util::handle_timestamp(parser, child),

            (NS::Content, "encoded") => entry.content = handle_content_encoded(child)?,

            (NS::DublinCore, "creator") => if_some_then(child.children_as_string().ok().flatten(), |name| entry.authors.push(Person::new(&name))),

            // iTunes elements populate the corresponding MediaObject
            (NS::Itunes, _) => handle_itunes_item_element(child, &mut itunes_media_obj)?,

            // MediaRSS tags that are not grouped are parsed into the default object
            (NS::MediaRSS, _) => parser.mediarss.handle_entry_mediarss_element(child)?,

            // Podcast elements populate the default MediaObject
            (NS::Podcast, _) => handle_podcast_item_element(child, &mut podcast_media_obj)?,

            // Comments can use the Well Formed Web NS or the RSS comments element
            // According to https://validator.w3.org/feed/docs/warning/CommentRSS.html we need to support both
            (NS::WellFormedWebComments, "commentRss") | (NS::WellFormedWebComments, "commentRSS") => {
                if_some_then(common::handle_link(Some(LinkTarget::CommentsFeed), child), |link| entry.links.push(link))
            }
            (NS::RSS, "comments") => if_some_then(common::handle_link(Some(LinkTarget::Comments), child), |link| entry.links.push(link)),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // Finalise parsing the MediaRSS namespace, emitting MediaObjects if found
    parser.mediarss.entry_end(&mut entry);

    // Emit any other media objects with content
    for media_obj in [itunes_media_obj, podcast_media_obj, enclosure_media_obj] {
        if media_obj.is_not_empty() {
            entry.media.push(media_obj);
        }
    }

    // Post-process the media objects
    postprocess_media(&mut entry.media);

    // If we have a published date, copy this to updated too for consistency
    if entry.updated.is_none() && entry.published.is_some() {
        entry.updated = entry.published;
    }

    // Remove any comment links that simply target the entry
    clean_links(&mut entry.links);

    Ok(Some(entry))
}

// Remove comment links that are just the same target as another link we've already discovered
fn clean_links(links: &mut Vec<Link>) {
    // First we collect the set of links that are not of type Comment
    let non_comment_targets: HashSet<String> = links
        .iter()
        .filter(|link| link.target != Some(LinkTarget::Comments))
        .map(|link| link.href.clone())
        .collect::<HashSet<_>>();

    // Retain only those comment links that are distinct URLs
    links.retain(|link| link.target != Some(LinkTarget::Comments) || !non_comment_targets.contains(link.href.as_str()));
}

// Apply standard processing to the parsed set of media
fn postprocess_media(objects: &mut Vec<MediaObject>) {
    // Merge into the highest priority object, in order: MediaRSS, Podcast
    if let Some(mediarss_idx) = objects.iter().position(|mo| mo.source == Some(MediaObjectSource::MediaRSS)) {
        // Merge into MediaRSS source

        // Remove it from the list so we can update it with other elements
        let mut mediarss_obj = objects.swap_remove(mediarss_idx);

        // Run through the remaining objects merging in RSS and iTunes sources
        let len = objects.len();
        for idx in (0..len).rev() {
            if objects[idx].source == Some(MediaObjectSource::RSS) {
                let rss_obj = objects.swap_remove(idx);
                postprocess_merge_media(rss_obj, &mut mediarss_obj);
            } else if objects[idx].source == Some(MediaObjectSource::ITunes) {
                // If we don't have any itunes specific data, merge it into the standard mediarss obj
                let itunes_obj = &objects[idx];
                if itunes_obj.season.is_none() && itunes_obj.episode.is_none() {
                    let itunes_obj = objects.swap_remove(idx);
                    postprocess_merge_media(itunes_obj, &mut mediarss_obj);
                }
            }
        }

        // Push the updated mediarss object back into the list
        objects.push(mediarss_obj);
    } else if let Some(podcast_idx) = objects.iter().position(|mo| mo.source == Some(MediaObjectSource::Podcast)) {
        // Merge into podcast source

        // Remove it from the list so we can update it with other elements
        let mut postcast_obj = objects.swap_remove(podcast_idx);

        // Merge in other sources
        let len = objects.len();
        for idx in (0..len).rev() {
            let other = objects.swap_remove(idx);
            postprocess_merge_media(other, &mut postcast_obj);
        }

        // Push the updated mediarss object back into the list
        objects.push(postcast_obj);
    }
}

fn postprocess_merge_media(source: MediaObject, target: &mut MediaObject) {
    if target.title.is_none() {
        target.title = source.title;
    }
    if target.content.is_empty() {
        target.content = source.content;
    }
    if target.duration.is_none() {
        target.duration = source.duration;
    }
    if target.thumbnails.is_empty() {
        target.thumbnails = source.thumbnails;
    }
    if target.texts.is_empty() {
        target.texts = source.texts;
    }
    if target.description.is_none() {
        target.description = source.description;
    }
    if target.community.is_none() {
        target.community = source.community;
    }
    if target.credits.is_empty() {
        target.credits = source.credits;
    }
    if target.people.is_empty() {
        target.people = source.people;
    }
    if target.season.is_none() {
        target.season = source.season;
    }
    if target.episode.is_none() {
        target.episode = source.episode;
    }
    if target.transcripts.is_empty() {
        target.transcripts = source.transcripts;
    }
}
