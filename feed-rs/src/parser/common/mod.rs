use crate::model::{Link, LinkTarget, Text};
use crate::xml::Element;
use std::io::BufRead;

// Handles <link>
pub(crate) fn handle_link<R: BufRead>(target: Option<LinkTarget>, element: Element<R>) -> Option<Link> {
    element.child_as_text().map(|s| {
        let mut link = Link::new(s, element.xml_base.as_ref());
        link.target = target;
        link
    })
}

// Handles <title>, <description> etc
pub(crate) fn handle_text<R: BufRead>(element: Element<R>) -> Option<Text> {
    if let Ok(Some(text)) = element.children_as_string() {
        Some(Text::new(text))
    } else {
        None
    }
}
