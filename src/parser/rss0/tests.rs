use crate::model::{Category, Content, Entry, Feed, Generator, Image, Link, Person, Text};
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
    let expected = Feed::default()
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("WriteTheWeb".to_owned()))
        .link(Link::new("http://writetheweb.com".to_owned()))
        .description(Text::new("News for web users that write back".to_owned()))
        .language("en-us")
        .rights(Text::new("Copyright 2000, WriteTheWeb team.".to_owned()))
        .contributor(Person::new("managingEditor".to_owned()).email("editor@writetheweb.com"))
        .contributor(Person::new("webMaster".to_owned()).email("webmaster@writetheweb.com"))
        .logo(Image::new("http://writetheweb.com/images/mynetscape88.gif".to_owned())
            .title("WriteTheWeb")
            .link("http://writetheweb.com")
            .width(88)
            .height(31)
            .description("News for web users that write back"))
        .updated(actual.updated)        // not in source data
        .entry(Entry::default()
            .title(Text::new("Giving the world a pluggable Gnutella".to_owned()))
            .link(Link::new("http://writetheweb.com/read.php?item=24".to_owned()))
            .summary(Text::new("WorldOS is a framework on which to build programs that work like Freenet or Gnutella -allowing\n                distributed applications using peer-to-peer routing.".to_owned()))
            .id(entry0.id.as_ref())     // not in source data
            .updated(entry0.updated))   // not in source data
        .entry(Entry::default()
            .title(Text::new("Syndication discussions hot up".to_owned()))
            .link(Link::new("http://writetheweb.com/read.php?item=23".to_owned()))
            .summary(Text::new("After a period of dormancy, the Syndication mailing list has become active again, with\n                contributions from leaders in traditional media and Web syndication.".to_owned()))
            .id(entry1.id.as_ref())     // not in source data
            .updated(entry1.updated));  // not in source data

    // Check
    assert_eq!(actual, expected);
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
    let expected = Feed::default()
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("Dave Winer: Grateful Dead".to_owned()))
        .link(Link::new("http://www.scripting.com/blog/categories/gratefulDead.html".to_owned()))
        .description(Text::new("A high-fidelity Grateful Dead song every day. This is where we're experimenting with\n            enclosures on RSS news items that download when you're not using your computer. If it works (it will)\n            it will be the end of the Click-And-Wait multimedia experience on the Internet.".to_owned()))
        .updated_rfc2822("Fri, 13 Apr 2001 19:23:02 GMT")
        .contributor(Person::new("managingEditor".to_owned()).email("dave@userland.com (Dave Winer)"))
        .contributor(Person::new("webMaster".to_owned()).email("dave@userland.com (Dave Winer)"))
        .entry(Entry::default()
            .summary(Text::new("Kevin Drennan started a <a href=\"http://deadend.editthispage.com/\">Grateful\n                Dead Weblog</a>. Hey it's cool, he even has a <a href=\"http://deadend.editthispage.com/directory/61\">directory</a>.\n                <i>A Frontier 7 feature.</i>".to_owned()))
            .id(entry0.id.as_ref())     // not in source data
            .updated(entry0.updated))   // not in source data
        .entry(Entry::default()
            .summary(Text::new("<a href=\"http://arts.ucsc.edu/GDead/AGDL/other1.html\">The Other One</a>,
                live instrumental, One From The Vault. Very rhythmic very spacy, you can listen to it many times, and
                enjoy something new every time.".to_owned()))
            .content(Content::default()
                .src("http://www.scripting.com/mp3s/theOtherOne.mp3")
                .length(6666097)
                .content_type("audio/mpeg"))
            .id(entry1.id.as_ref())     // not in source data
            .updated(entry1.updated))   // not in source data
        .entry(Entry::default()
            .summary(Text::new("This is a test of a change I just made. Still diggin..".to_owned()))
            .id(entry2.id.as_ref())     // not in source data
            .updated(entry2.updated));  // not in source data

    // Check
    assert_eq!(actual, expected);
}
