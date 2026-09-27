use crate::model::{Episode, Link, LinkTarget, Season, Text};
use crate::parser::{ParseFeedResult, Parser};
use crate::xml::Element;
use chrono::{DateTime, Utc};
use std::io::BufRead;

/// Handles <content:encoded>
pub(crate) fn handle_encoded<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Text>> {
    Ok(element.child_as_text().map(Text::html))
}

// Handles episode elements in the Podcast or iTunes namespace
pub fn handle_episode<R: BufRead>(element: Element<R>) -> Option<Episode> {
    element.child_as_text().and_then(|n| n.parse().ok()).map(|number| Episode {
        display: element.attr_value("display"),
        number,
    })
}

// Handles <link>
pub(crate) fn handle_link<R: BufRead>(target: Option<LinkTarget>, element: Element<R>) -> Option<Link> {
    element.child_as_text().map(|s| {
        let mut link = Link::new(s, element.xml_base.as_ref());
        link.target = target;
        link
    })
}

// Handles season elements in the Podcast or iTunes namespace
pub fn handle_season<R: BufRead>(element: Element<R>) -> Option<Season> {
    element.child_as_text().and_then(|n| n.parse().ok()).map(|number| Season {
        name: element.attr_value("name"),
        number,
    })
}

// Handles <title>, <description> etc
pub(crate) fn handle_text<R: BufRead>(element: Element<R>) -> Option<Text> {
    element.child_as_text().map(Text::new)
}

/// Handles date/time
pub(crate) fn handle_timestamp<R: BufRead>(parser: &Parser, element: Element<R>) -> Option<DateTime<Utc>> {
    if let Some(text) = element.child_as_text() {
        parser.parse_timestamp(&text)
    } else {
        None
    }
}
