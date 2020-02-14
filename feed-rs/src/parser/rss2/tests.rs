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
        .title(Text::new("RSS Title".into()))
        .description(Text::new("This is an example of an RSS feed".into()))
        .link(Link::new("http://www.example.com/main.html".into()))
        .updated_rfc2822("Mon, 06 Sep 2010 00:01:00 +0000")
        .published_rfc2822("Sun, 06 Sep 2009 16:20:00 +0000")
        .ttl(1800)
        .entry(Entry::default()
            .title(Text::new("Example entry".into()))
            .summary(Text::new("Here is some text containing an interesting description.".into()))
            .link(Link::new("http://www.example.com/blog/post/1".into()))
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
        .title(Text::new("NASA Breaking News".into()))
        .description(Text::new("A RSS news feed containing the latest NASA news articles and press releases.".into()))
        .link(Link::new("http://www.nasa.gov/".into()))
        .language("en-us")
        .contributor(Person::new("managingEditor".into())
            .email("jim.wilson@nasa.gov"))
        .contributor(Person::new("webMaster".into())
            .email("brian.dunbar@nasa.gov"))
        .entry(Entry::default()
            .title(Text::new("NASA Television to Broadcast Space Station Departure of Cygnus Cargo Ship".into()))
            .link(Link::new("http://www.nasa.gov/press-release/nasa-television-to-broadcast-space-station-departure-of-cygnus-cargo-ship".into()))
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
        .title(Text::new("News, Politics, Opinion, Commentary, and Analysis".into()))
        .description(Text::new("In-depth reporting, commentary on breaking news, political analysis, and opinion from The New\n            Yorker.".into()))
        .link(Link::new("https://www.newyorker.com/news".into()))
        .rights(Text::new("© Condé Nast 2019".into()))
        .language("en")
        .updated_rfc2822("Tue, 06 Aug 2019 10:46:05 +0000")
        .entry(Entry::default()
            .title(Text::new("How a Historian Uncovered Ronald Reagan’s Racist Remarks to Richard Nixon".into()))
            .link(Link::new("https://www.newyorker.com/news/q-and-a/how-a-historian-uncovered-ronald-reagans-racist-remarks-to-richard-nixon".into()))
            .id("5d420f3abfe6c20008d5eaad")
            .summary(Text::new("Isaac Chotiner talks with the historian Tim Naftali, who published the text and audio of a\n                taped call, from 1971, in which Reagan described the African delegates to the U.N. in luridly racist\n                terms.".into()))
            .category(Category::new("News / Q. & A.".into()))
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
        .title(Text::new("Earthquakes today".into()))
        .link(Link::new("http://www.earthquakenewstoday.com".into()))
        .description(Text::new("Current and latest world earthquakes breaking news, activity and articles today".into()))
        .updated_rfc2822("Tue, 06 Aug 2019 05:01:15 +0000")
        .language("en-us")
        .generator(Generator::new("https://wordpress.org/?v=5.1.1".into()))
        .entry(Entry::default()
            .title(Text::new("Minor earthquake, 3.5 mag was detected near Aris in Greece".into()))
            .link(Link::new("http://www.earthquakenewstoday.com/2019/08/06/minor-earthquake-3-5-mag-was-detected-near-aris-in-greece/".into()))
            .published_rfc2822("Tue, 06 Aug 2019 05:01:15 +0000")
            .category(Category::new("Earthquake breaking news".into()))
            .category(Category::new("Minor World Earthquakes Magnitude -3.9".into()))
            .category(Category::new("Spárti".into()))
            .id("http://www.earthquakenewstoday.com/2019/08/06/minor-earthquake-3-5-mag-was-detected-near-aris-in-greece/")

            .summary(Text::new("A minor earthquake magnitude 3.5 (ml/mb) strikes near Kalamáta, Trípoli, Pýrgos, Spárti, Filiatrá, Messíni, Greece on Tuesday. The temblor has occurred at 03:46:56/3:46 am (local time epicenter) at a depth of 10 km (6 miles). How did you react? Did you feel it?".into()))
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
        .title(Text::new("Ars Technica".into()))
        .link(Link::new("https://arstechnica.com".into()))
        .description(Text::new("Serving the Technologist for more than a decade. IT news, reviews, and analysis.".into()))
        .updated_rfc2822("Tue, 06 Aug 2019 00:03:56 +0000")
        .language("en-us")
        .generator(Generator::new("https://wordpress.org/?v=4.8.3".into()))
        .logo(Image::new("https://cdn.arstechnica.net/wp-content/uploads/2016/10/cropped-ars-logo-512_480-32x32.png".into())
            .title("Ars Technica")
            .link("https://arstechnica.com")
            .width(32)
            .height(32))
        .entry(Entry::default()
            .title(Text::new("Apple isn’t the most cash-rich company in the world anymore, but it doesn’t matter".into()))
            .link(Link::new("https://arstechnica.com/?p=1546121".into()))
            .published_rfc2822("Mon, 05 Aug 2019 23:11:09 +0000")
            .category(Category::new("Tech".into()))
            .category(Category::new("alphabet".into()))
            .category(Category::new("apple".into()))
            .category(Category::new("google".into()))
            .id("https://arstechnica.com/?p=1546121")
            .summary(Text::new("Alphabet has $117 billion in cash on hand.".into()))
            .updated(actual.updated));

    // Check
    assert_eq!(actual, expected);
}

// Trailers from Apple (no UUID)
#[test]
fn test_example_6() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_example_6.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::default()
        .id("8aed7f5d4466ac2c2de684823e36f0b4")     // hash of the link
        .title(Text::new("Latest Movie Trailers".into()))
        .link(Link::new("https://trailers.apple.com/".into()))
        .description(Text::new("Recently added Movie Trailers.".into()))
        .language("en-us")
        .updated_rfc3339("2020-02-07T15:30:28Z")
        .generator(Generator::new("Custom".into()))
        .rights(Text::new("2020 Apple Inc.".into()))
        .entry(Entry::default()
            .title(Text::new("Vitalina Varela - Trailer".into()))
            .link(Link::new("https://trailers.apple.com/trailers/independent/vitalina-varela".into()))
            .summary(Text::new("A film of deeply concentrated beauty, acclaimed filmmaker Pedro Costa’s VITALINA VARELA stars nonprofessional actor Vitalina Varela in an extraordinary performance based on her own life. Vitalina plays a Cape Verdean woman who has travelled to Lisbon to reunite with her husband, after two decades of separation, only to arrive mere days after his funeral. Alone in a strange forbidding land, she perseveres and begins to establish a new life. Winner of the Golden Leopard for Best Film and Best Actress at the Locarno Film Festival, as well as an official selection of the Sundance Film Festival, VITALINA VARELA is a film of shadow and whisper, a profoundly moving and visually ravishing masterpiece.".into()))
            .published_rfc3339("2020-02-06T08:00:00Z")
            .id("93c9e7fec11765c4547e537067e0155d")        // hash of the link
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
        .title(Text::new("Scripting News".into()))
        .link(Link::new("http://www.scripting.com/".into()))
        .description(Text::new("A weblog about scripting and stuff like that.".into()))
        .language("en-us")
        .rights(Text::new("Copyright 1997-2002 Dave Winer".into()))
        .updated_rfc2822("Mon, 30 Sep 2002 11:00:00 GMT")
        .generator(Generator::new("Radio UserLand v8.0.5".into()))
        .category(Category::new("1765".into())
            .scheme("Syndic8"))
        .contributor(Person::new("managingEditor".into())
            .email("dave@userland.com"))
        .contributor(Person::new("webMaster".into())
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
