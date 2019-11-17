use crate::model::Feed;
use crate::util::element_source::Element;
use std::io::Read;

use crate::parser;
use crate::parser::rss2;

#[cfg(test)]
mod tests;

/// Parses an RSS 0.9x feed into our model
pub fn parse<R: Read>(root: Element<R>) -> parser::Result<Feed> {
    // The 0.9x models are upward compatible with 2.x so we just delegate to that parser
    rss2::parse(root)
}
