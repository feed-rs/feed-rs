use crate::model::{Feed, Group, MediaObject, PodcastPerson, Role, Transcript};
use crate::parser::util::{if_some_then, parse_uri};
use crate::parser::{util, ParseFeedResult};
use crate::xml::{Element, NS};
use mediatype::MediaTypeBuf;
use std::io::BufRead;
use std::str::FromStr;
use url::Url;

// Process <podcast> elements at channel level updating the Feed object as required
pub(crate) fn handle_podcast_channel_element<R: BufRead>(element: Element<R>, feed: &mut Feed) -> ParseFeedResult<()> {
    if let (NS::Podcast, "person") = element.ns_and_tag() {
        if_some_then(handle_person(element)?, |person| feed.people.push(person))
    }

    Ok(())
}

/// Process <podcast> elements at entry level updating the MediaObject as required
pub(crate) fn handle_podcast_item_element<R: BufRead>(element: Element<R>, media_obj: &mut MediaObject) -> ParseFeedResult<()> {
    match element.ns_and_tag() {
        (NS::Podcast, "person") => if_some_then(handle_person(element)?, |person| media_obj.people.push(person)),

        (NS::Podcast, "season") => if_some_then(util::handle_season(element), |season| media_obj.season = Some(season)),

        (NS::Podcast, "episode") => if_some_then(util::handle_episode(element), |episode| media_obj.episode = Some(episode)),

        (NS::Podcast, "transcript") => if_some_then(handle_transcript(element), |transcript| media_obj.transcripts.push(transcript)),

        // skip unknown properties
        _ => {}
    }

    Ok(())
}

// Handles <podcast:person>
fn handle_person<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<PodcastPerson>> {
    let line = element.line_number();
    if let Some(role) = element.attr_value("role") {
        let role = Role::from_str(&role).map_err(|e| e.with_line(line))?;
        if let Some(name) = element.child_as_text() {
            return Ok(Some(PodcastPerson {
                name,
                role,
                group: element
                    .attr_value("group")
                    .map(|g| Group::from_str(&g).map_err(|e| e.with_line(line)))
                    .transpose()?
                    .unwrap_or_default(),
                img: element.attr_value("img").and_then(|img| Url::parse(&img).ok()),
                href: element.attr_value("href").and_then(|href| Url::parse(&href).ok()),
            }));
        }
    }

    Ok(None)
}

// Handles <podcast:transcript>
fn handle_transcript<R: BufRead>(element: Element<R>) -> Option<Transcript> {
    if let Some(url) = element.attr_value("url").and_then(|url| parse_uri(&url, None)) {
        if let Some(content_type) = element.attr_value("type").and_then(|content_type| content_type.parse::<MediaTypeBuf>().ok()) {
            return Some(Transcript {
                url,
                content_type,
                language: None,
                rel: element.attr_value("rel"),
            });
        }
    }

    None
}
