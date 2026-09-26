use chrono::{TimeZone, Utc};

use super::*;

// Verify we can parse non-spec compliant date strings
// Regression tests for https://github.com/feed-rs/feed-rs/issues/7
#[test]
fn test_timestamp_rss2() {
    let tests = vec![
        //
        ("26 August 2019 10:00:00 +0000", Utc.with_ymd_and_hms(2019, 8, 26, 10, 0, 0).unwrap()),
        // UTC is not a valid timezone in RFC-2822
        ("Mon, 01 Jan 0001 00:00:00 UTC", Utc.with_ymd_and_hms(1, 1, 1, 0, 0, 0).unwrap()),
        // -0000 is not considered a timezone in the parser
        ("Wed, 22 Jan 2020 10:58:02 -0000", Utc.with_ymd_and_hms(2020, 1, 22, 10, 58, 2).unwrap()),
        // The 25th of August 2012 was a Saturday, not a Wednesday
        ("Wed, 25 Aug 2012 03:25:42 GMT", Utc.with_ymd_and_hms(2012, 8, 25, 3, 25, 42).unwrap()),
        // Long month names are not allowed
        ("2 September 2019 20:00:00 +0000", Utc.with_ymd_and_hms(2019, 9, 2, 20, 0, 0).unwrap()),
        // RSS2 should be RFC-2822 but we get Atom/RFC-3339 formats
        ("2016-10-01T00:00:00+10:00", Utc.with_ymd_and_hms(2016, 9, 30, 14, 0, 0).unwrap()),
        // Single digit hours should be padded
        ("24 Sep 2013 1:27 PDT", Utc.with_ymd_and_hms(2013, 9, 24, 8, 27, 0).unwrap()),
        // Consider an invalid hour specification as start-of-day
        ("5 Jun 2017 24:05 PDT", Utc.with_ymd_and_hms(2017, 6, 5, 7, 5, 0).unwrap()),
        // We even see RFC1123
        ("Tue, 15 Nov 2022 20:15:04 Z", Utc.with_ymd_and_hms(2022, 11, 15, 20, 15, 4).unwrap()),
        // And RFC1123 with languages other than English...
        ("mer, 16 nov 2022 00:38:15 +0100", Utc.with_ymd_and_hms(2022, 11, 15, 23, 38, 15).unwrap()),
    ];

    for (source, expected) in tests {
        let parsed = parse_timestamp_lenient(source).unwrap_or_else(|| panic!("failed to parse {}", source));
        assert_eq!(parsed, expected);
    }
}

#[test]
fn test_timestamp_atom() {
    let tests = vec![
        // properly formated rfc3339 string
        ("2014-12-29T14:53:35+02:00", Utc.with_ymd_and_hms(2014, 12, 29, 12, 53, 35).unwrap()),
        // missing colon in timezone
        ("2014-12-29T14:53:35+0200", Utc.with_ymd_and_hms(2014, 12, 29, 12, 53, 35).unwrap()),
    ];

    for (source, expected) in tests {
        let parsed = parse_timestamp_lenient(source).unwrap_or_else(|| panic!("failed to parse {}", source));
        assert_eq!(parsed, expected);
    }
}

// Verify we can parse NPT times
#[test]
fn test_parse_npt() {
    assert_eq!(parse_npt("12:05:35").unwrap(), Duration::from_secs(12 * 3600 + 5 * 60 + 35));
    assert_eq!(
        parse_npt("12:05:35.123").unwrap(),
        Duration::from_millis(12 * 3600000 + 5 * 60000 + 35 * 1000 + 123)
    );
    assert_eq!(parse_npt("123.45").unwrap(), Duration::from_millis(123450));
}

// Test various forms of email and names in a string
#[test]
fn test_parse_person_name_email() {
    let tests = vec![
        ("First Last <user@example.com>", Some("First Last"), Some("user@example.com")),
        ("First Last [user@example.com]", Some("First Last"), Some("user@example.com")),
        ("First Last (user@example.com)", Some("First Last"), Some("user@example.com")),
        ("<user@example.com> First Last", Some("First Last"), Some("user@example.com")),
        ("[user@example.com] First Last", Some("First Last"), Some("user@example.com")),
        ("(user@example.com) First Last", Some("First Last"), Some("user@example.com")),
        ("First", Some("First"), None),
        (" First", Some("First"), None),
        ("First ", Some("First"), None),
        ("user@example.com", None, Some("user@example.com")),
        (" user@example.com", None, Some("user@example.com")),
        ("user@example.com ", None, Some("user@example.com")),
        (
            "Simon St.Laurent (mailto:simonstl@simonstl.com)",
            Some("Simon St.Laurent"),
            Some("simonstl@simonstl.com"),
        ),
    ];

    for (raw, expected_name, expected_email) in tests {
        let person = parse_person_name_email(raw);

        let expected_name = expected_name.map(|s| s.to_owned());
        assert_eq!(person.name, expected_name, "incorrect name for {}", raw);

        let expected_email = expected_email.map(|s| s.to_owned());
        assert_eq!(person.email, expected_email, "incorrect email for {}", raw);
    }
}
