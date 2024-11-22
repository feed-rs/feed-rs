#[allow(unused_imports)]
use feed_rs::parser;

// Bad input adapted from Python's feedparser library: https://github.com/kurtmckee/feedparser

#[test]
#[cfg(feature = "sanitize")]
fn test_sanitize_atom_plain_text() {
    let source = r#"
<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">

  <title><![CDATA[<script>title</script>]]></title>
  <subtitle><![CDATA[<script>subtitle</script>]]></subtitle>
  <link href="http://example.org/"/>
  <updated>2003-12-13T18:30:02Z</updated>
  <author>
    <name>Erika &lt;tag /&gt; Mustermann</name>
  </author>
  <id>urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6</id>

  <entry>
    <title type="text">&lt;script&gt;alert('xyzzy');&lt;/script&gt;title</title>
    <link href="http://example.org/2003/12/13/atom03"/>
    <content><p style="background-color: black;">Sphinx of black quartz, hear my vow!</p><style>p { color: white; }</style></content>
    <id>urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a</id>
    <updated>2003-12-13T18:30:02Z</updated>
  </entry>

</feed>
    "#;
    let feed = parser::parse(source.as_bytes()).unwrap();
    // Explicit plain/text
    let feed_title = feed.title.unwrap();
    // Implicit plain/text
    let feed_description = feed.description.unwrap();
    // No HTML supported
    let feed_author = feed.authors[0].clone();

    let entry = feed.entries[0].clone();
    // Explicit plain text
    let entry_title = entry.title.unwrap();
    // Implicit plain text
    let entry_content = entry.content.unwrap();

    assert_eq!(feed_description.content, r#"<script>subtitle</script>"#);
    assert_eq!(feed_description.content_type.as_str(), "text/plain");
    assert_eq!(feed_title.content, r#"<script>title</script>"#);
    assert_eq!(feed_title.content_type.as_str(), "text/plain");
    assert_eq!(feed_author.name, r#"Erika <tag /> Mustermann"#);

    assert_eq!(entry_title.content, "<script>alert('xyzzy');</script>title");
    assert_eq!(entry_title.content_type.as_str(), "text/plain");
    assert_eq!(
        entry_content.body.unwrap(),
        r#"<p style="background-color: black;">Sphinx of black quartz, hear my vow!</p><style>p { color: white; }</style>"#
    );
    assert_eq!(entry_content.content_type.as_str(), "text/plain");
}

#[test]
#[cfg(feature = "sanitize")]
fn test_sanitize_atom() {
    let source = r#"
<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">

  <title type="text/html" mode="escaped">&lt;img src="http://www.ragingplatypus.com/i/cam-full.jpg" onkeydown="location.href='http://www.ragingplatypus.com/';" /></title>
  <subtitle type="text/html"><div><faketag /><fake>Safe</fake> subtitle</div></subtitle>
  <link href="http://example.org/"/>
  <updated>2003-12-13T18:30:02Z</updated>
  <author>
    <name>Erika Mustermann</name>
  </author>
  <id>urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6</id>
  <rights type="text/html" mode="escaped">
&lt;!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">

&lt;html xmlns="http://www.w3.org/1999/xhtml">
&lt;head>
&lt;title>Crazy&lt;/title>
&lt;/head>
&lt;body    notRealAttribute="value"onload="executeMe();"foo="bar"

>
&lt;/body>

&lt;/html>
  </rights>

  <entry>
    <title type="text/html" mode="escaped"><a href="http://example.com">Safe&lt;iframe src="http://www.example.com/">&lt;/iframe> title</a></title>
    <link href="http://example.org/2003/12/13/atom03"/>
    <content type="text/html"><p style="background-color: black;">Sphinx of black quartz, hear my vow!</p><style>p { color: white; }</style></content>
    <id>urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a</id>
    <updated>2003-12-13T18:30:02Z</updated>
    <summary type="text/html">Safe&lt;script type="text/javascript">location.href='http:/'+'/example.com/';&lt;/script> summary.</summary>
  </entry>

</feed>
    "#;
    let feed = parser::parse(source.as_bytes()).unwrap();
    let feed_title = feed.title.unwrap();
    let feed_description = feed.description.unwrap();
    let feed_rights = feed.rights.unwrap();

    let entry = feed.entries[0].clone();
    let entry_title = entry.title.unwrap();
    let entry_summary = entry.summary.unwrap();
    let entry_content = entry.content.unwrap();

    assert_eq!(feed_title.content, r#"<img src="http://www.ragingplatypus.com/i/cam-full.jpg">"#);
    assert_eq!(feed_title.content_type.as_str(), "text/html");
    assert_eq!(feed_description.content, r#"<div>Safe subtitle</div>"#);
    assert_eq!(feed_description.content_type.as_str(), "text/html");
    assert_eq!(feed_rights.content, "\n\n\n\nCrazy\n\n\n\n\n");
    assert_eq!(feed_description.content_type.as_str(), "text/html");

    // noopener/noreferrer inserted by ammonia
    assert_eq!(entry_title.content, r#"<a href="http://example.com" rel="noopener noreferrer">Safe title</a>"#);
    assert_eq!(entry_title.content_type.as_str(), "text/html");
    assert_eq!(entry_summary.content, "Safe summary.");
    assert_eq!(entry_summary.content_type.as_str(), "text/html");
    assert_eq!(entry_content.body.unwrap(), "<p>Sphinx of black quartz, hear my vow!</p>");
    assert_eq!(entry_content.content_type.as_str(), "text/html");
}

#[test]
#[cfg(feature = "sanitize")]
fn test_sanitize_rss2() {
    let source = r#"
<rss version="2.0"
  xmlns:content="http://purl.org/rss/1.0/modules/content/"
>
  <channel>
  <title>2 &gt; 1 &lt;img src="http://www.ragingplatypus.com/i/cam-full.jpg" onkeydown="location.href='http://www.ragingplatypus.com/';" /></title>
  <link href="http://example.org/"/>
  <item>
    <title>Safe&lt;iframe src="http://www.example.com/">&lt;/iframe> title</title>
    <content:encoded><![CDATA[<p onclick="alert('xyzzy');">Safe <a href="http://example.com/">content</a></p>]]></content:encoded>
    <description>safe&lt;link rel="stylesheet" type="text/css" href="http://example.com/evil.css"> summary</description>
  </item>
  </channel>
</rss>
    "#;
    let feed = parser::parse(source.as_bytes()).unwrap();
    let feed_title = feed.title.unwrap();

    let entry = feed.entries[0].clone();
    let entry_title = entry.title.unwrap();
    let entry_summary = entry.summary.unwrap();
    let entry_content = entry.content.unwrap();

    /* DANGER ZONE: text/plain is not escaped! */
    assert_eq!(
        feed_title.content,
        r#"2 > 1 <img src="http://www.ragingplatypus.com/i/cam-full.jpg" onkeydown="location.href='http://www.ragingplatypus.com/';" />"#
    );
    assert_eq!(feed_title.content_type.as_str(), "text/plain");
    assert_eq!(entry_title.content, r#"Safe<iframe src="http://www.example.com/"></iframe> title"#);
    assert_eq!(entry_title.content_type.as_str(), "text/plain");

    assert_eq!(entry_summary.content, r#"safe summary"#);
    assert_eq!(entry_summary.content_type.as_str(), "text/html");

    // noopener/noreferrer inserted by ammonia
    assert_eq!(
        entry_content.body.unwrap(),
        r#"<p>Safe <a href="http://example.com/" rel="noopener noreferrer">content</a></p>"#
    );
    assert_eq!(entry_content.content_type.as_str(), "text/html");
}

#[test]
#[cfg(feature = "sanitize")]
fn test_sanitize_rss1() {
    let source = r#"
<rdf:RDF
    xmlns="http://purl.org/rss/1.0/"
    xmlns:content="http://purl.org/rss/1.0/modules/content/"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
>
<channel rdf:about="http://example.com/index.rdf">
<title>&lt;example feed&gt;</title>
</channel>
<item>
  <title><![CDATA[<example title>]]></title>
  <link>http://example.com/1</link>
  <dc:description>&lt;example description&gt;</dc:description>
  <content:encoded>&lt;p style="background-color: black;"&gt;Sphinx of black quartz, hear my vow!&lt;/p&gt;&lt;style&gt;p { color: white; }&lt;/style&gt;&lt;/content&gt;</content:encoded>
</item>
</rdf:RDF>
"#;
    let feed = parser::parse(source.as_bytes()).unwrap();
    println!("{:?}", &feed);
    let feed_title = feed.title.unwrap();

    let entry = feed.entries[0].clone();
    let entry_title = entry.title.unwrap();
    let entry_summary = entry.summary.unwrap();
    let entry_content = entry.content.unwrap();

    /* DANGER ZONE: text/plain is not escaped! */
    assert_eq!(feed_title.content, "<example feed>");
    assert_eq!(feed_title.content_type.as_str(), "text/plain");
    assert_eq!(entry_title.content, "<example title>");
    assert_eq!(entry_title.content_type.as_str(), "text/plain");
    assert_eq!(entry_summary.content, "<example description>");
    assert_eq!(entry_summary.content_type.as_str(), "text/plain");

    assert_eq!(entry_content.body.unwrap(), "<p>Sphinx of black quartz, hear my vow!</p>");
    assert_eq!(entry_content.content_type.as_str(), "text/html");
}

#[test]
#[cfg(feature = "sanitize")]
fn test_sanitize_json() {
    let source = r#"
{
    "version": "https://jsonfeed.org/version/1.1",
    "title": "My Example Feed",
    "home_page_url": "https://example.org/",
    "feed_url": "https://example.org/feed.json",
    "items": [
        {
            "id": "1",
            "content_text": "<script>alert(\"xyzzy\")</script>",
            "content_html": "<script>alert(\"xyzzy\")</script><p>Hello, world!</p>",
            "url": "https://example.org/initial-post"
        }
    ]
}"#;
    let feed = parser::parse(source.as_bytes()).unwrap();
    println!("{:?}", &feed);

    let entry = feed.entries[0].clone();
    let entry_summary = entry.summary.unwrap();
    let entry_content = entry.content.unwrap();

    assert_eq!(entry_summary.content, r#"<script>alert("xyzzy")</script>"#);
    assert_eq!(entry_summary.content_type.as_str(), "text/plain");
    assert_eq!(entry_content.body.unwrap(), "<p>Hello, world!</p>");
    assert_eq!(entry_content.content_type.as_str(), "text/html");
}
