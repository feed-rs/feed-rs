use crate::model::{Feed, Text, Link, Person, Entry, Generator, Category};
use crate::parser;
use crate::util::test;

// Verify we can parse the example contained in the RSS 2.0 specification
#[test]
fn test_spec_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_spec_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::new()
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("Scripting News".to_owned()))
        .link(Link::new("http://www.scripting.com/".to_owned()))
        .description(Text::new("A weblog about scripting and stuff like that.".to_owned()))
        .language("en-us")
        .rights(Text::new("Copyright 1997-2002 Dave Winer".to_owned()))
        .updated_rfc2822("Mon, 30 Sep 2002 11:00:00 GMT")
        .generator(Generator::new("Radio UserLand v8.0.5".to_owned()))
        .category(Category::new("1765".to_owned())
            .scheme("Syndic8"))
        .contributor(Person::new("managingEditor".to_owned())
            .email("dave@userland.com"))
        .contributor(Person::new("webMaster".to_owned())
            .email("dave@userland.com"))
        .ttl(40)
        .entry(Entry::new()
            .summary(Text::new(r#"Joshua Allen: <a href="http://www.netcrucible.com/blog/2002/09/29.html#a243">Who
                loves namespaces?</a>"#.to_owned()))
            .published_rfc3339("Sun, 29 Sep 2002 19:59:01 GMT")
            .id("http://scriptingnews.userland.com/backissues/2002/09/29#When:12:59:01PM")
            .updated_rfc2822("Mon, 30 Sep 2002 11:00:00 GMT"))
        .entry(Entry::new()
            .summary(Text::new(r#"<a href="http://www.docuverse.com/blog/donpark/2002/09/29.html#a68">Don Park</a>:
                "It is too easy for engineer to anticipate too much and XML Namespace is a frequent host of
                over-anticipation.""#.to_owned()))
            .published_rfc3339("Mon, 30 Sep 2002 01:52:02 GMT")
            .id("http://scriptingnews.userland.com/backissues/2002/09/29#When:6:52:02PM")
            .updated_rfc2822("Mon, 30 Sep 2002 11:00:00 GMT"));

    // Check
    assert_eq!(actual, expected);
}
