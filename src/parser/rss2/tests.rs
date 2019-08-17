use crate::model::{Feed, Text, Link, Person, Entry, Generator, Category, Image, Content};
use crate::parser;
use crate::util::test;

// Basic example from various sources (Wikipedia etc).
#[test]
fn test_example_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_example_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::default()
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("RSS Title".to_owned()))
        .description(Text::new("This is an example of an RSS feed".to_owned()))
        .link(Link::new("http://www.example.com/main.html".to_owned()))
        .updated_rfc2822("Mon, 06 Sep 2010 00:01:00 +0000")
        .published_rfc2822("Sun, 06 Sep 2009 16:20:00 +0000")
        .ttl(1800)
        .entry(Entry::default()
            .title(Text::new("Example entry".to_owned()))
            .summary(Text::new("Here is some text containing an interesting description.".to_owned()))
            .link(Link::new("http://www.example.com/blog/post/1".to_owned()))
            .id("7bd204c6-1655-4c27-aeee-53f933c5395f")
            .published_rfc2822("Sun, 06 Sep 2009 16:20:00 +0000")
            .updated_rfc2822("Mon, 06 Sep 2010 00:01:00 +0000"));       // copy from feed

    // Check
    assert_eq!(actual, expected);
}

// More detailed feed from NASA
#[test]
fn test_example_2() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_example_2.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected: Feed = Feed::default()
        .id(actual.id.as_ref())     // not present in the test data
        .updated(actual.updated)    // not present in the test data
        .title(Text::new("NASA Breaking News".to_owned()))
        .description(Text::new("A RSS news feed containing the latest NASA news articles and press releases.".to_owned()))
        .link(Link::new("http://www.nasa.gov/".to_owned()))
        .language("en-us")
        .contributor(Person::new("managingEditor".to_owned())
            .email("jim.wilson@nasa.gov"))
        .contributor(Person::new("webMaster".to_owned())
            .email("brian.dunbar@nasa.gov"))
        .entry(Entry::default()
            .title(Text::new("NASA Television to Broadcast Space Station Departure of Cygnus Cargo Ship".to_owned()))
            .link(Link::new("http://www.nasa.gov/press-release/nasa-television-to-broadcast-space-station-departure-of-cygnus-cargo-ship".to_owned()))
            .summary(Text::new(r#"More than three months after delivering several tons of supplies and scientific experiments to
                the International Space Station, Northrop Grumman’s Cygnus cargo spacecraft, the SS Roger Chaffee, will
                depart the orbiting laboratory Tuesday, Aug. 6."#.to_owned()))
            .content(Content::default()
                .src("http://www.nasa.gov/sites/default/files/styles/1x1_cardfeed/public/thumbnails/image/47616261882_4bb534d293_k.jpg?itok=Djjjs81t")
                .length(892854)
                .content_type("image/jpeg"))
            .id("http://www.nasa.gov/press-release/nasa-television-to-broadcast-space-station-departure-of-cygnus-cargo-ship")
            .published_rfc2822("Thu, 01 Aug 2019 16:15 EDT")
            .updated(actual.updated));

    // Check
    assert_eq!(actual, expected);
}

// News feed from the New Yorker
#[test]
fn test_example_3() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_example_3.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::default()
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("News, Politics, Opinion, Commentary, and Analysis".to_owned()))
        .description(Text::new("In-depth reporting, commentary on breaking news, political analysis, and opinion from The New\n            Yorker.".to_owned()))
        .link(Link::new("https://www.newyorker.com/news".to_owned()))
        .rights(Text::new("© Condé Nast 2019".to_owned()))
        .language("en")
        .updated_rfc2822("Tue, 06 Aug 2019 10:46:05 +0000")
        .entry(Entry::default()
            .title(Text::new("How a Historian Uncovered Ronald Reagan’s Racist Remarks to Richard Nixon".to_owned()))
            .link(Link::new("https://www.newyorker.com/news/q-and-a/how-a-historian-uncovered-ronald-reagans-racist-remarks-to-richard-nixon".to_owned()))
            .id("5d420f3abfe6c20008d5eaad")
            .summary(Text::new("Isaac Chotiner talks with the historian Tim Naftali, who published the text and audio of a\n                taped call, from 1971, in which Reagan described the African delegates to the U.N. in luridly racist\n                terms.".to_owned()))
            .category(Category::new("News / Q. & A.".to_owned()))
            .published_rfc2822("Fri, 02 Aug 2019 15:35:34 +0000")
            .updated(actual.updated));

    // Check
    assert_eq!(actual, expected);
}

// Structured event data on earthquakes
#[test]
fn test_example_4() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_example_4.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::default()
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("Earthquakes today".to_owned()))
        .link(Link::new("http://www.earthquakenewstoday.com".to_owned()))
        .description(Text::new("Current and latest world earthquakes breaking news, activity and articles today".to_owned()))
        .updated_rfc2822("Tue, 06 Aug 2019 05:01:15 +0000")
        .language("en-US")
        .generator(Generator::new("https://wordpress.org/?v=5.1.1".to_owned()))
        .entry(Entry::default()
            .title(Text::new("Minor earthquake, 3.5 mag was detected near Aris in Greece".to_owned()))
            .link(Link::new("http://www.earthquakenewstoday.com/2019/08/06/minor-earthquake-3-5-mag-was-detected-near-aris-in-greece/".to_owned()))
            .published_rfc2822("Tue, 06 Aug 2019 05:01:15 +0000")
            .category(Category::new("Earthquake breaking news".to_owned()))
            .category(Category::new("Minor World Earthquakes Magnitude -3.9".to_owned()))
            .category(Category::new("Spárti".to_owned()))
            .id("http://www.earthquakenewstoday.com/2019/08/06/minor-earthquake-3-5-mag-was-detected-near-aris-in-greece/")

            .summary(Text::new("A minor earthquake magnitude 3.5 (ml/mb) strikes near Kalamáta, Trípoli, Pýrgos, Spárti, Filiatrá, Messíni, Greece on Tuesday. The temblor has occurred at 03:46:56/3:46 am (local time epicenter) at a depth of 10 km (6 miles). How did you react? Did you feel it?".to_owned()))
            .updated(actual.updated));

    // Check
    assert_eq!(actual, expected);
}

// Tech news from Ars Technica
#[test]
fn test_example_5() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_example_5.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::default()
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("Ars Technica".to_owned()))
        .link(Link::new("https://arstechnica.com".to_owned()))
        .description(Text::new("Serving the Technologist for more than a decade. IT news, reviews, and analysis.".to_owned()))
        .updated_rfc2822("Tue, 06 Aug 2019 00:03:56 +0000")
        .language("en-US")
        .generator(Generator::new("https://wordpress.org/?v=4.8.3".to_owned()))
        .logo(Image::new("https://cdn.arstechnica.net/wp-content/uploads/2016/10/cropped-ars-logo-512_480-32x32.png".to_owned())
            .title("Ars Technica")
            .link("https://arstechnica.com")
            .width(32)
            .height(32))
        .entry(Entry::default()
            .title(Text::new("Apple isn’t the most cash-rich company in the world anymore, but it doesn’t matter".to_owned()))
            .link(Link::new("https://arstechnica.com/?p=1546121".to_owned()))
            .published_rfc2822("Mon, 05 Aug 2019 23:11:09 +0000")
            .category(Category::new("Tech".to_owned()))
            .category(Category::new("alphabet".to_owned()))
            .category(Category::new("apple".to_owned()))
            .category(Category::new("google".to_owned()))
            .id("https://arstechnica.com/?p=1546121")
            .summary(Text::new("Alphabet has $117 billion in cash on hand.".to_owned()))
            .updated(actual.updated));

    // Check
    assert_eq!(actual, expected);
}

// Verify we can parse the example contained in the RSS 2.0 specification
#[test]
fn test_spec_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_spec_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::default()
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
        .entry(Entry::default()
            .summary(Text::new(r#"Joshua Allen: <a href="http://www.netcrucible.com/blog/2002/09/29.html#a243">Who
                loves namespaces?</a>"#.to_owned()))
            .published_rfc2822("Sun, 29 Sep 2002 19:59:01 GMT")
            .id("http://scriptingnews.userland.com/backissues/2002/09/29#When:12:59:01PM")
            .updated_rfc2822("Mon, 30 Sep 2002 11:00:00 GMT"))       // copy from feed
        .entry(Entry::default()
            .summary(Text::new(r#"<a href="http://www.docuverse.com/blog/donpark/2002/09/29.html#a68">Don Park</a>:
                "It is too easy for engineer to anticipate too much and XML Namespace is a frequent host of
                over-anticipation.""#.to_owned()))
            .published_rfc2822("Mon, 30 Sep 2002 01:52:02 GMT")
            .id("http://scriptingnews.userland.com/backissues/2002/09/29#When:6:52:02PM")
            .updated_rfc2822("Mon, 30 Sep 2002 11:00:00 GMT"));      // copy from feed

    // Check
    assert_eq!(actual, expected);
}
