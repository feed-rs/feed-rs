use std::cell::RefCell;
use std::io::BufRead;
use std::mem;
use std::time::Duration;

use mediatype::{MediaTypeBuf, names};

use crate::model::{Entry, Image, MediaCommunity, MediaContent, MediaCredit, MediaObject, MediaObjectSource, MediaRating, MediaText, MediaThumbnail, Text};
use crate::parser::util::{if_ok_then_some, if_some_then, parse_npt};
use crate::parser::{ParseErrorKind, ParseFeedError, ParseFeedResult, util};
use crate::xml::{Element, NS};

pub(crate) struct MediaRssState {
    root: RefCell<Option<Scope>>,
}

#[derive(Default)]
struct Scope {
    title: Option<Text>,
    community: Option<MediaCommunity>,
    credits: Vec<MediaCredit>,
    description: Option<Text>,
    rating: Option<MediaRating>,
    texts: Vec<MediaText>,
    thumbnails: Vec<MediaThumbnail>,

    // <media:content> elements we have discovered at this scope which we process later into media objects
    pending: Vec<MediaContent>,

    // Children (i.e. <media:group> elements that nest MediaRSS content below them)
    children: Vec<Scope>,
}

impl MediaRssState {
    /// Initialise parsing for this entry
    pub(crate) fn entry_begin(&self) {
        // We should not have a scope at this point, since all entries are parsed separately and no media RSS elements are processed at the feed level
        assert!(self.root.borrow().is_none());

        // Initialise the scope
        self.root.borrow_mut().replace(Scope::default());
    }

    pub(crate) fn entry_end(&self, entry: &mut Entry) {
        // Emit media objects as required
        let scope = self.root.replace(None).unwrap();
        emit_media_objects(scope, entry);
    }

    /// Handle a primary or scoped optional MediaRSS element at the entry level
    pub(crate) fn handle_entry_mediarss_element<R: BufRead>(&self, element: Element<R>) -> ParseFeedResult<()> {
        // We will parse this element into the top-level scope
        if let Some(scope) = self.root.borrow_mut().as_mut() {
            // Pass off to the inner handle to process the element
            inner_handle_mediarss_element(element, scope)?;

            Ok(())
        } else {
            Err(ParseFeedError::ParseError(ParseErrorKind::IllegalState(
                "scope was not setup during mediarss entry parsing".to_string(),
            )))
        }
    }

    pub(crate) fn new() -> MediaRssState {
        MediaRssState { root: RefCell::new(None) }
    }
}

fn emit_media_objects(mut scope: Scope, entry: &mut Entry) {
    // If we have content at this scope, then build a media object for it
    if !scope.pending.is_empty() {
        // Build a media object for this scope
        let mut media_obj = MediaObject::new(MediaObjectSource::MediaRSS);
        media_obj.title = scope.title.clone();
        media_obj.description = scope.description.clone();
        media_obj.thumbnails = scope.thumbnails.clone();
        media_obj.community = scope.community.clone();
        media_obj.thumbnails = scope.thumbnails.clone();
        media_obj.texts = scope.texts.clone();
        media_obj.credits = scope.credits.clone();

        // Propagate the rating to the content if not already set
        let mut pending = mem::take(&mut scope.pending);
        if scope.rating.is_some() {
            pending.iter_mut().for_each(|content| {
                if content.rating.is_none() {
                    content.rating = scope.rating.clone();
                }
            });
        }

        // Add this content to the media object and emit
        media_obj.content = pending;
        entry.media.push(media_obj);
    }

    // Process each of the children
    let children = scope.children.drain(..).collect::<Vec<_>>();
    children.into_iter().for_each(|mut child| {
        // Propagate this scope to the child
        merge_parent_scope_into_child(&scope, &mut child);

        // Assemble the media objects on the child
        emit_media_objects(child, entry);
    });
}

fn inner_handle_mediarss_element<R: BufRead>(element: Element<R>, scope: &mut Scope) -> ParseFeedResult<()> {
    // A "group" element indicates that the children are different representations of the same content - https://www.rssboard.org/media-rss#media-group
    if let (NS::MediaRSS, "group") = element.ns_and_tag() {
        // Create a new scope for the nested elements
        let mut nested = Scope::default();

        // Process the children of this group node
        for child in element.children() {
            let child = child?;
            inner_handle_mediarss_element(child, &mut nested)?;
        }

        // Add this nested scope
        scope.children.push(nested);
    } else {
        // Process the Media RSS elements
        match element.ns_and_tag() {
            (NS::MediaRSS, "title") => scope.title = handle_text(element)?,

            (NS::MediaRSS, "content") => handle_media_content(element, scope)?,

            (NS::MediaRSS, "thumbnail") => if_some_then(handle_media_thumbnail(element), |thumbnail| scope.thumbnails.push(thumbnail)),

            (NS::MediaRSS, "description") => scope.description = handle_text(element)?,

            (NS::MediaRSS, "community") => scope.community = handle_media_community(element)?,

            (NS::MediaRSS, "credit") => if_some_then(handle_media_credit(element), |credit| scope.credits.push(credit)),

            (NS::MediaRSS, "text") => if_some_then(handle_media_text(element), |text| scope.texts.push(text)),

            (NS::MediaRSS, "rating") => scope.rating = handle_media_rating(element),

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(())
}

fn merge_parent_scope_into_child(parent: &Scope, child: &mut Scope) {
    if child.title.is_none() {
        child.title = parent.title.clone();
    }
    if child.community.is_none() {
        child.community = parent.community.clone();
    }
    if child.description.is_none() {
        child.description = parent.description.clone();
    }
    if child.rating.is_none() {
        child.rating = parent.rating.clone();
    }

    if child.credits.is_empty() {
        child.credits = parent.credits.clone();
    }
    if child.texts.is_empty() {
        child.texts = parent.texts.clone();
    }
    if child.thumbnails.is_empty() {
        child.thumbnails = parent.thumbnails.clone();
    }
}

// Handle "media:community"
fn handle_media_community<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<MediaCommunity>> {
    let mut community = MediaCommunity::new();

    for child in element.children() {
        let child = child?;
        match child.ns_and_tag() {
            (NS::MediaRSS, "starRating") => {
                for attr in &child.attributes {
                    match attr.name.as_str() {
                        "average" => if_ok_then_some(attr.value.parse::<f64>(), |v| community.stars_avg = v),
                        "count" => if_ok_then_some(attr.value.parse::<u64>(), |v| community.stars_count = v),
                        "min" => if_ok_then_some(attr.value.parse::<u64>(), |v| community.stars_min = v),
                        "max" => if_ok_then_some(attr.value.parse::<u64>(), |v| community.stars_max = v),

                        // Nothing required for unknown attributes
                        _ => {}
                    }
                }
            }
            (NS::MediaRSS, "statistics") => {
                for attr in &child.attributes {
                    match attr.name.as_str() {
                        "views" => if_ok_then_some(attr.value.parse::<u64>(), |v| community.stats_views = v),
                        "favorites" => if_ok_then_some(attr.value.parse::<u64>(), |v| community.stats_favorites = v),

                        // Nothing required for unknown attributes
                        _ => {}
                    }
                }
            }

            // Nothing required for unknown elements
            _ => {}
        }
    }

    Ok(Some(community))
}

// Handle the core attributes and elements from "media:content"
fn handle_media_content<R: BufRead>(element: Element<R>, media_obj: &mut Scope) -> ParseFeedResult<()> {
    let mut content = MediaContent::new();

    // Extract attributes from the content element
    for attr in &element.attributes {
        match attr.name.as_str() {
            "url" => content.url = util::parse_uri(&attr.value, element.xml_base.as_ref()),

            "type" => if_ok_then_some(attr.value.parse::<MediaTypeBuf>(), |v| content.content_type = v),

            "width" => if_ok_then_some(attr.value.parse::<u32>(), |v| content.width = v),
            "height" => if_ok_then_some(attr.value.parse::<u32>(), |v| content.height = v),

            "fileSize" => if_ok_then_some(attr.value.parse::<u64>(), |v| content.size = v),

            "duration" => if_ok_then_some(attr.value.parse::<u64>(), |v| content.duration = v.map(Duration::from_secs)),

            // Nothing required for unknown attributes
            _ => {}
        }
    }

    // Extract information from the child elements
    for child in element.children() {
        let child = child?;
        match child.ns_and_tag() {
            (NS::MediaRSS, "player") => {
                // According to the spec at https://www.rssboard.org/media-rss#media-content
                // "url should specify the direct URL to the media object. If not included, a <media:player> element must be specified."
                // So if we find a player element here, and it has a URL then we assign it
                if let Some(player_url) = child.attr_value("url") {
                    let player_url = util::parse_uri(&player_url, element.xml_base.as_ref());
                    if player_url.is_some() {
                        content.url = player_url;
                    }
                }
            }

            (NS::MediaRSS, "rating") => content.rating = handle_media_rating(child),

            // These elements are modelled as fields on the parent MediaObject, but only set if the parent field does not already have a value
            (NS::MediaRSS, "title") if media_obj.title.is_none() => media_obj.title = handle_text(child)?,

            (NS::MediaRSS, "description") if media_obj.description.is_none() => media_obj.description = handle_text(child)?,

            // These elements are accumulated in the corresponding field of the parent MediaObject
            (NS::MediaRSS, "text") => if_some_then(handle_media_text(child), |text| media_obj.texts.push(text)),
            (NS::MediaRSS, "credit") => if_some_then(handle_media_credit(child), |credit| media_obj.credits.push(credit)),

            // Other elements in the namespace are handled recursively
            (NS::MediaRSS, _) => inner_handle_mediarss_element(child, media_obj)?,

            // Nothing required for unknown elements
            _ => {}
        }
    }

    // If we found a URL from the content element or the player, we consider this valid
    if content.url.is_some() {
        // Emit this parsed content
        media_obj.pending.push(content);
    }

    Ok(())
}

// Handles the "media:credit" element
fn handle_media_credit<R: BufRead>(element: Element<R>) -> Option<MediaCredit> {
    element.child_as_text().map(MediaCredit::new)
}

// Handles the "media:rating" element
fn handle_media_rating<R: BufRead>(element: Element<R>) -> Option<MediaRating> {
    // Schema is "urn:simple" by default
    let scheme = element.attr_value("scheme").unwrap_or_else(|| "urn:simple".into());

    element.child_as_text().map(|rating| MediaRating::new(rating).urn(scheme.as_str()))
}

// Handles the "media:text" element
fn handle_media_text<R: BufRead>(element: Element<R>) -> Option<MediaText> {
    let mut start_time = None;
    let mut end_time = None;
    let mut mime = None;
    for attr in &element.attributes {
        match attr.name.as_str() {
            "start" => if_some_then(parse_npt(&attr.value), |npt| start_time = Some(npt)),
            "end" => if_some_then(parse_npt(&attr.value), |npt| end_time = Some(npt)),
            "type" => {
                mime = match attr.value.as_str() {
                    "plain" => Some(MediaTypeBuf::new(names::TEXT, names::PLAIN)),
                    "html" => Some(MediaTypeBuf::new(names::TEXT, names::HTML)),
                    _ => None,
                }
            }

            // Nothing required for unknown attributes
            _ => {}
        }
    }

    element.child_as_text().map(|t| {
        // Parse out the actual text of this element
        let mut text = Text::new(t);
        text.content_type = mime.map_or(MediaTypeBuf::new(names::TEXT, names::PLAIN), |m| m);
        let mut media_text = MediaText::new(text);

        // Add the time boundaries if we found them
        media_text.start_time = start_time;
        media_text.end_time = end_time;

        media_text
    })
}

// Handles the "media:thumbnail" element
fn handle_media_thumbnail<R: BufRead>(element: Element<R>) -> Option<MediaThumbnail> {
    // Extract the attributes on the thumbnail element
    let mut url = None;
    let mut width = None;
    let mut height = None;
    let mut time = None;
    for attr in &element.attributes {
        match attr.name.as_str() {
            "url" => url = Some(attr.value.clone()),

            "width" => if_ok_then_some(attr.value.parse::<u32>(), |v| width = v),
            "height" => if_ok_then_some(attr.value.parse::<u32>(), |v| height = v),

            "time" => if_some_then(parse_npt(&attr.value), |npt| time = Some(npt)),

            // Nothing required for unknown attributes
            _ => {}
        }
    }

    // We need url at least to assemble the image
    if let Some(url) = url {
        let mut image = Image::new(url);
        image.width = width;
        image.height = height;

        let mut thumbnail = MediaThumbnail::new(image);
        thumbnail.time = time;

        Some(thumbnail)
    } else {
        None
    }
}

// Handles a title or description element
fn handle_text<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Text>> {
    // Find type, defaulting to "plain" if not present
    let type_attr = element.attributes.iter().find(|a| &a.name == "type").map_or("plain", |a| a.value.as_str());

    let mime = match type_attr {
        "plain" => Ok(MediaTypeBuf::new(names::TEXT, names::PLAIN)),
        "html" => Ok(MediaTypeBuf::new(names::TEXT, names::HTML)),

        // Unknown content type
        _ => Err(ParseFeedError::ParseError(ParseErrorKind::UnknownMimeType(type_attr.into()))),
    }?;

    element
        .children_as_string()?
        .map(|content| {
            let mut text = Text::new(content);
            text.content_type = mime;
            Some(text)
        })
        // Need the text for a text element
        .ok_or(ParseFeedError::ParseError(ParseErrorKind::MissingContent("text")))
}
