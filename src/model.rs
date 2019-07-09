use chrono::{NaiveDateTime, Utc};

use crate::util;

#[derive(Debug)]
/// Combined model for a syndication feed (i.e. RSS1, RSS2, Atom)
/// Atom spec: http://www.atomenabled.org/developers/syndication/
pub struct Feed {
    /// Atom (required): Identifies the feed using a universally unique and permanent URI.
    pub id: String,
    /// Atom (required): Contains a human readable title for the feed. Often the same as the title of the associated website. This value should not be blank.
    pub title: String,
    /// Atom (required): Indicates the last time the feed was modified in a significant way.
    pub updated: NaiveDateTime,

    /// Atom (recommended): Collection of authors defined at the feed level.
    pub authors: Vec<Person>,
    /// Atom (recommended): Identifies a related Web page.
    pub link: Option<Link>,

    /// Atom (optional): Specifies a category that the feed belongs to. A feed may have multiple category elements.
    pub categories: Vec<Category>,
    /// Atom (optional): Names one contributor to the feed. A feed may have multiple contributor elements.
    pub contributors: Vec<Person>,
    /// Atom (optional): Identifies the software used to generate the feed, for debugging and other purposes.
    pub generator: Option<Generator>,
    /// Atom (optional): Identifies a small image which provides iconic visual identification for the feed.
    pub icon: Option<String>,
    /// Atom (optional): Identifies a larger image which provides visual identification for the feed.
    pub logo: Option<String>,
    /// Atom (optional): Conveys information about rights, e.g. copyrights, held in and over the feed.
    pub rights: Option<String>,
    /// Atom (optional): Contains a human-readable description or subtitle for the feed.
    pub subtitle: Option<String>,

    /// Atom (optional): Individual entries within the feed (e.g. a blog post)
    pub entries: Vec<Entry>,
}

impl Feed {
    fn new() -> Self {
        let id = util::uuid_gen();
        let title = format!("feed: {}", id);

        Feed {
            id,
            title,
            updated: Utc::now().naive_utc(),
            authors: Vec::new(),
            link: None,
            categories: Vec::new(),
            contributors: Vec::new(),
            generator: None,
            icon: None,
            logo: None,
            rights: None,
            subtitle: None,
            entries: Vec::new(),
        }
    }
}

/// An item within a feed
#[derive(Debug)]
pub struct Entry {
    /// Atom (required): Identifies the entry using a universally unique and permanent URI.
    pub id: String,
    /// Atom (required): Contains a human readable title for the entry.
    pub title: String,
    /// Atom (required): Indicates the last time the entry was modified in a significant way.
    pub updated: NaiveDateTime,

    /// Atom (recommended): Collection of authors defined at the entry level.
    pub authors: Vec<Person>,
    /// Atom (recommended): Contains or links to the complete content of the entry.
    pub content: Option<Content>,
    /// Atom (recommended): Identifies a related Web page.
    pub link: Option<Link>,
    /// Atom (recommended): Conveys a short summary, abstract, or excerpt of the entry.
    pub summary: Option<String>,

    /// Atom (optional): Specifies a category that the entry belongs to. A feed may have multiple category elements.
    pub categories: Vec<Category>,
    /// Atom (optional): Names one contributor to the entry. A feed may have multiple contributor elements.
    pub contributors: Vec<Person>,
    /// Atom (optional): Contains the time of the initial creation or first availability of the entry.
    pub published: Option<NaiveDateTime>,
    /// Atom (optional): If an entry is copied from one feed into another feed, then this contains the source feed metadata.
    pub source: Option<String>,
    /// Atom (optional): Conveys information about rights, e.g. copyrights, held in and over the feed.
    pub rights: Option<String>,
}

impl Entry {
    fn new() -> Self {
        let id = util::uuid_gen();
        let title = format!("entry: {}", id);

        Entry {
            id,
            title,
            updated: Utc::now().naive_utc(),
            authors: Vec::new(),
            content: None,
            link: None,
            summary: None,
            categories: Vec::new(),
            contributors: Vec::new(),
            published: None,
            source: None,
            rights: None,
        }
    }
}

/// Represents the category of a feed or entry
/// Atom spec: http://www.atomenabled.org/developers/syndication/#category
#[derive(Debug)]
pub struct Category {
    /// Atom (required): Identifies the category.
    pub term: String,
    /// Atom (optional): Identifies the categorization scheme via a URI.
    pub scheme: Option<String>,
    /// Atom (optional): Provides a human-readable label for display.
    pub label: Option<String>,
}

impl Category {
    pub fn new(term: String) -> Category {
        Category { term, scheme: None, label: None }
    }
}

/// The content, or link to the content, for a given entry.
/// Atom spec: http://www.atomenabled.org/developers/syndication/#contentElement
#[derive(Debug)]
pub struct Content {
    /// Atom: The type attribute is either text, html, xhtml, in which case the content element is defined identically to other text constructs.
   /// TODO enum
    pub content_type: Option<String>,
    /// Atom: If the src attribute is present, it represents the URI of where the content can be found. The type attribute, if present, is the media type of the content.
    pub src: Option<String>,
    /// Atom:
    ///     If the type attribute ends in +xml or /xml, then an xml document of this type is contained inline.
    ///     If the type attribute starts with text, then an escaped document of this type is contained inline.
    ///     Otherwise a base64 encoded document of the indicated media type is contained inline.
    // TODO enum
    pub inline: Option<String>,
}

impl Content {
    pub fn new() -> Content {
        Content { content_type: None, src: None, inline: None }
    }
}

/// Information on the tools used to generate the feed
/// Atom: Identifies the software used to generate the feed, for debugging and other purposes.
#[derive(Debug)]
pub struct Generator {
    /// Atom: Link to the tool
    pub uri: Option<String>,
    /// Atom: Tool version
    pub version: Option<String>,
    /// Atom: Additional data
    pub inline: Option<String>,
}

impl Generator {
    pub fn new() -> Generator {
        Generator { uri: None, version: None, inline: None }
    }
}

/// Represents a link to an associated resource for the feed or entry.
/// Atom spec: http://www.atomenabled.org/developers/syndication/#link
#[derive(Debug)]
pub struct Link {
    /// The URI of the referenced resource (typically a Web page).
    pub href: String,
    /// A single link relationship type.
    pub rel: Option<String>,
    /// Indicates the media type of the resource.
    pub media_type: Option<String>,
    /// Indicates the language of the referenced resource.
    pub hreflang: Option<String>,
    /// Human readable information about the link, typically for display purposes.
    pub title: Option<String>,
    /// The length of the resource, in bytes.
    pub length: Option<u64>,
}

impl Link {
    pub fn new(href: String) -> Link {
        Link {
            href,
            rel: None,
            media_type: None,
            hreflang: None,
            title: None,
            length: None,
        }
    }
}

/// Represents an author, contributor etc.
/// Atom spec: http://www.atomenabled.org/developers/syndication/#person
#[derive(Debug)]
pub struct Person {
    /// Atom: human-readable name for the person.
    pub name: String,
    /// Atom: home page for the person.
    pub uri: Option<String>,
    /// Atom: An email address for the person.
    pub email: Option<String>,
}

impl Person {
    pub fn new(name: String) -> Person {
        Person { name, uri: None, email: None }
    }
}
