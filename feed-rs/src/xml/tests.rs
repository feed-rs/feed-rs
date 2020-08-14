use std::io::BufRead;

use crate::util::test;

use super::*;

type Result = std::result::Result<(), XmlError>;

fn handle_book<R: BufRead>(book: Element<R>) -> Result {
    // Iterate over the children of the book
    let mut count = 0;
    for child in book.children() {
        let child = child?;
        match child.name.as_str() {
            "author" => {
                count += 1;
                assert_eq!(child.child_as_text()?.unwrap(), "Gambardella, Matthew");
            }
            "title" => {
                count += 1;
                assert_eq!(child.child_as_text()?.unwrap(), "XML Developer's Guide");
            }
            "nest1" => {
                handle_nest1(child)?;
            }
            "empty1" | "empty2" => {
                assert!(child.child_as_text()?.is_none());
            }
            _ => panic!("Unexpected child node: {}", child.name)
        }
    }

    // Should have found two elements
    assert_eq!(count, 2);

    Ok(())
}

fn handle_catalog<R: BufRead>(catalog: Element<R>) -> Result {
    // Iterate over the children of the catalog
    let mut count = 0;
    for child in catalog.children() {
        let child = child?;
        // First child should be book
        assert_eq!(child.name, "book");

        // Should have an id attribute
        assert!(child.attributes.iter().find(|attr| &attr.name == "id" && &attr.value == "bk101").is_some());

        // Should only have a single child at this level
        count += 1;

        // Handle the book
        handle_book(child)?;
    }
    assert_eq!(count, 1);

    Ok(())
}

fn handle_nest1<R: BufRead>(nest1: Element<R>) -> Result {
    // Should have a single child called "nest2"
    let mut count = 0;
    for child in nest1.children() {
        let child = child?;
        // First child should be nest2
        assert_eq!(child.name, "nest2");

        // It should have the expected text
        assert_eq!(child.child_as_text()?.unwrap(), "Nested");

        // Should only have a single child at this level
        count += 1;
    }
    assert_eq!(count, 1);

    Ok(())
}

#[test]
fn test_iterate_stream() -> Result {
    let test_data = test::fixture_as_string("xml_sample_1.xml");

    // Root element should be "catalog"
    let source = ElementSource::new(test_data.as_bytes());
    let catalog = source.root()?.unwrap();
    assert_eq!(catalog.name, "catalog");
    handle_catalog(catalog)?;

    Ok(())
}

// TODO expand test coverage (zero children, empty elements etc)
#[test]
fn test_children_as_string() -> Result {
    let test_data = test::fixture_as_string("xml_sample_2.xml");

    // Root element should be "catalog"
    let source = ElementSource::new(test_data.as_bytes());
    let catalog = source.root()?.unwrap();
    assert_eq!(catalog.name, "catalog");

    // Next element should be "book"
    let mut children = catalog.children();
    let book = children.next().unwrap()?;
    assert_eq!(book.name, "book");
    let expected = "\n        <author>Gambardella, Matthew</author>\n        <title>XML Developer's Guide</title>\n    ";
    assert_eq!(book.children_as_string()?.unwrap(), expected);

    // Next element should be "content:encoded"
    let encoded = children.next().unwrap()?;
    assert_eq!(&NS::Content, encoded.namespace.as_ref().unwrap());
    assert_eq!(encoded.name, "encoded");
    let text = encoded.children_as_string()?.unwrap();
    assert_eq!(text, "<p>10 km, 21.9072&deg; East, 37.102&deg; North. </p>");

    Ok(())
}

// Verifies the XML decoder handles the various encodings detailed in the RSS2 best practices guide (https://www.rssboard.org/rss-profile#data-types-characterdata)
#[test]
fn test_rss_decoding() -> Result {
    let tests = vec!(
        ("<title>AT&#x26;T</title>", "AT&T"),
        ("<title>Bill &#x26; Ted's Excellent Adventure</title>", "Bill & Ted's Excellent Adventure"),
        ("<title>The &#x26;amp; entity</title>", "The &amp; entity"),
        ("<title>I &#x3C;3 Phil Ringnalda</title>", "I <3 Phil Ringnalda"),
        ("<title>A &#x3C; B</title>", "A < B"),
        ("<title>A&#x3C;B</title>", "A<B"),
        ("<title>Nice &#x3C;gorilla&#x3E;, what's he weigh?</title>", "Nice <gorilla>, what's he weigh?"),
    );
    for (xml, expected) in tests {
        let source = ElementSource::new(xml.as_bytes());
        let title = source.root()?.unwrap();
        let parsed = title.children_as_string()?.unwrap();
        assert_eq!(expected, parsed);
    }

    Ok(())
}
