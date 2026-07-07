use core::fmt;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::Debug;
use std::io::BufRead;
use std::mem;

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, Reader};
use url::Url;

#[cfg(test)]
mod tests;

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
        let mut reader = NsReader::from_reader(xml_data);
        let config = reader.config_mut();
        config.expand_empty_elements = true;
        config.trim_markup_names_in_closing_tags = true;
        config.trim_text(false);

        let state = RefCell::new(SourceState::new(reader, xml_base_uri)?);
        Ok(ElementSource { state })
    }

    /// Set default namespace if not set explicitly by the document.
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

    // Return the raw XML of all children at or below the nominated depth
    fn children_as_string(&self, depth: u32, buffer: &mut String) -> XmlResult<()> {
        // Read nodes at the current depth or greater
        let mut state = self.state.borrow_mut();
        let mut current_depth = depth;

        loop {
            // A strange construction, but we need to throw an error if we cannot consume all the children (e.g. malformed XML)
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
                        append_element_start(buffer, name, attributes);
                    }

                    XmlEvent::Text(text) => {
                        // Append text to the buffer
                        append_element_text(buffer, text);
                    }

                    XmlEvent::End { name, .. } => {
                        // Break out of the iteration if we would move above our iteration depth
                        current_depth -= 1;
                        if current_depth < depth {
                            break;
                        }

                        // Append this terminating element
                        append_element_end(buffer, name);
                    }
                }

                // Consume this node
                state.next()?;
            } else {
                // In the case where we have no more nodes, we hit the end of the document so we can just break out of this loop
                break;
            }
        }

        Ok(())
    }

    // Returns the next element at the nominated depth
    fn next_element_at_depth(&self, iter_depth: u32) -> XmlResult<Option<Element<'_, R>>> {
        // Read nodes until we arrive at the correct depth
        let mut state = self.state.borrow_mut();
        while let Some(node) = state.next()? {
            match node {
                // The start of an element may be interesting to the iterator
                XmlEvent::Start { name, attributes, namespace } => {
                    // Starting an element increases our depth
                    state.current_depth += 1;

                    // Update the xml-base if required
                    ElementSource::xml_base_push(&mut state, &attributes)?;

                    // If we are at the correct depth we found a node of interest
                    if state.current_depth == iter_depth {
                        let element = Element {
                            namespace: namespace.unwrap_or(state.default_namespace),
                            name,
                            attributes,
                            xml_base: ElementSource::xml_base_fetch(&state),
                            source: self,
                            depth: state.current_depth,
                        };
                        return Ok(Some(element));
                    }
                }

                // The end of an element moves back up the hierarchy
                XmlEvent::End { .. } => {
                    state.current_depth -= 1;

                    // Update the xml-base if required
                    ElementSource::xml_base_pop(&mut state);
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

    // Extracts a text element
    fn text_node(&self) -> Option<String> {
        let mut state = self.state.borrow_mut();

        // If the next event is characters, we have found our text
        if let Ok(Some(XmlEvent::Text(_text))) = state.peek() {
            // Grab the next event - we know its a Text event from the above
            match state.next() {
                Ok(Some(XmlEvent::Text(text))) => return Some(text),
                _ => unreachable!("state.next() did not return expected XmlEvent::Text"),
            }
        }

        None
    }

    // Fetches the currently active xml-base
    fn xml_base_fetch(state: &SourceState<R>) -> Option<Url> {
        // Return the last entry on the stack if it exists
        state.base_uris.last().map(|(_, uri)| uri.clone())
    }

    // Pops xml-base entries off the stack as required
    fn xml_base_pop(state: &mut SourceState<R>) {
        // Pop any URIs off the stack if they are deeper than our new depth
        while !state.base_uris.is_empty() {
            let (depth, _) = state.base_uris.last().unwrap();
            if depth > &state.current_depth {
                state.base_uris.pop();
            } else {
                break;
            }
        }
    }

    // Pushes an updated xml-base on to the stack as required
    fn xml_base_push(state: &mut SourceState<R>, attributes: &[NameValue]) -> XmlResult<()> {
        // Find the xml-base attribute
        let xml_base = attributes.iter().find(|nv| nv.name == "xml:base").map(|nv| &nv.value);

        if let Some(xml_base) = xml_base {
            match Url::parse(xml_base) {
                Ok(uri) => {
                    // It is an absolute URI so push it straight on to the stack
                    state.base_uris.push((state.current_depth, uri));
                }
                Err(url::ParseError::RelativeUrlWithoutBase) => {
                    // Try and form a new URL and push it to the stack
                    if let Some((_, last)) = state.base_uris.last() {
                        if let Ok(with_base) = last.join(xml_base) {
                            state.base_uris.push((state.current_depth, with_base));
                        }
                    }
                }
                Err(e) => return Err(XmlError::Url { e }),
            }
        }

        // No xml-base found
        Ok(())
    }
}

// Wraps the XML source and current depth of iteration
struct SourceState<R: BufRead> {
    reader: NsReader<R>,
    buf_event: Vec<u8>,
    next: XmlResult<Option<XmlEvent>>,
    // An event stashed while coalescing text (e.g. the start tag terminating a run of text and entity references)
    pending: Option<XmlEvent>,
    current_depth: u32,
    base_uris: Vec<(u32, Url)>,
    default_namespace: NS,
}

impl<R: BufRead> SourceState<R> {
    // Wrap the reader in additional state (buffers, tree depth etc)
    fn new(reader: NsReader<R>, xml_base_uri: Option<&str>) -> XmlResult<SourceState<R>> {
        // If we have a base URI, parse it and init at the root
        let mut base_uris = Vec::new();
        if let Some(xml_base_uri) = xml_base_uri {
            let uri = Url::parse(xml_base_uri)?;
            base_uris.push((0, uri));
        }

        let buf_event = Vec::with_capacity(512);
        let mut state = SourceState {
            reader,
            buf_event,
            next: Ok(None),
            pending: None,
            current_depth: 0,
            base_uris,
            default_namespace: NS::Unknown,
        };
        state.next = state.fetch_next();
        Ok(state)
    }

    // Returns the next event
    fn fetch_next(&mut self) -> XmlResult<Option<XmlEvent>> {
        // Emit an event stashed while coalescing text
        if let Some(event) = self.pending.take() {
            return Ok(Some(event));
        }

        let decoder = self.reader.decoder();
        let reader = &mut self.reader;

        // Text, CData and entity references are coalesced into a single text event
        let mut text: Option<String> = None;

        loop {
            let (ns_resolution, event) = reader.read_resolved_event_into(&mut self.buf_event)?;

            match event {
                // Start of an element
                Event::Start(ref e) => {
                    // Parse the namespace
                    // The default namespace is applied when the event is consumed, since it may not be known yet
                    // (e.g. the root element is examined to determine the feed type, which in turn sets the default namespace)
                    let namespace = match ns_resolution {
                        ResolveResult::Bound(ns) => decoder.decode(ns.as_ref()).ok().map(|decoded| NS::parse(decoded.as_ref())),
                        ResolveResult::Unknown(_) => None,
                        ResolveResult::Unbound => None,
                    };

                    let start = XmlEvent::start(namespace, e, reader);
                    return match text.take() {
                        Some(text) => {
                            self.pending = Some(start);
                            Ok(Some(XmlEvent::Text(text)))
                        }
                        None => Ok(Some(start)),
                    };
                }

                // End of an element
                Event::End(ref e) => {
                    let end = XmlEvent::end(e, reader);
                    return match text.take() {
                        Some(text) => {
                            self.pending = Some(end);
                            Ok(Some(XmlEvent::Text(text)))
                        }
                        None => Ok(Some(end)),
                    };
                }

                // Text
                Event::Text(ref t) => {
                    if !t.is_empty() {
                        let decoded = decoder.decode(t)?;
                        text.get_or_insert_with(String::new).push_str(&decoded);
                    }
                }

                // CData is converted to text
                Event::CData(ref t) => {
                    if !t.is_empty() {
                        let decoded = decoder.decode(t)?;
                        text.get_or_insert_with(String::new).push_str(&decoded);
                    }
                }

                // Character and entity references (e.g. "&#38;" or "&amp;") are resolved into text
                Event::GeneralRef(ref r) => {
                    let buffer = text.get_or_insert_with(String::new);
                    if let Some(ch) = r.resolve_char_ref()? {
                        buffer.push(ch);
                    } else {
                        let name = decoder.decode(r)?;
                        match quick_xml::escape::resolve_predefined_entity(&name) {
                            Some(resolved) => buffer.push_str(resolved),
                            // Unknown entities cannot be resolved, so retain them in their escaped form
                            None => {
                                buffer.push('&');
                                buffer.push_str(&name);
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
}

// TODO this is flagged as needless, but is required in Element... fix this
#[allow(clippy::needless_lifetimes)]
impl<'a, R: BufRead> Element<'a, R> {
    /// Returns the value for an attribute if it exists
    pub(crate) fn attr_value(&self, name: &str) -> Option<String> {
        self.attributes.iter().find(|a| a.name == name).map(|a| a.value.clone())
    }

    /// If the first child of the current node is XML characters, then it is returned as a `String` otherwise `None`.
    pub(crate) fn child_as_text(&self) -> Option<String> {
        self.source.text_node()
    }

    /// Returns an iterator over children of this element (i.e. descends a level in the hierarchy)
    pub(crate) fn children(&self) -> ElementIter<'_, R> {
        ElementIter {
            source: self.source,
            depth: self.depth + 1,
        }
    }

    /// Concatenates the children of this node into a string
    ///
    /// NOTE: the input stream is parsed then re-serialised so the output will not be identical to the input
    pub(crate) fn children_as_string(&self) -> XmlResult<Option<String>> {
        // Fill the buffer with the XML content below this element
        let mut buffer = String::new();
        self.source.children_as_string(self.depth + 1, &mut buffer)?;

        Ok(Some(buffer))
    }

    /// Returns the namespace + tag name for this element
    pub(crate) fn ns_and_tag(&self) -> (NS, &str) {
        (self.namespace, &self.name)
    }
}

// TODO this is flagged as needless, but is required in Element... fix this
#[allow(clippy::needless_lifetimes)]
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
}

impl NS {
    fn parse(s: &str) -> NS {
        match s {
            "http://purl.org/rss/1.0/" => NS::RSS,
            "http://www.w3.org/2005/Atom" => NS::Atom,

            // Extension namespaces
            "http://purl.org/rss/1.0/modules/content/" => NS::Content,
            "http://purl.org/dc/elements/1.1/" => NS::DublinCore,
            "http://search.yahoo.com/mrss/" => NS::MediaRSS,
            "http://www.itunes.com/dtds/podcast-1.0.dtd" => NS::Itunes,

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
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmlError::Parser { e } => write!(f, "Parser error: {}", e),
            XmlError::Url { e } => write!(f, "Url error: {}", e),
            XmlError::Encoding { e } => write!(f, "Encoding error: {}", e),
            XmlError::Escape { e } => write!(f, "Escape error: {}", e),
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
    fn end<R: BufRead>(event: &BytesEnd, reader: &Reader<R>) -> XmlEvent {
        // Parse the name
        let name = XmlEvent::parse_name(event.name().as_ref(), reader);

        XmlEvent::End { name }
    }

    // Extracts the element name, dropping the namespace prefix if present
    fn parse_name<R: BufRead>(bytes: &[u8], reader: &Reader<R>) -> String {
        reader
            .decoder()
            .decode(bytes)
            .ok()
            .and_then(|name| name.split(':').next_back().map(str::to_string))
            .unwrap_or_default()
    }

    // Creates a new event corresponding to an XML start-tag
    fn start<R: BufRead>(namespace: Option<NS>, event: &BytesStart, reader: &Reader<R>) -> XmlEvent {
        // Parse the name
        let name = XmlEvent::parse_name(event.name().as_ref(), reader);

        // Parse the attributes
        let attributes = event
            .attributes()
            .filter_map(|a| {
                if let Ok(a) = a {
                    let name = match reader.decoder().decode(a.key.as_ref()) {
                        Ok(decoded) => decoded,
                        Err(_) => return None,
                    };

                    // Unescape the XML attribute, or use the original value if this fails (broken escape sequence etc)
                    let decoded_value = match reader.decoder().decode(&a.value) {
                        Ok(decoded) => decoded,
                        Err(_) => return None,
                    };
                    let value = quick_xml::escape::unescape(&decoded_value)
                        .unwrap_or_else(|_| decoded_value.clone())
                        .to_string();

                    Some(NameValue { name: name.into(), value })
                } else {
                    None
                }
            })
            .collect::<Vec<NameValue>>();

        XmlEvent::Start { namespace, name, attributes }
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

// Appends a text element
fn append_element_text(buffer: &mut String, text: &str) {
    buffer.push_str(text);
}
