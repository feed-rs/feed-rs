use crate::parser::{ParseFeedResult, ParseFeedError, ParseErrorKind};
use chrono::{DateTime, Utc};
use regex::Regex;
use uuid::Uuid;

lazy_static! {
    // Initialise the set of regular expressions we use to clean up broken dates
    // Feeds may not comply with the specification in various ways (https://tools.ietf.org/html/rfc2822#page-14)
    static ref RFC2822_FIXES: Vec<(Regex, &'static str)> = {
        vec!(
            // RFC 2822 mandates a +/- 4 digit offset, or UT/GMT (obsolete) but feeds have "UTC" or "-0000"
            // Suffixes that are not handled by the parser are trimmed and replaced with the corresponding value timezone.
            (Regex::new("(UTC|-0000$)").unwrap(), "+0000"),

            // The short weekday can be wrong e.g. "Wed, 25 Aug 2012" was actually a Saturday - https://www.timeanddate.com/calendar/monthly.html?year=2012&month=8
            // or it can be something other than a short weekday name e.g. "Thurs, 13 Jul 2011 07:38:00 GMT"
            // As its extraneous, we just remove it
            (Regex::new("(Sun|Mon|Tue|Wed|Thu|Fri|Sat)[a-z]*, ").unwrap(), ""),

            // Long month names are not allowed, so replace them with short
            (Regex::new("(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*").unwrap(), "$1"),

            // Some timestamps have an hours component adjusted by 24h, while not adjusting the day so we just reset to start of day
            (Regex::new(" 24:").unwrap(), " 00:"),

            // Single digit hours are padded
            (Regex::new(" ([0-9]):").unwrap(), " 0${1}:"),
        )
    };
}

/// Parses a timestamp from an RSS2 feed.
/// This should be an RFC-2822 formatted timestamp but we need a bunch of fixes / workarounds for the generally broken stuff we find on the internet
pub fn timestamp_rfc2822_lenient(text: &str) -> ParseFeedResult<DateTime<Utc>> {
    // Curiously, we see RFC-3339 dates in RSS 2 feeds so try that first
    if let Ok(ts) = timestamp_rfc3339(text) {
        return Ok(ts);
    }

    // Clean the input string by applying each of the regex fixes
    let mut text = text.trim().to_string();
    for (regex, replacement) in RFC2822_FIXES.iter() {
        text = regex.replace(&text, *replacement).to_string();
    }

    let parsed = DateTime::parse_from_rfc2822(&text)
        .map(|t| t.with_timezone(&Utc))
        .map_err(|pe| ParseFeedError::ParseError(ParseErrorKind::InvalidDateTime(Box::new(pe))));

    if parsed.is_err() {
        println!("**************** Unable to parse {}", text);
    }
    parsed
}

/// Parses a timestamp from an Atom or JSON feed
/// This should be an RFC-3339 formatted timestamp
pub fn timestamp_rfc3339(text: &str) -> ParseFeedResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text.trim())
        .map(|t| t.with_timezone(&Utc))
        .map_err(|pe| ParseFeedError::ParseError(ParseErrorKind::InvalidDateTime(Box::new(pe))))
}

/// Generates a new UUID.
pub fn uuid_gen() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    type Result = std::result::Result<(), ParseFeedError>;

    // Verify we can parse non-spec compliant date strings
    // Regression tests for https://github.com/feed-rs/feed-rs/issues/7
    #[test]
    fn test_timestamp_rss2() -> Result {
        let tests = vec!(
            //
            ("26 August 2019 10:00:00 +0000", Utc.ymd(2019, 8, 26).and_hms_milli(10, 0, 0, 0)),

            // UTC is not a valid timezone in RFC-2822
            ("Mon, 01 Jan 0001 00:00:00 UTC", Utc.ymd(1, 1, 1).and_hms_milli(0, 0, 0, 0)),

            // -0000 is not considered a timezone in the parser
            ("Wed, 22 Jan 2020 10:58:02 -0000", Utc.ymd(2020, 1, 22).and_hms_milli(10, 58, 02, 0)),

            // The 25th of August 2012 was a Saturday, not a Wednesday
            ("Wed, 25 Aug 2012 03:25:42 GMT", Utc.ymd(2012, 8, 25).and_hms_milli(3, 25, 42, 0)),

            // Long month names are not allowed
            ("2 September 2019 20:00:00 +0000", Utc.ymd(2019, 9, 2).and_hms_milli(20, 0, 0, 0)),

            // RSS2 should be RFC-2822 but we get Atom/RFC-3339 formats
            ("2016-10-01T00:00:00+10:00", Utc.ymd(2016, 9, 30).and_hms_milli(14,0,0, 0)),

            // Single digit hours should be padded
            ("24 Sep 2013 1:27 PDT", Utc.ymd(2013, 9, 24).and_hms_milli(8, 27, 0, 0)),

            // Consider an invalid hour specification as start-of-day
            ("5 Jun 2017 24:05 PDT", Utc.ymd(2017, 6, 5).and_hms_milli(7, 5, 0, 0)),

            // TODO what date format is this? https://feeds.feedburner.com/oshogbo : "Dec. 9, 2019, 6:11 p.m."
        );

        for (source, expected) in tests {
            let parsed = timestamp_rfc2822_lenient(source)?;
            assert_eq!(parsed, expected);
        }

        Ok(())
    }
}
