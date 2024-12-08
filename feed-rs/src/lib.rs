#![doc(html_root_url = "https://docs.rs/rss/")]
//! This crate provides a parser and common data model over Atom, RSS and JSON Feed content.
//!
//! The parser will automatically detect the type of content (XML vs. JSON) and the feed format (Atom vs. RSS).
//!
//! It populates a unified data model over all feed formats, attempting to find a balance between:
//! * convenience of a single field where semantic equivalence exists e.g. "description" from RSS 2 and "subtitle" from Atom are semantically equivalent and mapped to the same field
//! * the real world where mandatory fields may not be specified and data may not be in the correct form
//!
//! The parser errs on the side of leniency with the outcome that certain fields are represented as `Option<T>` in the model, even though they may be mandatory in one of the specifications.
//!
//! It uses [quick-xml](https://crates.io/crates/quick-xml) - a light-weight, streaming XML parser to minimise memory usage and avoids copying (clone) where possible.
//!
//! # Usage
//!
//! The `parser::parse` method accepts any source that implements the `Read` trait.
//! For example, to process a string:
//!
//! ```rust,no_run
//! use feed_rs::parser;
//!
//! let example_rss = r#"<?xml version="1.0" encoding="UTF-8" ?>
//!   <rss version="2.0">
//!     <channel>
//!       <title>RSS Title</title>
//!       <description>This is an example of an RSS feed</description>
//!       <link>http://www.example.com/main.html</link>
//!       <lastBuildDate>Mon, 06 Sep 2010 00:01:00 +0000</lastBuildDate>
//!       <pubDate>Sun, 06 Sep 2009 16:20:00 +0000</pubDate>
//!       <ttl>1800</ttl>
//!
//!       <item>
//!         <title>Example entry</title>
//!         <description>Here is some text containing an interesting description.</description>
//!         <link>http://www.example.com/blog/post/1</link>
//!         <guid isPermaLink="true">7bd204c6-1655-4c27-aeee-53f933c5395f</guid>
//!         <pubDate>Sun, 06 Sep 2009 16:20:00 +0000</pubDate>
//!       </item>
//!
//!     </channel>
//!   </rss>"#;
//!
//! let feed = parser::parse(example_rss.as_bytes()).unwrap();
//!```
//!
//! Or from a file:
//!
//! ```rust,no_run
//! use std::fs::File;
//! use std::io::BufReader;
//! use feed_rs::parser;
//!
//! let file = File::open("example.xml").unwrap();
//! let feed = parser::parse(BufReader::new(file)).unwrap();
//! ```
//!
//! ## Parser configuration
//!
//! The default parser configuration provides sensible defaults, such as lenient timestamp parsing. You may change this behaviour and configure other parser behaviour with the `parser::Builder`.
//! For example, to enable content sanitisation:
//!
//! ```rust,no_run
//! use std::fs::File;
//! use std::io::BufReader;
//! use feed_rs::parser;
//!
//! let file = File::open("example.xml").unwrap();
//! let parser = parser::Builder::new()
//!     .sanitize_content(true)
//!     .build();
//! let feed = parser.parse(BufReader::new(file)).unwrap();
//! ```

// TODO review the Rust doc guidelines and fix up links
// TODO improve tests with Coverage analysis e.g. https://github.com/mozilla/grcov

#![forbid(unsafe_code)]
// Standard names like MediaRSS and JSON are used throughout this crate
#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate serde;
extern crate core;

mod util;
mod xml;

pub mod model;
pub mod parser;
