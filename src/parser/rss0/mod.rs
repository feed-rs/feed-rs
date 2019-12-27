use std::io::Read;

use crate::model::Feed;
use crate::parser::{ParseFeedResult, rss2};
use crate::util::element_source::Element;

#[cfg(test)]
mod tests;

/// Parses an RSS 0.9x feed into our model
pub fn parse<R: Read>(root: Element<R>) -> ParseFeedResult<Feed> {
    // The 0.9x models are upward compatible with 2.x so we just delegate to that parser
    rss2::parse(root)
}
