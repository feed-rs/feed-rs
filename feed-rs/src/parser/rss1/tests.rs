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
        .title(Text::new("Feed title".to_owned()))
        .link(Link::new("http://www.example.com/main.html".to_owned()))
        .description(Text::new("Site description".to_owned()))
        .updated(actual.updated)    // not present in the test data
        .entry(Entry::default()
            .id("e759cc25dc488d95ec723f6553e10a4e")     // hash of the link
            .updated(entry0.updated)                    // not present in the test data
            .title(Text::new("記事1のタイトル".to_owned()))
            .link(Link::new("記事1のURL".to_owned()))
            .summary(Text::new("記事1の内容".to_owned())))
        .entry(Entry::default()
            .id("2c802e03278c5c9b7c6f3690f69f7570")     // hash of the link
            .updated(entry1.updated)                    // not present in the test data
            .title(Text::new("記事2のタイトル".to_owned()))
            .link(Link::new("記事2のURL".to_owned()))
            .summary(Text::new("記事2の内容".to_owned())));

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
        .title(Text::new("XML.com".to_owned()))
        .link(Link::new("http://xml.com/pub".to_owned()))
        .description(Text::new("XML.com features a rich mix of information and services\n            for the XML community.".to_owned()))
        .logo(Image::new("http://xml.com/universal/images/xml_tiny.gif".to_owned())
            .link("http://www.xml.com")
            .title("XML.com"))
        .updated(actual.updated)    // not present in the test data
        .entry(Entry::default()
            .id("4e02e6b5e0c197983a5347308272fa20")     // hash of the link
            .updated(entry0.updated)            // not present in the test data
            .title(Text::new("Processing Inclusions with XSLT".to_owned()))
            .link(Link::new("http://xml.com/pub/2000/08/09/xslt/xslt.html".to_owned()))
            .summary(Text::new("Processing document inclusions with general XML tools can be\n            problematic. This article proposes a way of preserving inclusion\n            information through SAX-based processing.".to_owned())))
        .entry(Entry::default()
            .id("82feaa3d819de1de3e5ed0eb59b53706")     // hash of the link
            .updated(entry1.updated)            // not present in the test data
            .title(Text::new("Putting RDF to Work".to_owned()))
            .link(Link::new("http://xml.com/pub/2000/08/09/rdfdb/index.html".to_owned()))
            .summary(Text::new("Tool and API support for the Resource Description Framework\n            is slowly coming of age. Edd Dumbill takes a look at RDFDB,\n            one of the most exciting new RDF toolkits.".to_owned())));

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
        .title(Text::new("Meerkat".to_owned()))
        .link(Link::new("http://meerkat.oreillynet.com".to_owned()))
        .description(Text::new("Meerkat: An Open Wire Service".to_owned()))
        .logo(Image::new("http://meerkat.oreillynet.com/icons/meerkat-powered.jpg".to_owned())
            .link("http://meerkat.oreillynet.com")
            .title("Meerkat Powered!"))
        .updated(actual.updated)    // not present in the test data
        .entry(Entry::default()
            .id("3c8aa6d4cb85520f531b5a9c08498d04")     // hash of the link
            .updated(entry0.updated)            // not present in the test data
            .title(Text::new("XML: A Disruptive Technology".to_owned()))
            .link(Link::new("http://c.moreover.com/click/here.pl?r123".to_owned()))
            .summary(Text::new("XML is placing increasingly heavy loads on the existing technical\n            infrastructure of the Internet.".to_owned())));

    // Check
    assert_eq!(actual, expected);
}
