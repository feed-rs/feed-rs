use crate::model::{Episode, Link, LinkTarget, Season};
use crate::parser::Parser;
use crate::xml::Element;
use jiff::ZonedDateTime;
use std::io::BufRead;

// Handles episode elements in the Podcast or iTunes namespace
pub fn handle_episode<R: BufRead>(element: Element<R>) -> Option<Episode> {
    element.content_as_text().and_then(|n| n.parse().ok()).map(|number| Episode {
        display: element.attr_value("display"),
        number,
    })
}

// Handles <link>
pub(crate) fn handle_link<R: BufRead>(target: Option<LinkTarget>, element: Element<R>) -> Option<Link> {
    element.content_as_text().map(|s| {
        let mut link = Link::new(s, element.xml_base.as_ref());
        link.target = target;
        link
    })
}

// Handles season elements in the Podcast or iTunes namespace
pub fn handle_season<R: BufRead>(element: Element<R>) -> Option<Season> {
    element.content_as_text().and_then(|n| n.parse().ok()).map(|number| Season {
        name: element.attr_value("name"),
        number,
    })
}

/// Handles date/time
pub(crate) fn handle_timestamp<R: BufRead>(parser: &Parser, element: Element<R>) -> Option<ZonedDateTime> {
    if let Some(text) = element.content_as_text() {
        parser.parse_timestamp(&text)
    } else {
        None
    }
}
