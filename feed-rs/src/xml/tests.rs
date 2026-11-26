use crate::util::test;
use std::error::Error;
use std::io::BufRead;

use super::*;

pub(crate) type TestResult = Result<(), Box<dyn Error>>;

fn handle_book<R: BufRead>(book: Element<R>) -> TestResult {
    // Iterate over the children of the book
    let mut count = 0;
    for child in book.children() {
        let child = child?;
        match child.name.as_str() {
            "author" => {
                count += 1;
                assert_eq!(child.content_as_literal_markup()?.unwrap(), "Gambardella, Matthew");
            }
            "title" => {
                count += 1;
                assert_eq!(child.content_as_literal_markup()?.unwrap(), "XML Developer's Guide");
            }
            "nest1" => {
                handle_nest1(child)?;
            }
            "empty1" | "empty2" => {
                assert!(child.content_as_literal_markup()?.is_none());
            }
            _ => panic!("Unexpected child node: {}", child.name),
        }
    }

    // Should have found two elements
    assert_eq!(count, 2);

    Ok(())
}

fn handle_catalog<R: BufRead>(catalog: Element<R>) -> TestResult {
    // Iterate over the children of the catalog
    let mut count = 0;
    for child in catalog.children() {
        let child = child?;
        // First child should be book
        assert_eq!(child.name, "book");

        // Should have an id attribute
        assert!(child.attributes.iter().any(|attr| &attr.name == "id" && &attr.value == "bk101"));

        // Should only have a single child at this level
        count += 1;

        // Handle the book
        handle_book(child)?;
    }
    assert_eq!(count, 1);

    Ok(())
}

fn handle_nest1<R: BufRead>(nest1: Element<R>) -> TestResult {
    // Should have a single child called "nest2"
    let mut count = 0;
    for child in nest1.children() {
        let child = child?;
        // First child should be nest2
        assert_eq!(child.name, "nest2");

        // It should have the expected text
        assert_eq!(child.content_as_literal_markup()?.unwrap(), "Nested");

        // Should only have a single child at this level
        count += 1;
    }
    assert_eq!(count, 1);

    Ok(())
}

#[test]
fn test_iterate_stream() -> TestResult {
    let test_data = test::fixture_as_string("xml/xml_sample_1.xml");

    // Root element should be "catalog"
    let source = ElementSource::new(test_data.as_bytes(), None)?;
    let catalog = source.root()?.unwrap();
    assert_eq!(catalog.name, "catalog");
    handle_catalog(catalog)?;

    Ok(())
}

// TODO expand test coverage (zero children, empty elements etc)
#[test]
fn test_children_as_string() -> TestResult {
    let test_data = test::fixture_as_string("xml/xml_sample_2.xml");

    // Root element should be "catalog"
    let source = ElementSource::new(test_data.as_bytes(), None)?;
    let catalog = source.root()?.unwrap();
    assert_eq!(catalog.name, "catalog");

    // Next element should be "book"
    let mut children = catalog.children();
    let book = children.next().unwrap()?;
    assert_eq!(book.name, "book");

    // Next element should be "content:encoded"
    let encoded = children.next().unwrap()?;
    assert_eq!(NS::Content, encoded.namespace);
    assert_eq!(encoded.name, "encoded");
    let text = encoded.content_as_literal_markup()?.unwrap();
    assert_eq!(text, "<p>10 km, 21.9072&deg; East, 37.102&deg; North. </p>");

    Ok(())
}

// Verifies the XML decoder handles the various encodings detailed in the RSS2 best practices guide (https://www.rssboard.org/rss-profile#data-types-characterdata)
#[test]
fn test_rss_decoding() -> TestResult {
    let tests = vec![
        ("<title>The &#x26;amp; entity</title>", "The &amp; entity"),
        ("<title>AT&#x26;T</title>", "AT&T"),
        ("<title>Bill &#x26; Ted's Excellent Adventure</title>", "Bill & Ted's Excellent Adventure"),
        ("<title>The &#x26;amp; entity</title>", "The &amp; entity"),
        ("<title>I &#x3C;3 Phil Ringnalda</title>", "I <3 Phil Ringnalda"),
        ("<title>A &#x3C; B</title>", "A < B"),
        ("<title>A&#x3C;B</title>", "A<B"),
        ("<title>Nice &#x3C;gorilla&#x3E;, what's he weigh?</title>", "Nice <gorilla>, what's he weigh?"),
    ];
    for (xml, expected) in tests {
        let source = ElementSource::new(xml.as_bytes(), None)?;
        let title = source.root()?.unwrap();
        let parsed = title.content_as_literal_markup()?;
        assert_eq!(parsed, Some(expected.to_string()));
    }

    Ok(())
}

// Verifies the XML parser handles the xml:base schema
#[test]
fn test_xml_unescape_attrib() -> TestResult {
    let xml = r#"
        <feed xmlns="http://www.w3.org/2005/Atom">
            <link rel="self"
                  href="https://www.reddit.com/search.rss?q=site%3Akevincox.ca&amp;restrict_sr=&amp;sort=new&amp;t=all"
                  type="application/atom+xml"/>
        </feed>
    "#;

    let source = ElementSource::new(xml.as_bytes(), None)?;
    let root = source.root()?.unwrap();
    let link = root.children().next().unwrap()?;
    let href = link.attributes.iter().find(|a| a.name == "href").unwrap();
    assert_eq!(href.value, "https://www.reddit.com/search.rss?q=site%3Akevincox.ca&restrict_sr=&sort=new&t=all");

    Ok(())
}

// Verifies decoding of ISO 8859 content works correctly
#[test]
fn test_iso8859_decode() -> TestResult {
    let xml = test::fixture_as_raw("xml/xml_iso8859.xml");
    let source = ElementSource::new(xml.as_slice(), None)?;
    let root = source.root()?.unwrap();
    let item = root.children().next().unwrap()?;

    let mut elements = item.children();

    let title = elements.next().unwrap()?.content_as_literal_markup()?.unwrap();
    assert_eq!(title, "Digitalministerium: Neue Glasfaserförderung mit Schnellkasse");

    let expected = "Ab April soll es wieder Förderung für den Ausbau von Glasfaser geben.";

    let description = elements.next().unwrap()?.content_as_literal_markup()?.unwrap();
    assert_eq!(description, expected);

    let cdata = elements.next().unwrap()?.content_as_literal_markup()?.unwrap();
    assert_eq!(cdata, expected);

    // The nested XML (or HTML in feeds) breaks the decoding
    let nested = elements.next().unwrap()?.content_as_literal_markup()?.unwrap();
    assert_eq!(nested, format!("<p>{}</p>", expected));

    Ok(())
}

// Verifies we handle unescaped content correctly
#[test]
fn test_content_as_literal_markup() -> TestResult {
    let xml = test::fixture_as_raw("xml/xml_escape_parsing.xml");
    let source = ElementSource::new(xml.as_slice(), None)?;
    let root = source.root()?.unwrap();
    let mut elements = root.children();

    // Test returning unescaped content
    check_content_as_literal_markup(&mut elements, "text", "Plain text")?;
    check_content_as_literal_markup(&mut elements, "text-escaped", "Some <b>bold</b> text")?;
    check_content_as_literal_markup(&mut elements, "text-unescaped", "Some <b>bold</b> text")?;
    check_content_as_literal_markup(&mut elements, "text-in-sub", "<p>text</p>")?;
    check_content_as_literal_markup(&mut elements, "text-in-sub-escaped", "<p>text</p>")?;
    check_content_as_literal_markup(&mut elements, "text-cdata", "Some text in Some <html></html> in cdata with a suffix")?;
    check_content_as_literal_markup(&mut elements, "text-in-cdata", "Some <html></html> in cdata")?;

    Ok(())
}

fn check_content_as_literal_markup(elements: &mut ElementIter<&[u8]>, name: &str, value: &str) -> TestResult {
    let element = elements.next().unwrap()?;
    assert_eq!(element.name, name);
    assert_eq!(element.content_as_literal_markup()?.unwrap(), value);

    Ok(())
}
