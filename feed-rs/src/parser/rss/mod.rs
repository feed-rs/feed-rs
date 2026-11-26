use crate::model::Text;
use crate::parser::ParseFeedResult;
use crate::xml::Element;
use std::io::BufRead;

pub mod rss0;
pub mod rss1;
pub mod rss2;

/// Handles <content:encoded>
pub(crate) fn handle_encoded<R: BufRead>(element: Element<R>) -> ParseFeedResult<Option<Text>> {
    Ok(element.content_as_text().map(Text::html))
}

// Handles <title>, <description> etc
pub(crate) fn handle_text<R: BufRead>(element: Element<R>) -> Option<Text> {
    element.content_as_text().map(Text::new)
}
