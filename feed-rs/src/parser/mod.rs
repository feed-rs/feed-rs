use std::io::{BufRead, BufReader, Read};

use siphasher::sip128::{Hasher128, SipHasher};
use xml::reader as xml_reader;

use crate::model;
use crate::util::attr_value;
use crate::util::element_source::ElementSource;
use std::hash::Hasher;

mod atom;
mod json;
mod rss0;
mod rss1;
mod rss2;

pub(crate) mod util;

pub type ParseFeedResult<T> = std::result::Result<T, ParseFeedError>;

/// An error returned when parsing a feed from a source fails
#[derive(Debug)]
pub enum ParseFeedError {
    // TODO add line number/position
    ParseError(ParseErrorKind),
    // IO error
    IoError(std::io::Error),
    // Underlying issue with JSON (poorly formatted etc)
    JsonSerde(serde_json::error::Error),
    // Underlying issue with XML (poorly formatted etc)
    XmlReader(xml_reader::Error),
}

impl From<serde_json::error::Error> for ParseFeedError {
    fn from(err: serde_json::error::Error) -> Self { ParseFeedError::JsonSerde(err) }
}

impl From<std::io::Error> for ParseFeedError {
    fn from(err: std::io::Error) -> Self { ParseFeedError::IoError(err) }
}

impl From<xml_reader::Error> for ParseFeedError {
    fn from(err: xml_reader::Error) -> Self {
        ParseFeedError::XmlReader(err)
    }
}

/// Underlying cause of the parse failure
#[derive(Debug)]
pub enum ParseErrorKind {
    /// Could not find the expected root element (e.g. "channel" for RSS 2, a JSON node etc)
    NoFeedRoot,
    /// The content type is unsupported and we cannot parse the value into a known representation
    UnknownMimeType(String),
    /// Required content within the source was not found e.g. the XML child text element for a "content" element
    MissingContent(&'static str),
}

/// Parse the input (Atom, a flavour of RSS or JSON Feed) into our model
///
/// # Arguments
///
/// * `input` - A source of content such as a string, file etc.
///
/// # Examples
///
/// ```
/// use feed_rs::parser;
/// let xml = r#"
/// <feed>
///    <title type="text">sample feed</title>
///    <updated>2005-07-31T12:29:29Z</updated>
///    <id>feed1</id>
///    <entry>
///        <title>sample entry</title>
///        <id>entry1</id>
///    </entry>
/// </feed>
/// "#;
/// let feed_from_xml = parser::parse(xml.as_bytes()).unwrap();
///
///
/// ```
pub fn parse<R: Read>(source: R) -> ParseFeedResult<model::Feed> {
    // Buffer the reader for performance (e.g. when streaming from a network) and so we can peek to determine the type of content
    let mut input = BufReader::new(source);

    // Determine whether this is XML or JSON and call the appropriate parser
    input.fill_buf()?;
    let first_char = input.buffer().iter().find(|b| **b == b'<' || **b == b'{').map(|b| *b as char);
    let result = match first_char {
        Some('<') => parse_xml(input),

        Some('{') => parse_json(input),

        _ => Err(ParseFeedError::ParseError(ParseErrorKind::NoFeedRoot))
    };

    // Post processing as required
    if let Ok(mut feed) = result {
        assign_missing_ids(&mut feed);

        Ok(feed)
    } else {
        result
    }
}

// Assigns IDs to missing feed + entries as required
fn assign_missing_ids(feed: &mut model::Feed) {
    if feed.id.is_empty() {
        feed.id = create_id(&feed.links, &feed.title);
    }

    for entry in feed.entries.iter_mut() {
        if entry.id.is_empty() {
            entry.id = create_id(&entry.links, &entry.title);
        }
    }
}

const LINK_HASH_KEY1: u64 = 0x5d78_4074_2887_2d60;
const LINK_HASH_KEY2: u64 = 0x90ee_ca4c_90a5_e228;

// Creates a unique ID from the first link, or a UUID if no links are available
fn create_id(links : &[model::Link], title: &Option<model::Text>) -> String {
    // Generate a stable ID for this item based on the first link
    if let Some(link) = links.iter().next() {
        let mut hasher = SipHasher::new_with_keys(LINK_HASH_KEY1, LINK_HASH_KEY2);
        hasher.write(link.href.as_bytes());
        if let Some(title) = title {
            hasher.write(title.content.as_bytes());
        }
        let hash = hasher.finish128();
        format!("{:x}{:x}", hash.h1, hash.h2)
    } else {
        // Generate a UUID as last resort
        util::uuid_gen()
    }
}

// Handles JSON content
fn parse_json<R: Read>(source: R) -> ParseFeedResult<model::Feed> {
    json::parse(source)
}

// Handles XML content
fn parse_xml<R: Read>(source: R) -> ParseFeedResult<model::Feed> {
    // Set up the source of XML elements from the input
    let source = ElementSource::new(source);

    if let Ok(Some(root)) = source.root() {
        // Dispatch to the correct parser
        let version = attr_value(&root.attributes, "version");
        match (root.name.local_name.as_str(), version) {
            ("feed", _) => return atom::parse(root),
            ("rss", Some("2.0")) => return rss2::parse(root),
            ("rss", Some("0.91")) | ("rss", Some("0.92")) => return rss0::parse(root),
            ("RDF", _) => return rss1::parse(root),
            _ => {}
        };
    }

    // Couldn't find a recognised feed within the provided XML stream
    Err(ParseFeedError::ParseError(ParseErrorKind::NoFeedRoot))
}
