use crate::parser;
use crate::model::{Feed, Text, Link, Entry, Image};

use crate::util::test;

// Example from the web
#[test]
fn test_example_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_1.0_example_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let entry0 = actual.entries.get(0).unwrap();
    let entry1 = actual.entries.get(1).unwrap();
    let expected = Feed::default()
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("Feed title".into()))
        .link(Link::new("http://www.example.com/main.html".into()))
        .description(Text::new("Site description".into()))
        .updated(actual.updated)    // not present in the test data
        .entry(Entry::default()
            .id("7d61c42a2d8ecf2289e789e1fb2035d1")     // hash of the link
            .updated(entry0.updated)                    // not present in the test data
            .title(Text::new("記事1のタイトル".into()))
            .link(Link::new("記事1のURL".into()))
            .summary(Text::new("記事1の内容".into())))
        .entry(Entry::default()
            .id("e342c1b080da9ffbfd10c0a6ba49395f")     // hash of the link
            .updated(entry1.updated)                    // not present in the test data
            .title(Text::new("記事2のタイトル".into()))
            .link(Link::new("記事2のURL".into()))
            .summary(Text::new("記事2の内容".into())));

    // Check
    assert_eq!(actual, expected);
}

// Example 1 from the spec at https://validator.w3.org/feed/docs/rss1.html
#[test]
fn test_spec_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_1.0_spec_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let entry0 = actual.entries.get(0).unwrap();
    let entry1 = actual.entries.get(1).unwrap();
    let expected = Feed::default()
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("XML.com".into()))
        .link(Link::new("http://xml.com/pub".into()))
        .description(Text::new("\n            XML.com features a rich mix of information and services\n            for the XML community.\n        ".into()))
        .logo(Image::new("http://xml.com/universal/images/xml_tiny.gif".into())
            .link("http://www.xml.com")
            .title("XML.com"))
        .updated(actual.updated)    // not present in the test data
        .entry(Entry::default()
            .id("958983927af7075ad55f6d2c9b0c24b2")     // hash of the link
            .updated(entry0.updated)            // not present in the test data
            .title(Text::new("Processing Inclusions with XSLT".into()))
            .link(Link::new("http://xml.com/pub/2000/08/09/xslt/xslt.html".into()))
            .summary(Text::new("\n\n            Processing document inclusions with general XML tools can be\n            problematic. This article proposes a way of preserving inclusion\n            information through SAX-based processing.\n        ".into())))
        .entry(Entry::default()
            .id("54f62e9fe8901546d7e25c0827e1b8e8")     // hash of the link
            .updated(entry1.updated)            // not present in the test data
            .title(Text::new("Putting RDF to Work".into()))
            .link(Link::new("http://xml.com/pub/2000/08/09/rdfdb/index.html".into()))
            .summary(Text::new("\n            Tool and API support for the Resource Description Framework\n            is slowly coming of age. Edd Dumbill takes a look at RDFDB,\n            one of the most exciting new RDF toolkits.\n        ".into())));

    // Check
    assert_eq!(actual, expected);
}

// Example 2 from the spec at https://validator.w3.org/feed/docs/rss1.html
#[test]
fn test_spec_2() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_1.0_spec_2.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let entry0 = actual.entries.get(0).unwrap();
    let expected = Feed::default()
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("Meerkat".into()))
        .link(Link::new("http://meerkat.oreillynet.com".into()))
        .description(Text::new("Meerkat: An Open Wire Service".into()))
        .logo(Image::new("http://meerkat.oreillynet.com/icons/meerkat-powered.jpg".into())
            .link("http://meerkat.oreillynet.com")
            .title("Meerkat Powered!"))
        .updated(actual.updated)    // not present in the test data
        .entry(Entry::default()
            .id("acf7c86547d5d594af6d8f3327e84b06")     // hash of the link
            .updated(entry0.updated)            // not present in the test data
            .title(Text::new("XML: A Disruptive Technology".into()))
            .link(Link::new("http://c.moreover.com/click/here.pl?r123".into()))
            .summary(Text::new("\n            XML is placing increasingly heavy loads on the existing technical\n            infrastructure of the Internet.\n        ".into())));

    // Check
    assert_eq!(actual, expected);
}
