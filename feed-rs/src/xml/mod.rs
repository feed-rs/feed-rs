use core::fmt;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::Debug;
use std::io::BufRead;
use std::mem;

use crate::xml::xml_base::XmlBase;
use quick_xml::encoding::DecodingReader;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, Reader};
use url::Url;

mod xml_base;

/// Iteration over the XML elements may return an error (malformed content etc)
pub(crate) type XmlResult<T> = Result<T, XmlError>;

/// Produces elements from the provided source
pub(crate) struct ElementSource<R: BufRead> {
    // Needs to be a RefCell since we can't borrow mutably multiple times (e.g. when calls to Element::children() are nested)
    state: RefCell<SourceState<R>>,
}

impl<R: BufRead> ElementSource<R> {
    /// Parses the XML stream and emits elements
    ///
    /// # Arguments
    ///
    /// * `xml_data` - the data you wish to parse
    /// * `xml_base_uri` - the base URI if known (e.g. Content-Location, feed URI etc)
    pub(crate) fn new(xml_data: R, xml_base_uri: Option<&str>) -> XmlResult<ElementSource<R>> {
        // Create the XML parser
        let mut reader = NsReader::from_reader(DecodingReader::new(xml_data));
        let config = reader.config_mut();
        config.expand_empty_elements = true;
        config.trim_markup_names_in_closing_tags = true;
        config.trim_text(false);

        let state = RefCell::new(SourceState::new(reader, xml_base_uri)?);
        Ok(ElementSource { state })
    }

    /// Set the default namespace if not set explicitly by the document.
    pub(crate) fn set_default_default_namespace(&self, namespace: NS) {
        let mut state = self.state.borrow_mut();
        if state.default_namespace == NS::Unknown {
            state.default_namespace = namespace;
        }
    }

    /// Returns the first element in the source
    pub(crate) fn root(&self) -> XmlResult<Option<Element<'_, R>>> {
        self.next_element_at_depth(1)
    }

    /// Collects content (text, sub-elements, CDATA sections etc)
    fn content(&self, depth: u32, escape_text: bool) -> XmlResult<String> {
        // Read nodes at the current depth or greater and append them to the buffer
        let mut state = self.state.borrow_mut();
        let mut current_depth = depth;
        let mut buffer = String::new();

        loop {
            // We need to look ahead via peek() to avoid consuming the terminating element at our current depth as that will break the state machine built over the top of the parse stream
            // e.g. handling an RSS <item> expects to find the terminating </item> to run post-processing
            let peeked = state.peek();
            if peeked.is_err() {
                return Err(state.next().err().unwrap());
            }

            // Fetch the next event
            if let Some(event) = peeked.as_ref().unwrap() {
                match event {
                    XmlEvent::Start { name, attributes, .. } => {
                        // Note that we have descended into an element
                        current_depth += 1;

                        // Append element start to the buffer
                        append_element_start(&mut buffer, name, attributes);
                    }

                    XmlEvent::Text(text) => {
                        // Append text to the buffer, escaped again if the result is to be read as markup
                        if escape_text {
                            buffer.push_str(&quick_xml::escape::minimal_escape(text));
                        } else {
                            buffer.push_str(text);
                        }
                    }

                    XmlEvent::End { name, .. } => {
                        // Break out of the iteration if we would move above our iteration depth
                        current_depth -= 1;
                        if current_depth < depth {
                            break;
                        }

                        // Append this terminating element
                        append_element_end(&mut buffer, name);
                    }
                }

                // Consume this node
                state.next()?;
            } else {
                // In the case where we have no more nodes, we hit the end of the document so we can just break out of this loop
                break;
            }
        }

        Ok(buffer)
    }

    // Returns the next element at the nominated depth
    fn next_element_at_depth(&self, iter_depth: u32) -> XmlResult<Option<Element<'_, R>>> {
        // Read nodes until we arrive at the correct depth
        let mut state = self.state.borrow_mut();
        while let Some(node) = state.next()? {
            match node {
                // The start of an element may be interesting to the iterator
                XmlEvent::Start {
                    buffer_pos,
                    name,
                    attributes,
                    namespace,
                } => {
                    // Starting an element increases our depth
                    state.current_depth += 1;

                    // Update the xml-base if required
                    let current_depth = state.current_depth;
                    state.xml_base.push(current_depth, &attributes)?;

                    // If we are at the correct depth we found a node of interest
                    if state.current_depth == iter_depth {
                        let element = Element {
                            namespace: namespace.unwrap_or(state.default_namespace),
                            name,
                            attributes,
                            xml_base: state.xml_base.fetch(),
                            source: self,
                            depth: state.current_depth,
                            buffer_pos,
                        };
                        return Ok(Some(element));
                    }
                }

                // The end of an element moves back up the hierarchy
                XmlEvent::End { .. } => {
                    state.current_depth -= 1;
                    let current_depth = state.current_depth;
                    state.xml_base.pop(current_depth);
                }

                // Not interested in other events when looking for elements
                _ => {}
            }

            // If we have hit the end of children at this level we terminate
            if state.current_depth < iter_depth - 1 {
                return Ok(None);
            }
        }

        // Hit the end of the document
        if state.current_depth > 0 {
            // let msg = format!("documented terminated at depth {}", state.current_depth);
            let e = quick_xml::Error::Syntax(quick_xml::errors::SyntaxError::UnclosedTag);
            Err(XmlError::Parser { e })
        } else {
            Ok(None)
        }
    }
}

// Wraps the XML source and current depth of iteration
struct SourceState<R: BufRead> {
    reader: NsReader<DecodingReader<R>>,
    buf_event: Vec<u8>,
    next: XmlResult<Option<XmlEvent>>,
    // An event stashed while coalescing text (e.g. the start tag terminating a run of text and entity references)
    pending_text: Option<XmlEvent>,
    current_depth: u32,
    xml_base: XmlBase,
    default_namespace: NS,
}

impl<R: BufRead> SourceState<R> {
    // Wrap the reader in additional state (buffers, tree depth etc)
    fn new(reader: NsReader<DecodingReader<R>>, xml_base_uri: Option<&str>) -> XmlResult<SourceState<R>> {
        let buf_event = Vec::with_capacity(512);
        let mut state = SourceState {
            reader,
            buf_event,
            next: Ok(None),
            pending_text: None,
            current_depth: 0,
            xml_base: XmlBase::new(xml_base_uri)?,
            default_namespace: NS::Unknown,
        };
        state.next = state.fetch_next();
        Ok(state)
    }

    // Returns the next event
    fn fetch_next(&mut self) -> XmlResult<Option<XmlEvent>> {
        // Emit an event stashed while coalescing text
        if let Some(event) = self.pending_text.take() {
            return Ok(Some(event));
        }

        let reader = &mut self.reader;

        // Text, CData and entity references are coalesced into a single text event
        let mut text: Option<String> = None;

        loop {
            let (ns_resolution, event) = reader.read_resolved_event_into(&mut self.buf_event)?;

            match event {
                // XML declaration for the encoding
                Event::Decl(ref decl) => {
                    if let Some(encoding) = decl.encoder() {
                        reader.get_mut().set_encoding(encoding);
                    }
                }

                // Start of an element
                Event::Start(ref e) => {
                    // Parse the namespace
                    // The default namespace is applied when the event is consumed, since it may not be known yet
                    // (e.g. the root element is examined to determine the feed type, which in turn sets the default namespace)
                    let namespace = match ns_resolution {
                        ResolveResult::Bound(ns) => Some(NS::parse(ns.as_ref())),
                        ResolveResult::Unknown(_) => None,
                        ResolveResult::Unbound => None,
                    };

                    let start = XmlEvent::start(namespace, e, reader);
                    return match text.take() {
                        Some(text) => {
                            self.pending_text = Some(start);
                            Ok(Some(XmlEvent::Text(text)))
                        }
                        None => Ok(Some(start)),
                    };
                }

                // End of an element
                Event::End(ref e) => {
                    let end = XmlEvent::end(e);
                    return match text.take() {
                        Some(text) => {
                            self.pending_text = Some(end);
                            Ok(Some(XmlEvent::Text(text)))
                        }
                        None => Ok(Some(end)),
                    };
                }

                // Text
                Event::Text(ref t) => {
                    if !t.is_empty() {
                        text.get_or_insert_with(String::new).push_str(t);
                    }
                }

                // CData is converted to text
                Event::CData(ref t) => {
                    if !t.is_empty() {
                        text.get_or_insert_with(String::new).push_str(t);
                    }
                }

                // Character and entity references (e.g. "&#38;" or "&amp;") are resolved into text
                Event::GeneralRef(ref r) => {
                    let buffer = text.get_or_insert_with(String::new);
                    if let Some(ch) = r.resolve_char_ref()? {
                        buffer.push(ch);
                    } else {
                        let name = r.as_ref();
                        match quick_xml::escape::resolve_predefined_entity(name) {
                            Some(resolved) => buffer.push_str(resolved),
                            // Unknown entities cannot be resolved, so retain them in their escaped form
                            None => {
                                buffer.push('&');
                                buffer.push_str(name);
                                buffer.push(';');
                            }
                        }
                    }
                }

                // The end of the document
                Event::Eof => {
                    return Ok(text.take().map(XmlEvent::Text));
                }

                // Ignore everything else
                _ => {}
            }
        }
    }

    // Returns the next interesting event or None if no more events are found
    fn next(&mut self) -> XmlResult<Option<XmlEvent>> {
        let next = mem::replace(&mut self.next, Ok(None));
        self.next = self.fetch_next();
        next
    }

    // Peeks the next event (does not advance)
    // Callers should call next() to consume the event to move on
    fn peek(&mut self) -> &XmlResult<Option<XmlEvent>> {
        &self.next
    }
}

/// An element (specifically, XML element start tag)
pub(crate) struct Element<'a, R: BufRead> {
    /// Qualified name of the element.
    pub name: String,

    /// The namespace mapping at this point of the document.
    pub namespace: NS,

    /// A list of attributes associated with the element.
    pub attributes: Vec<NameValue>,

    /// The base URL for this element per the xml:base specification (https://www.w3.org/TR/xmlbase/)
    pub xml_base: Option<Url>,

    // Depth of this element
    depth: u32,

    // The underlying source of XML events
    source: &'a ElementSource<R>,

    // Byte position in the source at which this element starts
    pub buffer_pos: u64,
}

impl<'a, R: BufRead> Element<'a, R> {
    /// Returns the value for an attribute if it exists
    pub(crate) fn attr_value(&self, name: &str) -> Option<String> {
        self.attributes.iter().find(|a| a.name == name).map(|a| a.value.clone())
    }

    /// Returns an iterator over children of this element (i.e. descends a level in the hierarchy)
    pub(crate) fn children(&self) -> ElementIter<'_, R> {
        ElementIter {
            source: self.source,
            depth: self.depth + 1,
        }
    }

    /// Returns the content (text, CDATA sections, mixed content, etc.) of this element with literal markup intact e.g. `&lt;` will be returned as `<`.
    pub(crate) fn content_as_literal_markup(&self) -> XmlResult<Option<String>> {
        // Collect content at this level or lower.
        let content = self.source.content(self.depth + 1, false)?;

        // Convert to its unescaped form
        Ok(content_as_literal_markup(content))
    }

    /// Collect the content (text or children) of this node and return it as a `String` otherwise `None`.
    pub(crate) fn content_as_text(&self) -> Option<String> {
        self.source.content(self.depth + 1, false).ok().filter(|content| !content.is_empty())
    }

    /// Returns the content of this element as markup, with text escaped again (e.g. `a &lt; b` stays `a &lt; b`).
    pub(crate) fn content_as_xhtml(&self) -> XmlResult<Option<String>> {
        let content = self.source.content(self.depth + 1, true)?;
        Ok(Some(content).filter(|content| !content.trim().is_empty()))
    }

    /// Returns the namespace + tag name for this element
    pub(crate) fn ns_and_tag(&self) -> (NS, &str) {
        (self.namespace, &self.name)
    }
}

impl<'a, R: BufRead> Debug for Element<'a, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = String::new();
        append_element_start(&mut buffer, &self.name, &self.attributes);
        writeln!(f, "{}", buffer)
    }
}

/// Iterator over elements at a specific depth in the hierarchy
pub(crate) struct ElementIter<'a, R: BufRead> {
    source: &'a ElementSource<R>,
    depth: u32,
}

impl<'a, R: BufRead> Iterator for ElementIter<'a, R> {
    type Item = XmlResult<Element<'a, R>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next_element_at_depth(self.depth).transpose()
    }
}

/// Set of automatically recognised namespaces
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum NS {
    Atom,
    RSS,
    // Namespaces we do not support are treated as this special case, to avoid processing content incorrectly
    Unknown,
    // Extensions
    Content,
    DublinCore,
    MediaRSS,
    Itunes,
    Podcast,
    WellFormedWebComments,
}

impl NS {
    //noinspection HttpUrlsUsage
    fn parse(s: &str) -> NS {
        match s {
            "http://purl.org/rss/1.0/" => NS::RSS,
            "http://www.w3.org/2005/Atom" => NS::Atom,

            // Extension namespaces
            "http://purl.org/rss/1.0/modules/content/" => NS::Content,
            "http://purl.org/dc/elements/1.1/" => NS::DublinCore,
            "http://search.yahoo.com/mrss/" => NS::MediaRSS,
            "http://www.itunes.com/dtds/podcast-1.0.dtd" => NS::Itunes,
            "https://podcastindex.org/namespace/1.0" => NS::Podcast,
            "http://wellformedweb.org/CommentAPI/" => NS::WellFormedWebComments,

            // Everything else is ignored
            _ => NS::Unknown,
        }
    }
}

/// Combination of a name and value (e.g. attribute name + value)
pub(crate) struct NameValue {
    pub name: String,
    pub value: String,
}

/// Errors for the underlying parser
#[derive(Debug)]
pub enum XmlError {
    Parser { e: quick_xml::Error },
    Url { e: url::ParseError },
    Encoding { e: quick_xml::encoding::EncodingError },
    Escape { e: quick_xml::escape::EscapeError },
    ValidationError { msg: String },
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmlError::Parser { e } => write!(f, "Parser error: {}", e),
            XmlError::Url { e } => write!(f, "Url error: {}", e),
            XmlError::Encoding { e } => write!(f, "Encoding error: {}", e),
            XmlError::Escape { e } => write!(f, "Escape error: {}", e),
            XmlError::ValidationError { msg } => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for XmlError {}

impl From<quick_xml::Error> for XmlError {
    fn from(e: quick_xml::Error) -> Self {
        XmlError::Parser { e }
    }
}

impl From<url::ParseError> for XmlError {
    fn from(e: url::ParseError) -> Self {
        XmlError::Url { e }
    }
}

impl From<quick_xml::encoding::EncodingError> for XmlError {
    fn from(e: quick_xml::encoding::EncodingError) -> Self {
        XmlError::Encoding { e }
    }
}

impl From<quick_xml::escape::EscapeError> for XmlError {
    fn from(e: quick_xml::escape::EscapeError) -> Self {
        XmlError::Escape { e }
    }
}

// Abstraction over the underlying XML reader event model
enum XmlEvent {
    // An XML start tag
    // The namespace is `None` when the document does not declare one; the source's default namespace is applied on consumption
    Start {
        buffer_pos: u64,
        namespace: Option<NS>,
        name: String,
        attributes: Vec<NameValue>,
    },
    // An XML end tag
    End {
        name: String,
    },
    // Text or CData
    Text(String),
}

impl XmlEvent {
    // Creates a new event corresponding to an XML end-tag
    fn end(event: &BytesEnd) -> XmlEvent {
        // Parse the name
        let name = XmlEvent::parse_name(event.name().as_ref());

        XmlEvent::End { name }
    }

    // Extracts the element name, dropping the namespace prefix if present
    fn parse_name(name: &str) -> String {
        name.split(':').next_back().map(|s| s.to_string()).unwrap_or_default()
    }

    // Creates a new event corresponding to an XML start-tag
    fn start<R: BufRead>(namespace: Option<NS>, event: &BytesStart, reader: &Reader<R>) -> XmlEvent {
        let buffer_pos = reader.buffer_position();
        // Parse the name
        let name = XmlEvent::parse_name(event.name().as_ref());

        // Parse the attributes
        let attributes = event
            .attributes()
            .filter_map(|a| {
                if let Ok(a) = a {
                    let name = a.key.as_ref();

                    // Unescape the XML attribute or use the original value if this fails (broken escape sequence etc)
                    let decoded_value = &a.value;
                    let value = quick_xml::escape::unescape(decoded_value).unwrap_or_else(|_| decoded_value.clone()).to_string();

                    Some(NameValue { name: name.into(), value })
                } else {
                    None
                }
            })
            .collect::<Vec<NameValue>>();

        XmlEvent::Start {
            buffer_pos,
            namespace,
            name,
            attributes,
        }
    }
}

// Appends an element-end to the buffer
// TODO find example XML that has namespaces and use a separate function to serialise the name
fn append_element_end(buffer: &mut String, name: &str) {
    buffer.push_str("</");
    buffer.push_str(name);
    buffer.push('>');
}

// Appends an element-start to the buffer
fn append_element_start(buffer: &mut String, name: &str, attributes: &[NameValue]) {
    buffer.push('<');
    buffer.push_str(name);
    for attr in attributes {
        buffer.push(' ');
        buffer.push_str(attr.name.as_str());
        buffer.push_str("=\"");
        buffer.push_str(attr.value.as_str());
        buffer.push('"');
    }
    buffer.push('>');
}

fn content_as_literal_markup(content: String) -> Option<String> {
    if content.trim().is_empty() {
        return None;
    };

    // If the escaped "<" is present, we try to unescape it
    if content.contains("&lt;") {
        if let Ok(unescaped) = quick_xml::escape::unescape(&content) {
            return Some(unescaped.to_string());
        }
    }

    // It wasn't escaped, or we couldn't unescape, so just return the original content
    Some(content.to_string())
}

#[cfg(test)]
mod tests;
