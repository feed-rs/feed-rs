use chrono::{NaiveDateTime, Utc};
use mime::Mime;

use crate::util;
#[cfg(test)]
use crate::util::timestamp_from_rfc2822;
#[cfg(test)]
use crate::util::timestamp_from_rfc3339;

#[derive(Debug, PartialEq)]
/// Combined model for a syndication feed (i.e. RSS1, RSS 2, Atom)
///
/// The model is based on the Atom standard as a start with RSS1+2 mapped on to it
/// Atom:
///     Feed -> Feed, Entry -> Entry
/// RSS 1 + 2:
///     Channel -> Feed, Item -> Entry
///
/// Atom spec: http://www.atomenabled.org/developers/syndication/
/// RSS 2 spec: https://validator.w3.org/feed/docs/rss2.html
/// RSS 1 spec: https://validator.w3.org/feed/docs/rss1.html
///
/// Certain elements are not mapped given their limited utility:
///   * RSS 2:
///     * channel - docs (pointer to the spec), cloud (for callbacks), textInput (text box e.g. for search)
///     * item - comments (link to comments on the article), source (pointer to the channel, but our data model links items to a channel)
///   * RSS 1:
///     * channel - rdf:about attribute (pointer to feed), textinput (text box e.g. for search)
pub struct Feed {
    /// Atom (required): Identifies the feed using a universally unique and permanent URI.
    pub id: String,
    /// Atom (required): Contains a human readable title for the feed. Often the same as the title of the associated website. This value should not be blank.
    /// RSS 1 + 2 (required) "title": The name of the channel. It's how people refer to your service.
    pub title: Option<Text>,
    /// Atom (required): Indicates the last time the feed was modified in a significant way.
    /// RSS 2 (optional) "lastBuildDate": The last time the content of the channel changed.
    pub updated: NaiveDateTime,

    /// Atom (recommended): Collection of authors defined at the feed level.
    pub authors: Vec<Person>,
    /// Atom (optional): Contains a human-readable description or subtitle for the feed (from <subtitle>).
    /// RSS 1 + 2 (required): Phrase or sentence describing the channel.
    pub description: Option<Text>,
    /// Atom (recommended): Identifies a related Web page.
    /// TODO Atom supports multiple links (double check the validator for other elements with N)
    /// RSS 1 + 2 (required): The URL to the HTML website corresponding to the channel.
    pub links: Vec<Link>,

    /// Atom (optional): Specifies a category that the feed belongs to. A feed may have multiple category elements.
    /// RSS 2 (optional) "category": Specify one or more categories that the channel belongs to.
    pub categories: Vec<Category>,
    /// Atom (optional): Names one contributor to the feed. A feed may have multiple contributor elements.
    /// RSS 2 (optional) "managingEditor": Email address for person responsible for editorial content.
    /// RSS 2 (optional) "webMaster": Email address for person responsible for technical issues relating to channel.
    pub contributors: Vec<Person>,
    /// Atom (optional): Identifies the software used to generate the feed, for debugging and other purposes.
    /// RSS 2 (optional): A string indicating the program used to generate the channel.
    pub generator: Option<Generator>,
    /// Atom (optional): Identifies a small image which provides iconic visual identification for the feed.
    pub icon: Option<Image>,
    /// RSS 2 (optional): The language the channel is written in.
    pub language: Option<String>,
    /// Atom (optional): Identifies a larger image which provides visual identification for the feed.
    /// RSS 1 + 2 (optional) "image": Specifies a GIF, JPEG or PNG image that can be displayed with the channel.
    pub logo: Option<Image>,
    /// RSS 2 (optional): The publication date for the content in the channel.
    pub published: Option<NaiveDateTime>,
    /// Atom (optional): Conveys information about rights, e.g. copyrights, held in and over the feed.
    /// RSS 2 (optional) "copyright": Copyright notice for content in the channel.
    pub rights: Option<Text>,
    /// RSS 2 (optional): It's a number of minutes that indicates how long a channel can be cached before refreshing from the source.
    pub ttl: Option<u32>,

    /// Atom (optional): Individual entries within the feed (e.g. a blog post)
    /// RSS 1+2 (optional): Individual items within the channel.
    pub entries: Vec<Entry>,
}

impl Default for Feed {
    fn default() -> Self {
        Feed {
            id: util::uuid_gen(),
            title: None,
            updated: Utc::now().naive_utc(),
            authors: Vec::new(),
            description: None,
            links: Vec::new(),
            categories: Vec::new(),
            contributors: Vec::new(),
            generator: None,
            icon: None,
            language: None,
            logo: None,
            published: None,
            rights: None,
            ttl: None,
            entries: Vec::new(),
        }
    }
}

#[cfg(test)]
impl Feed {
    pub fn author(mut self, person: Person) -> Self {
        self.authors.push(person);
        self
    }

    pub fn category(mut self, category: Category) -> Self {
        self.categories.push(category);
        self
    }

    pub fn contributor(mut self, person: Person) -> Self {
        self.contributors.push(person);
        self
    }

    pub fn description(mut self, description: Text) -> Self {
        self.description = Some(description);
        self
    }

    pub fn entry(mut self, entry: Entry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn generator(mut self, generator: Generator) -> Self {
        self.generator = Some(generator);
        self
    }

    pub fn icon(mut self, image: Image) -> Self {
        self.icon = Some(image);
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    pub fn language(mut self, language: &str) -> Self {
        self.language = Some(language.to_owned());
        self
    }

    pub fn link(mut self, link: Link) -> Self {
        self.links.push(link);
        self
    }

    pub fn logo(mut self, image: Image) -> Self {
        self.logo = Some(image);
        self
    }

    pub fn published_rfc2822(mut self, pub_date: &str) -> Self {
        self.published = timestamp_from_rfc2822(pub_date).ok();
        self
    }

    pub fn rights(mut self, rights: Text) -> Self {
        self.rights = Some(rights);
        self
    }

    pub fn title(mut self, title: Text) -> Self {
        self.title = Some(title);
        self
    }

    pub fn ttl(mut self, ttl: u32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn updated(mut self, updated: NaiveDateTime) -> Self {
        self.updated = updated;
        self
    }

    pub fn updated_rfc2822(mut self, updated: &str) -> Self {
        self.updated = timestamp_from_rfc2822(updated).unwrap();
        self
    }

    pub fn updated_rfc3339(mut self, updated: &str) -> Self {
        self.updated = timestamp_from_rfc3339(updated).unwrap();
        self
    }
}

/// An item within a feed
#[derive(Debug, PartialEq)]
pub struct Entry {
    /// Atom (required): Identifies the entry using a universally unique and permanent URI.
    /// RSS 2 (optional) "guid": A string that uniquely identifies the item.
    pub id: String,
    /// Atom, RSS 1(required): Contains a human readable title for the entry.
    /// RSS 2 (optional): The title of the item.
    pub title: Option<Text>,
    /// Atom (required): Indicates the last time the entry was modified in a significant way.
    pub updated: NaiveDateTime,

    /// Atom (recommended): Collection of authors defined at the entry level.
    /// RSS 2 (optional): Email address of the author of the item.
    pub authors: Vec<Person>,
    /// Atom (recommended): Contains or links to the complete content of the entry.
    /// RSS 2 (optional) "enclosure": Describes a media object that is attached to the item.
    pub content: Option<Content>,
    /// Atom (recommended): Identifies a related Web page.
    /// RSS 2 (optional): The URL of the item.
    /// RSS 1 (required): The item's URL.
    pub links: Vec<Link>,
    /// Atom (recommended): Conveys a short summary, abstract, or excerpt of the entry.
    /// RSS 1+2 (optional): The item synopsis.
    pub summary: Option<Text>,

    /// Atom (optional): Specifies a category that the entry belongs to. A feed may have multiple category elements.
    /// RSS 2 (optional): Includes the item in one or more categories.
    pub categories: Vec<Category>,
    /// Atom (optional): Names one contributor to the entry. A feed may have multiple contributor elements.
    pub contributors: Vec<Person>,
    /// Atom (optional): Contains the time of the initial creation or first availability of the entry.
    /// RSS 2 (optional) "pubDate": Indicates when the item was published.
    pub published: Option<NaiveDateTime>,
    /// Atom (optional): If an entry is copied from one feed into another feed, then this contains the source feed metadata.
    pub source: Option<String>,
    /// Atom (optional): Conveys information about rights, e.g. copyrights, held in and over the feed.
    pub rights: Option<Text>,
}

impl Default for Entry {
    fn default() -> Self {
        let id = util::uuid_gen();

        Entry {
            id,
            title: None,
            updated: Utc::now().naive_utc(),
            authors: Vec::new(),
            content: None,
            links: Vec::new(),
            summary: None,
            categories: Vec::new(),
            contributors: Vec::new(),
            published: None,
            source: None,
            rights: None,
        }
    }
}

#[cfg(test)]
impl Entry {
    pub fn author(mut self, person: Person) -> Self {
        self.authors.push(person);
        self
    }

    pub fn category(mut self, category: Category) -> Self {
        self.categories.push(category);
        self
    }

    pub fn content(mut self, content: Content) -> Self {
        self.content = Some(content);
        self
    }

    pub fn contributor(mut self, person: Person) -> Self {
        self.contributors.push(person);
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    pub fn link(mut self, link: Link) -> Self {
        self.links.push(link);
        self
    }

    pub fn published_rfc2822(mut self, published: &str) -> Self {
        self.published = timestamp_from_rfc2822(published).ok();
        self
    }

    pub fn published_rfc3339(mut self, published: &str) -> Self {
        self.published = timestamp_from_rfc3339(published).ok();
        self
    }

    pub fn summary(mut self, summary: Text) -> Self {
        self.summary = Some(summary);
        self
    }

    pub fn title(mut self, title: Text) -> Self {
        self.title = Some(title);
        self
    }

    pub fn updated(mut self, updated: NaiveDateTime) -> Self {
        self.updated = updated;
        self
    }

    pub fn updated_rfc2822(mut self, updated: &str) -> Self {
        self.updated = timestamp_from_rfc2822(updated).unwrap();
        self
    }

    pub fn updated_rfc3339(mut self, updated: &str) -> Self {
        self.updated = timestamp_from_rfc3339(updated).unwrap();
        self
    }
}

/// Represents the category of a feed or entry
/// Atom spec: http://www.atomenabled.org/developers/syndication/#category
/// RSS 2 spec: https://validator.w3.org/feed/docs/rss2.html#ltcategorygtSubelementOfLtitemgt
#[derive(Debug, PartialEq)]
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

#[cfg(test)]
impl Category {
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_owned());
        self
    }

    pub fn scheme(mut self, scheme: &str) -> Self {
        self.scheme = Some(scheme.to_owned());
        self
    }
}

/// Content, or link to the content, for a given entry.
/// Atom spec: http://www.atomenabled.org/developers/syndication/#contentElement
/// RSS 2.0: https://validator.w3.org/feed/docs/rss2.html#ltenclosuregtSubelementOfLtitemgt
#[derive(Debug, PartialEq)]
pub struct Content {
    /// Atom:
    ///     If the type attribute ends in +xml or /xml, then an xml document of this type is contained inline.
    ///     If the type attribute starts with text, then an escaped document of this type is contained inline.
    ///     Otherwise a base64 encoded document of the indicated media type is contained inline.
    // TODO review after enum above
    pub body: Option<String>,
    /// Atom: The type attribute is either text, html, xhtml, in which case the content element is defined identically to other text constructs.
    pub content_type: Mime,
    /// RSS 2.0: Length of the content in bytes
    pub length: Option<u64>,
    /// Atom: If the src attribute is present, it represents the URI of where the content can be found. The type attribute, if present, is the media type of the content.
    /// RSS 2.0: where the enclosure is located
    pub src: Option<String>,
}

impl Default for Content {
    fn default() -> Content {
        Content { body: None, content_type: mime::TEXT_PLAIN, length: None, src: None }
    }
}

#[cfg(test)]
impl Content {
    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_owned());
        self
    }

    pub fn content_type(mut self, content_type: &str) -> Self {
        self.content_type = content_type.parse::<Mime>().unwrap();
        self
    }

    pub fn length(mut self, length: u64) -> Self {
        self.length = Some(length);
        self
    }

    pub fn src(mut self, url: &str) -> Self {
        self.src = Some(url.to_owned());
        self
    }
}

/// Information on the tools used to generate the feed
/// Atom: Identifies the software used to generate the feed, for debugging and other purposes.
#[derive(Debug, PartialEq)]
pub struct Generator {
    /// Atom: Additional data
    pub content: String,
    /// Atom: Link to the tool
    pub uri: Option<String>,
    /// Atom: Tool version
    pub version: Option<String>,
}

impl Generator {
    pub fn new(content: String) -> Generator {
        Generator { uri: None, version: None, content }
    }
}

#[cfg(test)]
impl Generator {
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Some(uri.to_owned());
        self
    }

    pub fn version(mut self, version: &str) -> Self {
        self.version = Some(version.to_owned());
        self
    }
}

/// Represents a a link to an image.
/// Atom spec: item + logo in http://www.atomenabled.org/developers/syndication/#optionalFeedElements
/// RSS 2 spec: https://validator.w3.org/feed/docs/rss2.html#ltimagegtSubelementOfLtchannelgt
/// RSS 1 spec: https://validator.w3.org/feed/docs/rss1.html#s5.4
#[derive(Debug, PartialEq)]
pub struct Image {
    /// RSS 1 + 2: the URL of a GIF, JPEG or PNG image that represents the channel.
    pub uri: String,
    /// RSS 1 + 2: describes the image, it's used in the ALT attribute of the HTML <img> tag when the channel is rendered in HTML.
    pub title: Option<String>,
    /// RSS 1 + 2: the URL of the site, when the channel is rendered, the image is a link to the site.
    pub link: Option<Link>,

    /// RSS 2 (optional): width of the image
    pub width: Option<u32>,
    /// RSS 2 (optional): height of the image
    pub height: Option<u32>,
    /// RSS 2 (optional): contains text that is included in the TITLE attribute of the link formed around the image in the HTML rendering.
    pub description: Option<String>,
}

impl Image {
    pub fn new(uri: String) -> Image {
        Image { uri, title: None, link: None, width: None, height: None, description: None }
    }
}

#[cfg(test)]
impl Image {
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_owned());
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn link(mut self, link: &str) -> Self {
        self.link = Some(Link::new(link.to_owned()));
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_owned());
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }
}

/// Represents a link to an associated resource for the feed or entry.
/// Atom spec: http://www.atomenabled.org/developers/syndication/#link
#[derive(Debug, PartialEq)]
pub struct Link {
    /// The URI of the referenced resource (typically a Web page).
    pub href: String,
    /// A single link relationship type.
    pub rel: Option<String>,
    /// Indicates the media type of the resource.
    pub media_type: Option<String>,
    /// Indicates the language of the referenced resource.
    pub href_lang: Option<String>,
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
            href_lang: None,
            title: None,
            length: None,
        }
    }
}

#[cfg(test)]
impl Link {
    pub fn href_lang(mut self, lang: &str) -> Self {
        self.href_lang = Some(lang.to_owned());
        self
    }

    pub fn length(mut self, length: u64) -> Self {
        self.length = Some(length);
        self
    }

    pub fn media_type(mut self, media_type: &str) -> Self {
        self.media_type = Some(media_type.to_owned());
        self
    }

    pub fn rel(mut self, rel: &str) -> Self {
        self.rel = Some(rel.to_owned());
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_owned());
        self
    }
}

/// Represents an author, contributor etc.
/// Atom spec: http://www.atomenabled.org/developers/syndication/#person
#[derive(Debug, PartialEq)]
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

#[cfg(test)]
impl Person {
    pub fn email(mut self, email: &str) -> Self {
        self.email = Some(email.to_owned());
        self
    }

    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Some(uri.to_owned());
        self
    }
}

/// Textual content, or link to the content, for a given entry.
#[derive(Debug, PartialEq)]
pub struct Text {
    pub content_type: Mime,
    pub src: Option<String>,
    // TODO review after enum above
    pub content: String,
}

impl Text {
    pub fn new(content: String) -> Text {
        Text { content_type: mime::TEXT_PLAIN, src: None, content }
    }
}

#[cfg(test)]
impl Text {
    pub fn content_type(mut self, content_type: &str) -> Self {
        self.content_type = content_type.parse::<Mime>().unwrap();
        self
    }
}
