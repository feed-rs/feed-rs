use crate::model::*;
use crate::parser;
use crate::util::test;

// Trimmed example of RSS 0.91 from the specification at http://backend.userland.com/rss091
#[test]
fn test_0_91_spec_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_0.91_spec_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let entry0 = actual.entries.get(0).unwrap();
    let entry1 = actual.entries.get(1).unwrap();
    let expected = Feed::new(FeedType::RSS0)
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("WriteTheWeb".into()))
        .link(Link::new("http://writetheweb.com".into()))
        .description(Text::new("News for web users that write back".into()))
        .language("en-us")
        .rights(Text::new("Copyright 2000, WriteTheWeb team.".into()))
        .contributor(Person::new("managingEditor").email("editor@writetheweb.com"))
        .contributor(Person::new("webMaster").email("webmaster@writetheweb.com"))
        .logo(Image::new("http://writetheweb.com/images/mynetscape88.gif".into())
            .title("WriteTheWeb")
            .link("http://writetheweb.com")
            .width(88)
            .height(31)
            .description("News for web users that write back"))
        .updated(actual.updated)        // not in source data
        .entry(Entry::default()
            .title(Text::new("Giving the world a pluggable Gnutella".into()))
            .link(Link::new("http://writetheweb.com/read.php?item=24".into()))
            .summary(Text::new("WorldOS is a framework on which to build programs that work like Freenet or Gnutella -allowing\n                distributed applications using peer-to-peer routing.\n            ".into()))
            .id(entry0.id.as_ref())     // not in source data
            .updated(entry0.updated))   // not in source data
        .entry(Entry::default()
            .title(Text::new("Syndication discussions hot up".into()))
            .link(Link::new("http://writetheweb.com/read.php?item=23".into()))
            .summary(Text::new("After a period of dormancy, the Syndication mailing list has become active again, with\n                contributions from leaders in traditional media and Web syndication.\n            ".into()))
            .id(entry1.id.as_ref())     // not in source data
            .updated(entry1.updated)); // not in source data

    // Check
    assert_eq!(actual, expected);
}

// Verifies that we can handle non-UTF8 streams
#[test]
fn test_0_91_encoding_1() {
    let test_data = test::fixture_as_raw("rss_0.91_encoding_1.xml");
    let feed = parser::parse(test_data.as_slice()).unwrap();
    assert_eq!(feed.description.unwrap().content, "Dicas-L: Informações Úteis para Administradores de Sistemas");
}

// Verifies that we can handle non-UTF8 streams
#[test]
fn test_0_91_encoding_2() {
    let test_data = test::fixture_as_raw("rss_0.91_encoding_2.xml");
    let feed = parser::parse(test_data.as_slice()).unwrap();
    assert_eq!(feed.title.unwrap().content, "Tribunal de Justiça do Estado do Rio Grande do Sul");
    assert!(feed.entries[0].title.as_ref().unwrap().content.contains("atuação"));
    assert!(feed.entries[0].summary.as_ref().unwrap().content.contains("prevenção"));
}

// Trimmed example of RSS 0.92 from the specification at http://backend.userland.com/rss092
#[test]
fn test_0_92_spec_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_0.92_spec_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let entry0 = actual.entries.get(0).unwrap();
    let entry1 = actual.entries.get(1).unwrap();
    let entry2 = actual.entries.get(2).unwrap();
    let expected = Feed::new(FeedType::RSS0)
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("Dave Winer: Grateful Dead".into()))
        .link(Link::new("http://www.scripting.com/blog/categories/gratefulDead.html".into()))
        .description(Text::new("A high-fidelity Grateful Dead song every day. This is where we're experimenting with\n            enclosures on RSS news items that download when you're not using your computer. If it works (it will)\n            it will be the end of the Click-And-Wait multimedia experience on the Internet.\n        ".into()))
        .updated_rfc2822("Fri, 13 Apr 2001 19:23:02 GMT")
        .contributor(Person::new("managingEditor").email("dave@userland.com (Dave Winer)"))
        .contributor(Person::new("webMaster").email("dave@userland.com (Dave Winer)"))
        .entry(Entry::default()
            .summary(Text::new("Kevin Drennan started a <a href=\"http://deadend.editthispage.com/\">Grateful\n                Dead Weblog</a>. Hey it's cool, he even has a <a href=\"http://deadend.editthispage.com/directory/61\">directory</a>.\n                <i>A Frontier 7 feature.</i>\n            ".into()))
            .id(entry0.id.as_ref())     // not in source data
            .updated(entry0.updated))   // not in source data
        .entry(Entry::default()
            .summary(Text::new("<a href=\"http://arts.ucsc.edu/GDead/AGDL/other1.html\">The Other One</a>,
                live instrumental, One From The Vault. Very rhythmic very spacy, you can listen to it many times, and
                enjoy something new every time.\n            ".into()))
            .content(Content::default()
                .src("http://www.scripting.com/mp3s/theOtherOne.mp3")
                .length(6666097)
                .content_type("audio/mpeg"))
            .id(entry1.id.as_ref())     // not in source data
            .updated(entry1.updated)   // not in source data
            .media(MediaObject::new()
                .content(MediaContent::new()
                    .url("http://www.scripting.com/mp3s/theOtherOne.mp3")
                    .content_type("audio/mpeg")
                    .size(6666097))))
        .entry(Entry::default()
            .summary(Text::new("This is a test of a change I just made. Still diggin..".into()))
            .id(entry2.id.as_ref())     // not in source data
            .updated(entry2.updated)); // not in source data

    // Check
    assert_eq!(actual, expected);
}
