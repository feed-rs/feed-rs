use crate::model::{Entry, Person, Link, Feed, Text, Generator};
use crate::parser;
use crate::util::test;

// Verify we can parse a more complete example
#[test]
fn test_example_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("atom_example_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::new()
        .title("dive into mark")
        .description(Text::new("A <em>lot</em> of effort\n        went into making this effortless".to_owned())
            .content_type("html"))
        .updated("2005-07-31T12:29:29Z")
        .id("tag:example.org,2003:3")
        .link(Link::new("http://example.org/".to_owned())
            .rel("alternate")
            .media_type("text/html")
            .href_lang("en"))
        .link(Link::new("http://example.org/feed.atom".to_owned())
            .rel("self")
            .media_type("application/atom+xml"))
        .rights(Text::new("Copyright (c) 2003, Mark Pilgrim".to_owned()))
        .generator(Generator::new("Example Toolkit".to_owned())
            .uri("http://www.example.com/")
            .version("1.0"))
        .entry(Entry::new()
            .id("tag:example.org,2003:3.2397")
            .title("Atom draft-07 snapshot")
            .updated("2005-07-31T12:29:29Z")
            .author(Person::new("Mark Pilgrim".to_owned())
                .uri("http://example.org/")
                .email("f8dy@example.com"))
            .link(Link::new("http://example.org/2005/04/02/atom".to_owned())
                .rel("alternate")
                .media_type("text/html"))
            .link(Link::new("http://example.org/audio/ph34r_my_podcast.mp3".to_owned())
                .rel("enclosure")
                .media_type("audio/mpeg")
                .length(1337))
            .contributor(Person::new("Sam Ruby".to_owned()))
            .contributor(Person::new("Joe Gregorio".to_string()))
            .published("2003-12-13T08:29:29-04:00"));

    // Check
    assert_eq!(actual, expected);
}

// Verify we can parse the example contained in the Atom specification
#[test]
fn test_spec_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("atom_spec_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::new()
        .id("urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6")
        .title("Example Feed")
        .link(Link::new("http://example.org/".to_owned())
            .rel("alternate"))
        .updated("2003-12-13T18:30:02Z")
        .author(Person::new("John Doe".to_owned()))
        .entry(Entry::new()
            .id("urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a")
            .title("Atom-Powered Robots Run Amok")
            .updated("2003-12-13T18:30:02Z")
            .summary("Some text.")
            .link(Link::new("http://example.org/2003/12/13/atom03".to_owned())
                .rel("alternate")));

    // Check
    assert_eq!(actual, expected);
}
