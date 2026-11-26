use feed_rs::parser;

// Validate that a parser which failed on a previous run is reusable on the next parse
#[test]
fn test_parser_reuse_after_media_error() {
    // The invalid media:title type triggers an error between "entry_begin" and "entry_end"
    let broken_source = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:media="http://search.yahoo.com/mrss/">
    <channel>
        <title>Broken feed</title>
        <link>http://example.org/</link>
        <description>MediaRSS element which fails to parse</description>
        <item>
            <title>Item 1</title>
            <media:group><media:title type="bogus">broken</media:title></media:group>
        </item>
    </channel>
</rss>"#;

    let valid_source = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:media="http://search.yahoo.com/mrss/">
    <channel>
        <title>Valid feed</title>
        <link>http://example.org/</link>
        <description>Valid MediaRSS element</description>
        <item>
            <title>Item 1</title>
            <media:content url="http://example.org/image.png" type="image/png"/>
        </item>
    </channel>
</rss>"#;

    let parser = parser::Builder::new().build();

    // First parse is expected to fail and the second parse must not panic
    let broken = parser.parse(broken_source.as_bytes());
    assert!(broken.is_err());

    let feed = parser.parse(valid_source.as_bytes()).unwrap();
    assert_eq!(1, feed.entries.len());
    let media = &feed.entries[0].media;
    assert_eq!(1, media.len());
    assert_eq!(1, media[0].content.len());
}

// Deeply nested media:group elements are reported as an error rather than exhausting the stack
#[test]
fn test_deeply_nested_media_group() {
    // Namespace declared once on the root so the tokenizer itself does not bound the nesting
    const NESTING: usize = 5000;
    let mut source = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0" xmlns:media="http://search.yahoo.com/mrss/"><channel><title>Deep</title><link>http://example.org/</link><description>Deeply nested media groups</description><item><title>Item 1</title>"#,
    );
    source.push_str(&"<media:group>".repeat(NESTING));
    source.push_str(&"</media:group>".repeat(NESTING));
    source.push_str("</item></channel></rss>");

    // Must be an error (depth limit) rather than a stack overflow
    let result = parser::parse(source.as_bytes());
    assert!(result.is_err());
}
