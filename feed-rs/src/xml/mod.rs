use core::fmt;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::Debug;
use std::io::BufRead;
use std::mem;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use serde::export::Formatter;

#[cfg(test)]
mod tests;

/// Iteration over the XML elements may return an error (malformed content etc)
pub(crate) type Result<T> = std::result::Result<T, XmlError>;

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
    pub(crate) fn new(xml_data: R) -> ElementSource<R> {
        // Create the XML parser
        let mut reader = quick_xml::Reader::from_reader(xml_data);
        reader.expand_empty_elements(true)
            .trim_markup_names_in_closing_tags(true)
            .trim_text(false);

        let state = RefCell::new(SourceState::new(reader));
        ElementSource { state }
    }

    /// Returns the first element in the source
    pub(crate) fn root(&self) -> Result<Option<Element<R>>> {
        self.next_element_at_depth(1)
    }

    // Return the raw XML of all children at or below the nominated depth
    fn children_as_string(&self, depth: u32, buffer: &mut String) -> Result<()> {
        // Read nodes at the current depth or greater
        let mut state = self.state.borrow_mut();
        let mut current_depth = depth;

        loop {
            // A strange construction, but we need to throw an error if we cannot consume all the children (e.g. malformed XML)
            // TODO idiomatic
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
    fn next_element_at_depth(&self, iter_depth: u32) -> Result<Option<Element<R>>> {
        // Read nodes until we arrive at the correct depth
        let mut state = self.state.borrow_mut();
        while let Some(node) = state.next()? {
            match node {
                // The start of an element may be interesting to the iterator
                XmlEvent::Start { name, attributes, namespace } => {
                    // Starting an element increases our depth
                    state.current_depth += 1;

                    // If we are at the correct depth we found a node of interest
                    if state.current_depth == iter_depth {
                        let element = Element { namespace, name, attributes, source: &self, depth: state.current_depth };
                        return Ok(Some(element));
                    }
                }

                // The end of an element moves back up the hierarchy
                XmlEvent::End { .. } => state.current_depth -= 1,

                // Not interested in other events when looking for elements
                _ => {}
            }

            // If we have hit the end of children at this level we terminate
            if state.current_depth < iter_depth - 1 {
                return Ok(None);
            }
        };

        // Hit the end of the document
        if state.current_depth > 0 {
            let msg = format!("documented terminated at depth {}", state.current_depth);
            let e = quick_xml::Error::UnexpectedEof(msg);
            Err(XmlError::Parser { e })
        } else {
            Ok(None)
        }
    }

    // Extracts a text element
    fn text_node(&self) -> Result<Option<String>> {
        let mut state = self.state.borrow_mut();

        // If the next event is characters, we have found our text
        if let Ok(Some(XmlEvent::Text(_text))) = state.peek() {
            // Grab the next event - we know its a Text event from the above
            match state.next() {
                Ok(Some(XmlEvent::Text(text))) => return Ok(Some(text)),
                _ => unreachable!("state.next() did not return expected XmlEvent::Text")
            }
        }

        Ok(None)
    }
}

// Wraps the XML source and current depth of iteration
struct SourceState<R: BufRead> {
    reader: Reader<R>,
    buf_event: Vec<u8>,
    buf_ns: Vec<u8>,
    next: Result<Option<XmlEvent>>,
    current_depth: u32,
}

impl<R: BufRead> SourceState<R> {
    // Wrap the reader in additional state (buffers, tree depth etc)
    fn new(reader: Reader<R>) -> SourceState<R> {
        let buf_event = Vec::with_capacity(512);
        let buf_ns = Vec::with_capacity(128);
        let mut state = SourceState { reader, buf_event, buf_ns, next: Ok(None), current_depth: 0 };
        state.next = state.fetch_next();
        state
    }

    // Returns the next event
    fn fetch_next(&mut self) -> Result<Option<XmlEvent>> {
        let reader = &mut self.reader;
        loop {
            let (ns, event) = reader.read_namespaced_event(&mut self.buf_event, &mut self.buf_ns)?;
            match event {
                // Start of an element
                Event::Start(ref e) => { return XmlEvent::start(ns, e, &reader); }

                // End of an element
                Event::End(ref e) => { return XmlEvent::end(e, &reader); }

                // Text
                Event::Text(ref t) => {
                    // TODO idiomatic
                    let event = XmlEvent::text(t, &reader);
                    if let Ok(Some(ref _t)) = event {
                        return event;
                    }
                }

                // CData
                Event::CData(ref t) => { return XmlEvent::text_from_cdata(t, &reader); }

                // The end of the document
                Event::Eof => { return Ok(None); }

                // Ignore everything else
                _ => {}
            }
        }
    }

    // Returns the next interesting event or None if no more events are found
    fn next(&mut self) -> Result<Option<XmlEvent>> {
        let next = mem::replace(&mut self.next, Ok(None));
        self.next = self.fetch_next();
        next
    }

    // Peeks the next event (does not advance)
    // Callers should call next() to consume the event to move on
    fn peek(&mut self) -> &Result<Option<XmlEvent>> {
        &self.next
    }
}

/// An element (specifically, XML element start tag)
pub(crate) struct Element<'a, R: BufRead> {
    /// Qualified name of the element.
    pub name: String,

    /// The namespace mapping at this point of the document.
    pub namespace: Option<NS>,

    /// A list of attributes associated with the element.
    pub attributes: Vec<NameValue>,

    // Depth of this element
    depth: u32,

    // The underlying source of XML events
    source: &'a ElementSource<R>,
}

impl<'a, R: BufRead> Element<'a, R> {
    /// Returns the value for an attribute if it exists
    pub(crate) fn attr_value(&self, name: &str) -> Option<String> {
        self.attributes.iter()
            .find(|a| a.name == name)
            .map(|a| a.value.clone())
    }

    /// If the first child of the current node is XML characters, then it is returned as a `String` otherwise `None`.
    pub(crate) fn child_as_text(&self) -> Result<Option<String>> {
        self.source.text_node()
    }

    /// Returns an iterator over children of this element (i.e. descends a level in the hierarchy)
    pub(crate) fn children(&self) -> ElementIter<R> {
        ElementIter { source: &self.source, depth: self.depth + 1 }
    }

    /// Concatenates the children of this node into a string
    ///
    /// NOTE: the input stream is parsed then re-serialised so the output will not be identical to the input
    pub(crate) fn children_as_string(&self) -> Result<Option<String>> {
        // Fill the buffer with the XML content below this element
        let mut buffer = String::new();
        self.source.children_as_string(self.depth + 1, &mut buffer)?;

        Ok(Some(buffer))
    }

    /// Returns the namespace + tag name for this element
    pub(crate) fn ns_and_tag(&self) -> (&Option<NS>, &str) {
        (&self.namespace, &self.name)
    }
}

impl<'a, R: BufRead> Debug for Element<'a, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    type Item = Result<Element<'a, R>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next_element_at_depth(self.depth).transpose()
    }
}

/// Set of automatically recognised namespaces
#[derive(Debug, PartialEq)]
pub(crate) enum NS {
    // http://purl.org/rss/1.0/modules/content/
    Content,
    // http://purl.org/dc/elements/1.1/
    DublinCore,
}

impl NS {
    fn parse(s: &str) -> Option<NS> {
        match s {
            "http://purl.org/rss/1.0/modules/content/" => Some(NS::Content),
            "http://purl.org/dc/elements/1.1/" => Some(NS::DublinCore),
            _ => None
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
    Parser { e: quick_xml::Error }
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmlError::Parser { e } => write!(f, "Parser error: {}", e)
        }
    }
}

impl Error for XmlError {}

impl From<quick_xml::Error> for XmlError {
    fn from(e: quick_xml::Error) -> Self {
        XmlError::Parser { e }
    }
}

// Abstraction over the underlying XML reader event model
enum XmlEvent {
    // An XML start tag
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

// TODO how do we handle errors at this level?
impl XmlEvent {
    // Creates a new event corresponding to an XML end-tag
    fn end<R: BufRead>(event: &BytesEnd, reader: &Reader<R>) -> Result<Option<XmlEvent>> {
        // Parse the name
        let name = XmlEvent::parse_name(event.name(), reader);

        Ok(Some(XmlEvent::End { name }))
    }

    // Extracts the element name, dropping the namespace prefix if present
    fn parse_name<R: BufRead>(bytes: &[u8], reader: &Reader<R>) -> String {
        let mut name = reader.decode(bytes).unwrap().to_owned();
        if let Some(index) = name.find(':') {
            name = name.split_off(index + 1);
        }
        name
    }
    // Creates a new event corresponding to an XML start-tag
    fn start<R: BufRead>(ns: Option<&[u8]>, event: &BytesStart, reader: &Reader<R>) -> Result<Option<XmlEvent>> {
        // Parse the namespace
        let namespace = ns.map(|bytes| reader.decode(bytes).unwrap())
            .and_then(|s| NS::parse(s));

        // Parse the name
        let name = XmlEvent::parse_name(event.name(), reader);

        // Parse the attributes
        let attributes = event.attributes()
            .map(|a| {
                let a = a.unwrap();
                let name = reader.decode(a.key).unwrap();
                let value = reader.decode(a.value.as_ref()).unwrap();

                NameValue { name: name.into(), value: value.into() }
            })
            .collect::<Vec<NameValue>>();

        Ok(Some(XmlEvent::Start { namespace, name, attributes }))
    }

    // Creates a new event corresponding to an XML text node
    fn text<R: BufRead>(text: &BytesText, reader: &Reader<R>) -> Result<Option<XmlEvent>> {
        let text = text.unescape_and_decode(reader)?;
        if text.is_empty() {
            Ok(None)
        } else {
            Ok(Some(XmlEvent::Text(text)))
        }
    }

    // Creates a new event corresponding to an XML CData tag
    fn text_from_cdata<R: BufRead>(text: &BytesText, reader: &Reader<R>) -> Result<Option<XmlEvent>> {
        let text = reader.decode(text)?;
        Ok(Some(XmlEvent::Text(text.into())))
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
fn append_element_start(buffer: &mut String, name: &str, attributes: &Vec<NameValue>) {
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
