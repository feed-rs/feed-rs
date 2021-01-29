use crate::model::{Image, MediaContent, MediaCredit, MediaObject, MediaThumbnail};
use crate::parser::atom::handle_text;
use crate::parser::util::{if_some_then, parse_npt};
use crate::parser::ParseFeedResult;
use crate::xml::{Element, NS};
use std::io::BufRead;
use std::time::Duration;

// Ref:
// https://help.apple.com/itc/podcasts_connect/#/itcb54353390
// https://www.feedforall.com/itune-tutorial-tags.htm

// TODO:
// - Handle <itunes:> elements for the whole feed in <channel>
// - Add rating/adult support to MediaObject
// - More elements like itunes:subtitle, itunes:episode etc.

/*
// Handles <itunes:explicit>
fn handle_itunes_explicit<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<bool>> {
    Ok(element.child_as_text()?.and_then(|text| match text.as_ref() {
        "yes" | "true" => Some(true),
        "no" | "false" | "clean" => Some(false),
        _ => None,
    }))
}
*/

// Handles <itunes:image>
fn handle_itunes_image<R: BufRead>(element: Element<R>) -> Option<MediaThumbnail> {
    element.attr_value("href").map(|url| MediaThumbnail::new(Image::new(url)))
}

// Handles <itunes:duration>
fn handle_itunes_duration<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Duration>> {
    Ok(element.child_as_text()?.and_then(|text| parse_npt(&text)))
}

// Handles <itunes:author>
fn handle_itunes_author<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<MediaCredit>> {
    Ok(element.child_as_text()?.map(MediaCredit::new))
}

// Process <itunes> elements and turn them into something that looks like MediaRSS objects.
// This is used to enrich <content:enclosure> in the RSS2 feed.
pub(crate) fn handle_itunes_element<R: BufRead>(element: Element<R>, media_obj: &mut MediaObject) -> ParseFeedResult<()> {
    match element.ns_and_tag() {
        (Some(NS::Itunes), "title") => media_obj.title = handle_text(element)?,

        (Some(NS::Itunes), "image") => if_some_then(handle_itunes_image(element), |thumbnail| media_obj.thumbnails.push(thumbnail)),

        (Some(NS::Itunes), "duration") => if_some_then(handle_itunes_duration(element)?, |duration| {
            media_obj.content.get_or_insert_with(MediaContent::new).duration = Some(duration)
        }),

        (Some(NS::Itunes), "author") => if_some_then(handle_itunes_author(element)?, |credit| media_obj.credits.push(credit)),

        (Some(NS::Itunes), "summary") => media_obj.description = handle_text(element)?,

        // Nothing required for unknown elements
        _ => {}
    }

    Ok(())
}
