use crate::xml::{NameValue, XmlError, XmlResult};
use url::Url;

pub(crate) struct XmlBase {
    base_uris: Vec<(u32, Url)>,
}

// Maximum length for a base URI
//
// A little bit of research turns up:
// - IBM setting "XML element name, attribute name, or prefix name" to 1000 in DB2 - https://www.ibm.com/docs/en/db2-as-a-service?topic=reference-sql-xml-limits
// - Stack Overflow suggesting 2048 - https://stackoverflow.com/questions/29458445/what-is-a-safe-maximum-length-a-segment-in-a-url-path-should-be
// - Google Search suggesting Edge is 2083, Firefix is 64k, Chrome is 2mb
//
// A 1kb base URL is extremely long, given other base URIs will be appended to it, then the resource to that.
// Feels like this is already plenty, but we will, of course, wait for the bug report saying its not long enough
static MAX_XML_BASE_URI_LEN: usize = 1024;

impl XmlBase {
    pub(crate) fn new(maybe_root: Option<&str>) -> Result<Self, XmlError> {
        let mut base_uris = Vec::new();
        if let Some(root_uri) = maybe_root {
            let uri = Url::parse(root_uri)?;
            base_uris.push((0, uri));
        }

        Ok(XmlBase { base_uris })
    }

    // Fetches the currently active xml-base
    pub(crate) fn fetch(&self) -> Option<Url> {
        // Return the last entry on the stack if it exists
        self.base_uris.last().map(|(_, uri)| uri.clone())
    }

    // Unwinds the stack of base URLs to the reflect the current parse depth
    pub(crate) fn pop(&mut self, current_depth: u32) {
        // Pop any URIs off the stack if they are deeper than our new depth
        while !self.base_uris.is_empty() {
            let (depth, _) = self.base_uris.last().unwrap();
            if depth > &current_depth {
                self.base_uris.pop();
            } else {
                break;
            }
        }
    }

    // Pushes an updated xml-base on to the stack as required
    pub(crate) fn push(&mut self, current_depth: u32, attributes: &[NameValue]) -> XmlResult<()> {
        // Find the xml-base attribute
        let xml_base = attributes.iter().find(|nv| nv.name == "xml:base").map(|nv| &nv.value);

        if let Some(new_base) = xml_base {
            let maybe_new_base_uri = match Url::parse(new_base) {
                Ok(uri) => {
                    // It is an absolute URI so push it straight on to the stack
                    Some(uri)
                }
                Err(url::ParseError::RelativeUrlWithoutBase) => {
                    // Try and form a new URL and push it to the stack
                    if let Some((_, last)) = self.base_uris.last() {
                        last.join(new_base).ok()
                    } else {
                        None
                    }
                }
                Err(e) => return Err(XmlError::Url { e }),
            };

            if let Some(new_base_uri) = maybe_new_base_uri {
                // Check that the new base URI is valid
                if new_base_uri.as_ref().len() <= MAX_XML_BASE_URI_LEN {
                    self.base_uris.push((current_depth, new_base_uri));
                } else {
                    return Err(XmlError::ValidationError {
                        msg: format!("Base URI too long: {}", new_base_uri),
                    });
                }
            }
        }

        // No xml-base found
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::xml::{Element, ElementSource, XmlError};
    use std::io::BufRead;
    use url::Url;

    fn assert_title_bases<R: BufRead>(feed: Element<R>, expected: Vec<&str>) -> crate::xml::tests::TestResult {
        // Find the actual title bases
        let mut title_bases = Vec::new();
        for entry in feed.children() {
            let entry = entry?;
            for title in entry.children() {
                let title = title?;
                title_bases.push(title.xml_base.unwrap());
            }
        }

        // Verify the are as we expect
        let expected = expected.iter().map(|uri| Url::parse(uri).unwrap()).collect::<Vec<Url>>();
        assert_eq!(expected, title_bases);

        Ok(())
    }

    // Verifies the XML parser handles the xml:base schema
    #[test]
    fn test_xml_base() -> crate::xml::tests::TestResult {
        let xml = r#"
        <feed version="0.3" xml:base="http://1.example.com/">
        <entry>
        <title type="application/xhtml+xml" xml:base="test/"><div xmlns="http://www.w3.org/1999/xhtml">Example <a href="test.html">test</a></div></title>
        </entry>
        <entry xml:base="http://2.example.com/">
        <title type="application/xhtml+xml" xml:base="test1/test2"><div xmlns="http://www.w3.org/1999/xhtml">Example <a href="test.html">test</a></div></title>
        </entry>
        <entry xml:base="http://3.example.com/">
        <title type="application/xhtml+xml" xml:base="/test3"><div xmlns="http://www.w3.org/1999/xhtml">Example <a href="test.html">test</a></div></title>
        </entry>
        </feed>"#;

        let source = ElementSource::new(xml.as_bytes(), None)?;
        let feed = source.root()?.unwrap();
        assert_eq!(&Url::parse("http://1.example.com/")?, feed.xml_base.as_ref().unwrap());

        assert_title_bases(
            feed,
            vec!["http://1.example.com/test/", "http://2.example.com/test1/test2", "http://3.example.com/test3"],
        )?;

        Ok(())
    }

    // Verifies the XML parser handles the xml:base schema
    #[test]
    fn test_xml_base_header() -> crate::xml::tests::TestResult {
        let xml = r#"
        <feed version="0.3" xmlns="http://purl.org/atom/ns#" xml:base="feed/base/">
        <entry>
          <title type="application/xhtml+xml" xml:base="../"><div xmlns="http://www.w3.org/1999/xhtml">Example <a href="test.html">test</a></div></title>
        </entry>
        <entry xml:base="/feed2/entry/base/">
          <title type="application/xhtml+xml" xml:base="../"><div xmlns="http://www.w3.org/1999/xhtml">Example <a href="test.html">test</a></div></title>
        </entry>
        </feed>"#;

        let source = ElementSource::new(xml.as_bytes(), Some("http://example.com"))?;
        let feed = source.root()?.unwrap();
        assert_eq!(&Url::parse("http://example.com/feed/base/")?, feed.xml_base.as_ref().unwrap());

        assert_title_bases(feed, vec!["http://example.com/feed/", "http://example.com/feed2/entry/"])?;

        Ok(())
    }

    // Verifies we catch XML bases that exceed the length limit
    #[test]
    fn test_xml_base_invalid_len() -> crate::xml::tests::TestResult {
        // Try with an initial base that exceeds the limit
        let too_long = format!("http://example.com/?q={}", "a".repeat(1024));
        let xml = format!(r#" <feed xml:base="{}"> </feed>"#, too_long);
        let source = ElementSource::new(xml.as_bytes(), None)?;
        let root = source.root();
        match root {
            // Nothing required, we expect this
            Err(XmlError::ValidationError { msg }) => {}

            // Anything else should be an error
            _ => panic!("Expected validation error, got {:?}", root),
        }

        // Try with a feed that builds up a relative entry that is too long
        let fragment1 = "a".repeat(512);
        let fragment2 = "b".repeat(512);
        let xml = format!(
            r#" <feed xml:base="http://example.com/{}/"><entry xml:base="{}"></entry></feed>"#,
            fragment1, fragment2
        );
        let source = ElementSource::new(xml.as_bytes(), None)?;
        let root = source.root()?.unwrap();
        let child = root.children().next().unwrap();
        match child {
            // Nothing required, we expect this
            Err(XmlError::ValidationError { msg }) => {}

            // Anything else should be an error
            _ => panic!("Expected validation error, got {:?}", root),
        }

        Ok(())
    }
}
