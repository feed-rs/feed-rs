use uuid::Uuid;
use xml::attribute::OwnedAttribute;
use chrono::{NaiveDateTime, DateTime};
use crate::parser::{ParseFeedResult, ParseFeedError, ParseErrorKind};

pub mod element_source;

/// Returns the value of the first attribute with the given name
pub fn attr_value<'a>(attributes: &'a [OwnedAttribute], name: &str) -> Option<&'a str> {
    attributes.iter()
        .find(|attr| attr.name.local_name == name)
        .map(|attr| attr.value.as_str())
}

/// Parses an RFC-2822 formatted timestamp
pub fn timestamp_from_rfc2822(text: &str) -> ParseFeedResult<NaiveDateTime> {
    DateTime::parse_from_rfc2822(text.trim())
        .map(|t| t.naive_utc())
        .map_err(|pe| ParseFeedError::ParseError(ParseErrorKind::InvalidDateTime(Box::new(pe))))
}

/// Parses an RFC-3339 formatted timestamp
pub fn timestamp_from_rfc3339(text: &str) -> ParseFeedResult<NaiveDateTime> {
    DateTime::parse_from_rfc3339(text.trim())
        .map(|t| t.naive_utc())
        .map_err(|pe| ParseFeedError::ParseError(ParseErrorKind::InvalidDateTime(Box::new(pe))))
}

/// Generates a new UUID.
pub fn uuid_gen() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
pub mod test;
