# feed-rs

[![Build Status](https://travis-ci.org/feed-rs/feed-rs.svg?branch=master)](https://travis-ci.org/feed-rs/feed-rs.svg?branch=master)
[![Crates.io Status](https://img.shields.io/crates/v/feed-rs.svg)](https://crates.io/crates/feed-rs)

Library for parsing Atom and RSS.

[Documentation](https://docs.rs/atom_syndication/)

## Usage

Add the dependency to your `Cargo.toml`.

```toml
[dependencies]
feed-rs = "0.2.0"
```

Or, if you want Serde include the feature like this:

```toml
[dependencies]
feed-rs = { version = "0.2.0", features = ["serde"] }
```

## Reading

A feed can be parsed from any object that implements the `Read` trait.

```rust
use feed_rs::parser;
let xml = r#"
<feed>
   <title type="text">sample feed</title>
   <updated>2005-07-31T12:29:29Z</updated>
   <id>feed1</id>
   <entry>
       <title>sample entry</title>
       <id>entry1</id>
   </entry>
</feed>
"#;
let feed = parser::parse(xml.as_bytes()).unwrap();
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.