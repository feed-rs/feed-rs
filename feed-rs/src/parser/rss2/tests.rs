use std::time::Duration;

use crate::model::*;
use crate::parser;
use crate::util::test;

// Basic example from various sources (Wikipedia etc).
#[test]
fn test_example_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_example_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::new(FeedType::RSS2)
        .id(actual.id.as_ref()) // not present in the test data
        .title(Text::new("RSS Title".into()))
        .description(Text::new("This is an example of an RSS feed".into()))
        .link(Link::new("http://www.example.com/main.html".into()))
        .updated_rfc2822("Mon, 06 Sep 2010 00:01:00 +0000")
        .published_rfc2822("Sun, 06 Sep 2009 16:20:00 +0000")
        .ttl(1800)
        .entry(
            Entry::default()
                .title(Text::new("Example entry".into()))
                .summary(Text::new("Here is some text containing an interesting description.".into()))
                .link(Link::new("http://www.example.com/blog/post/1".into()))
                .id("7bd204c6-1655-4c27-aeee-53f933c5395f")
                .published_rfc2822("Sun, 06 Sep 2009 16:20:00 +0000")
                .updated_rfc2822("Mon, 06 Sep 2010 00:01:00 +0000"),
        ); // copy from feed

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
    let expected: Feed = Feed::new(FeedType::RSS2)
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
            .link(Link::new("\n                http://www.nasa.gov/press-release/nasa-television-to-broadcast-space-station-departure-of-cygnus-cargo-ship\n            ".into()))
            .summary(Text::new(r#"More than three months after delivering several tons of supplies and scientific experiments to
                the International Space Station, Northrop Grumman’s Cygnus cargo spacecraft, the SS Roger Chaffee, will
                depart the orbiting laboratory Tuesday, Aug. 6.
            "#.to_owned()))
            .id("\n                http://www.nasa.gov/press-release/nasa-television-to-broadcast-space-station-departure-of-cygnus-cargo-ship\n            ")
            .published_rfc2822("Thu, 01 Aug 2019 16:15 EDT")
            .updated(actual.updated)
            .media(MediaObject::default()
                .content(MediaContent::new()
                    .url("http://www.nasa.gov/sites/default/files/styles/1x1_cardfeed/public/thumbnails/image/47616261882_4bb534d293_k.jpg?itok=Djjjs81t")
                    .content_type("image/jpeg")
                    .size(892854))));

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
    let expected = Feed::new(FeedType::RSS2)
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("News, Politics, Opinion, Commentary, and Analysis".into()))
        .description(Text::new("In-depth reporting, commentary on breaking news, political analysis, and opinion from The New\n            Yorker.\n        ".into()))
        .link(Link::new("https://www.newyorker.com/news".into()))
        .rights(Text::new("© Condé Nast 2019".into()))
        .language("en")
        .updated_rfc2822("Tue, 06 Aug 2019 10:46:05 +0000")
        .entry(Entry::default()
            .title(Text::new("How a Historian Uncovered Ronald Reagan’s Racist Remarks to Richard Nixon".into()))
            .link(Link::new("\n                https://www.newyorker.com/news/q-and-a/how-a-historian-uncovered-ronald-reagans-racist-remarks-to-richard-nixon\n            ".into()))
            .id("5d420f3abfe6c20008d5eaad")
            .author(Person::new("Isaac Chotiner".into()))
            .summary(Text::new("Isaac Chotiner talks with the historian Tim Naftali, who published the text and audio of a\n                taped call, from 1971, in which Reagan described the African delegates to the U.N. in luridly racist\n                terms.\n            ".into()))
            .category(Category::new("News / Q. & A."))
            .published_rfc2822("Fri, 02 Aug 2019 15:35:34 +0000")
            .updated(actual.updated)
            .media(MediaObject::default()
                .thumbnail(MediaThumbnail::new(Image::new("https://media.newyorker.com/photos/5d4211a4ba8a9c0009a57cfd/master/pass/Chotiner-ReaganRacismNaftali-3.jpg".into()).width(2560).height(1819)))
            )
        );

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
    let expected = Feed::new(FeedType::RSS2)
        .id(actual.id.as_ref())     // not present in the test data
        .title(Text::new("Earthquakes today".into()))
        .link(Link::new("http://www.earthquakenewstoday.com".into()))
        .description(Text::new("Current and latest world earthquakes breaking news, activity and articles today".into()))
        .updated_rfc2822("Tue, 06 Aug 2019 05:01:15 +0000")
        .language("en-us")
        .generator(Generator::new("https://wordpress.org/?v=5.1.1"))
        .entry(Entry::default()
            .title(Text::new("Minor earthquake, 3.5 mag was detected near Aris in Greece".into()))
            .author(Person::new("admin".into()))
            .link(Link::new("\n                http://www.earthquakenewstoday.com/2019/08/06/minor-earthquake-3-5-mag-was-detected-near-aris-in-greece/\n            ".into()))
            .published_rfc2822("Tue, 06 Aug 2019 05:01:15 +0000")
            .category(Category::new("Earthquake breaking news"))
            .category(Category::new("Minor World Earthquakes Magnitude -3.9"))
            .category(Category::new("Spárti"))
            .id("\n                http://www.earthquakenewstoday.com/2019/08/06/minor-earthquake-3-5-mag-was-detected-near-aris-in-greece/\n            ")
            .summary(Text::new("\n                A minor earthquake magnitude 3.5 (ml/mb) strikes near Kalamáta, Trípoli, Pýrgos, Spárti, Filiatrá, Messíni, Greece on Tuesday.".into()))
            .content(Content::default()
                .body("<p><img class='size-full alignleft' title='Earthquake location 37.102S, 21.9072W' alt='Earthquake location 37.102S, 21.9072W' src='http://www.earthquakenewstoday.com/wp-content/uploads/35_20.jpg' width='146' height='146' />A minor earthquake with magnitude 3.5 (ml/mb) was detected on Tuesday, 8 kilometers (5 miles) from Aris in Greece.Exact location of event, depth 10 km, 21.9072&deg; East, 37.102&deg; North. </p>"))
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
    let expected = Feed::new(FeedType::RSS2)
        .id(actual.id.as_ref()) // not present in the test data
        .title(Text::new("Ars Technica".into()))
        .link(Link::new("https://arstechnica.com".into()))
        .description(Text::new(
            "Serving the Technologist for more than a decade. IT news, reviews, and analysis.".into(),
        ))
        .updated_rfc2822("Tue, 06 Aug 2019 00:03:56 +0000")
        .language("en-us")
        .generator(Generator::new("https://wordpress.org/?v=4.8.3"))
        .logo(
            Image::new("https://cdn.arstechnica.net/wp-content/uploads/2016/10/cropped-ars-logo-512_480-32x32.png".into())
                .title("Ars Technica")
                .link("https://arstechnica.com")
                .width(32)
                .height(32),
        )
        .entry(
            Entry::default()
                .title(Text::new(
                    "Apple isn’t the most cash-rich company in the world anymore, but it doesn’t matter".into(),
                ))
                .link(Link::new("https://arstechnica.com/?p=1546121".into()))
                .published_rfc2822("Mon, 05 Aug 2019 23:11:09 +0000")
                .category(Category::new("Tech"))
                .category(Category::new("alphabet"))
                .category(Category::new("apple"))
                .category(Category::new("google"))
                .id("https://arstechnica.com/?p=1546121")
                .author(Person::new("Samuel Axon".into()))
                .summary(Text::new("Alphabet has $117 billion in cash on hand.".into()))
                .content(Content::default().body("Google co-founder Larry Page is now CEO of Alphabet."))
                .updated(actual.updated),
        );

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
    let expected = Feed::new(FeedType::RSS2)
        .id("b2ef47d837e6c0d9d757e14852e5bde")     // hash of the link
        .title(Text::new("Latest Movie Trailers".into()))
        .link(Link::new("https://trailers.apple.com/".into()))
        .description(Text::new("Recently added Movie Trailers.".into()))
        .language("en-us")
        .updated_rfc3339("2020-02-07T15:30:28Z")
        .generator(Generator::new("Custom"))
        .rights(Text::new("2020 Apple Inc.".into()))
        .entry(Entry::default()
            .title(Text::new("Vitalina Varela - Trailer".into()))
            .link(Link::new("https://trailers.apple.com/trailers/independent/vitalina-varela".into()))
            .summary(Text::new("A film of deeply concentrated beauty, acclaimed filmmaker Pedro Costa’s VITALINA VARELA stars nonprofessional actor Vitalina Varela in an extraordinary performance based on her own life. Vitalina plays a Cape Verdean woman who has travelled to Lisbon to reunite with her husband, after two decades of separation, only to arrive mere days after his funeral. Alone in a strange forbidding land, she perseveres and begins to establish a new life. Winner of the Golden Leopard for Best Film and Best Actress at the Locarno Film Festival, as well as an official selection of the Sundance Film Festival, VITALINA VARELA is a film of shadow and whisper, a profoundly moving and visually ravishing masterpiece.".into()))
            .content(Content::default()
                .body(r#"<span style="font-size: 16px; font-weight: 900; text-decoration: underline;">Vitalina Varela - Trailer</span>"#))
            .published_rfc3339("2020-02-06T08:00:00Z")
            .id("73226f21f249d758bd97a1fac90897d2")        // hash of the link
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
    let expected = Feed::new(FeedType::RSS2)
        .id(actual.id.as_ref()) // not present in the test data
        .title(Text::new("Scripting News".into()))
        .link(Link::new("http://www.scripting.com/".into()))
        .description(Text::new("A weblog about scripting and stuff like that.".into()))
        .language("en-us")
        .rights(Text::new("Copyright 1997-2002 Dave Winer".into()))
        .updated_rfc2822("Mon, 30 Sep 2002 11:00:00 GMT")
        .generator(Generator::new("Radio UserLand v8.0.5"))
        .category(Category::new("1765").scheme("Syndic8"))
        .contributor(Person::new("managingEditor".into()).email("dave@userland.com"))
        .contributor(Person::new("webMaster".into()).email("dave@userland.com"))
        .ttl(40)
        .entry(
            Entry::default()
                .summary(Text::new(
                    r#"Joshua Allen: <a href="http://www.netcrucible.com/blog/2002/09/29.html#a243">Who
                loves namespaces?</a>
            "#
                        .to_owned(),
                ))
                .published_rfc2822("Sun, 29 Sep 2002 19:59:01 GMT")
                .id("http://scriptingnews.userland.com/backissues/2002/09/29#When:12:59:01PM")
                .updated_rfc2822("Mon, 30 Sep 2002 11:00:00 GMT"),
        ) // copy from feed
        .entry(
            Entry::default()
                .summary(Text::new(
                    r#"<a href="http://www.docuverse.com/blog/donpark/2002/09/29.html#a68">Don Park</a>:
                "It is too easy for engineer to anticipate too much and XML Namespace is a frequent host of
                over-anticipation."
            "#
                        .to_owned(),
                ))
                .published_rfc2822("Mon, 30 Sep 2002 01:52:02 GMT")
                .id("http://scriptingnews.userland.com/backissues/2002/09/29#When:6:52:02PM")
                .updated_rfc2822("Mon, 30 Sep 2002 11:00:00 GMT"),
        ); // copy from feed

    // Check
    assert_eq!(actual, expected);
}

// Verifies that an invalid XML document (e.g. truncated) fails to parse
#[test]
fn test_invalid_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_invalid_1.xml");
    let feed = parser::parse(test_data.as_bytes());
    assert!(feed.is_err());
}

// Verifies that we can handle non-UTF8 streams
#[test]
fn test_encoding_1() {
    let test_data = test::fixture_as_raw("rss_2.0_encoding_1.xml");
    let feed = parser::parse(test_data.as_slice()).unwrap();
    assert_eq!(feed.title.unwrap().content, "RSS Feed do Site Inovação Tecnológica");
}

// Verifies we extract the content:encoded element
#[test]
fn test_heated() {
    let test_data = test::fixture_as_raw("rss_2.0_heated.xml");
    let feed = parser::parse(test_data.as_slice()).unwrap();
    let content = &feed.entries[0].content.as_ref().unwrap();
    assert!(content.body.as_ref().unwrap().contains("I have some good news and some bad news"));
}

// Verifies that we can handle mixed MediaRSS and itunes/enclosure
#[test]
fn test_spiegel() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_spiegel.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::new(FeedType::RSS2)
        .id(actual.id.as_ref()) // not present in the test data
        .language("de")
        .title(Text::new("SPIEGEL Update – Die Nachrichten".into()))
        .link(Link::new("https://www.spiegel.de/thema/spiegel-update/".into()))
        .description(Text::new("<p>Die wichtigsten Nachrichten des Tages &ndash; erg&auml;nzt um Meinungen und Empfehlungen aus der SPIEGEL-Redaktion. Wochentags aktualisieren wir morgens, mittags und abends unsere Meldungen. Am Wochenende blicken wir zur&uuml;ck auf die vergangene Woche &ndash; und erkl&auml;ren, was in der n&auml;chsten Woche wichtig wird.</p>".into()))
        .rights(Text::new("2021 DER SPIEGEL GmbH & Co. KG".into()))
        .logo(Image::new("https://www.omnycontent.com/d/programs/5ac1e950-45c7-4eb7-87c0-aa0f018441b8/bb17ca27-51f4-4349-bc1e-abc00102c975/image.jpg?t=1589902935&size=Large".into())
            .title("SPIEGEL Update – Die Nachrichten")
            .link("https://www.spiegel.de/thema/spiegel-update/")
        )
        .entry(
            Entry::default()
                .title(Text::new("07.02. – die Wochenvorschau: Lockdown-Verlängerung, Kriegsverbrecher vor Gericht, Super Bowl, Karneval ".into()))
                .content(Content::default().body(r#"Die wichtigsten Nachrichten aus der SPIEGEL-Redaktion. <br><br><p>See <a href="https://omnystudio.com/listener">omnystudio.com/listener</a> for privacy information.</p>"#))
                .summary(Text::new("Die wichtigsten Nachrichten aus der SPIEGEL-Redaktion. \r\nSee omnystudio.com/listener for privacy information.".into()))
                .link(Link::new("https://omny.fm/shows/spiegel-update-die-nachrichten/07-02-die-wochenvorschau-lockdown-verl-ngerung-kri".into()))
                .published_rfc3339("2021-02-06T23:01:00Z")
                .id("c7e3cca2-665e-4bc4-bcac-acc6011b9fa2")
                // <enclosure>
                .media(MediaObject::default()
                    .content(MediaContent::new()
                        .url("https://traffic.omny.fm/d/clips/5ac1e950-45c7-4eb7-87c0-aa0f018441b8/bb17ca27-51f4-4349-bc1e-abc00102c975/c7e3cca2-665e-4bc4-bcac-acc6011b9fa2/audio.mp3?utm_source=Podcast&amp;in_playlist=4c18e072-24d2-4d60-9a42-abc00102c97e&amp;t=1612652510".into())
                        .size(2519606)
                        .content_type("audio/mpeg")
                    )
                )
                // media: and itunes: tags
                .media(MediaObject::default()
                    .title("07.02. – die Wochenvorschau: Lockdown-Verlängerung, Kriegsverbrecher vor Gericht, Super Bowl, Karneval ".into())
                    .description("Die wichtigsten Nachrichten aus der SPIEGEL-Redaktion. \r\nSee omnystudio.com/listener for privacy information.".into())
                    .credit("DER SPIEGEL")
                    .thumbnail(MediaThumbnail::new(Image::new("https://www.omnycontent.com/d/programs/5ac1e950-45c7-4eb7-87c0-aa0f018441b8/bb17ca27-51f4-4349-bc1e-abc00102c975/image.jpg?t=1589902935&amp;size=Large".into())))
                    .content(MediaContent::new()
                        .url("https://traffic.omny.fm/d/clips/5ac1e950-45c7-4eb7-87c0-aa0f018441b8/bb17ca27-51f4-4349-bc1e-abc00102c975/c7e3cca2-665e-4bc4-bcac-acc6011b9fa2/audio.mp3?utm_source=Podcast&amp;in_playlist=4c18e072-24d2-4d60-9a42-abc00102c97e&amp;t=1612652510".into())
                        .content_type("audio/mpeg")
                    )
                    .content(MediaContent::new()
                        .url("https://www.omnycontent.com/d/programs/5ac1e950-45c7-4eb7-87c0-aa0f018441b8/bb17ca27-51f4-4349-bc1e-abc00102c975/image.jpg?t=1589902935&amp;size=Large".into())
                        .content_type("image/jpeg")
                    )
                    .duration(Duration::from_secs(312))
                )
        );

    // Check
    assert_eq!(actual, expected);
}

// Verifies that we can handle mixed MediaRSS and itunes/enclosure
#[test]
fn test_bbc() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_bbc.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::new(FeedType::RSS2)
        .id(actual.id.as_ref()) // not present in the test data
        .title(Text::new("In Our Time".into()))
        .link(Link::new("http://www.bbc.co.uk/programmes/b006qykl".into()))
        .description(Text::new("Melvyn Bragg and guests discuss the history of ideas".into()))
        .language("en")
        .logo(Image::new("http://ichef.bbci.co.uk/images/ic/3000x3000/p087hyhs.jpg".into())
            .title("In Our Time")
            .link("http://www.bbc.co.uk/programmes/b006qykl")
        )
        .rights(Text::new("(C) BBC 2021".into()))
        .published_rfc2822("Thu, 25 Feb 2021 10:15:00 +0000")
        .entry(
            Entry::default()
                .title(Text::new("Marcus Aurelius".into()))
                .summary(Text::new("Melvyn Bragg and guests discuss...".into()))
                .published_rfc2822("Thu, 25 Feb 2021 10:15:00 +0000")
                .id("urn:bbc:podcast:m000sjxt")
                .link(Link::new("http://www.bbc.co.uk/programmes/m000sjxt".into()))
                // <enclosure>
                .media(MediaObject::default()
                    .content(MediaContent::new()
                        .url("http://open.live.bbc.co.uk/mediaselector/6/redir/version/2.0/mediaset/audio-nondrm-download/proto/http/vpid/p097wt5b.mp3".into())
                        .size(50496000)
                        .content_type("audio/mpeg")
                    )
                )
                // media: and itunes: tags
                .media(MediaObject::default()
                    .description("Melvyn Bragg and guests discuss the man who, according to Machiavelli...".into())
                    .duration(Duration::from_secs(3156))
                    .content(MediaContent::new()
                        .url("http://open.live.bbc.co.uk/mediaselector/6/redir/version/2.0/mediaset/audio-nondrm-download/proto/http/vpid/p097wt5b.mp3".into())
                        .size(50496000)
                        .content_type("audio/mpeg")
                        .duration(Duration::from_secs(3156))
                    )
                    .credit("BBC Radio 4")
                )
        );

    // Check
    assert_eq!(actual, expected);
}

// Verifies that we can handle mixed MediaRSS and itunes/enclosure
#[test]
fn test_ch9() {
    // Parse the feed
    let test_data = test::fixture_as_string("rss_2.0_ch9.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::new(FeedType::RSS2)
        .id(actual.id.as_ref()) // not present in the test data
        .title(Text::new("Azure Friday (HD) - Channel 9".into()))
        .logo(Image::new("https://f.ch9.ms/thumbnail/4761e196-da48-4b41-abfe-e56e0509f04d.png".into())
            .title("Azure Friday (HD) - Channel 9")
            .link("https://s.ch9.ms/Shows/Azure-Friday")
        )
        .description(Text::new("Join Scott Hanselman, Donovan Brown, or Lara Rubbelke as they host the engineers who build Azure, demo it, answer questions, and share insights. ".into()))
        .link(Link::new("https://s.ch9.ms/Shows/Azure-Friday".into()))
        .language("en")
        .published_rfc2822("Sat, 27 Feb 2021 06:55:01 GMT")
        .updated_rfc2822("Sat, 27 Feb 2021 06:55:01 GMT")
        .generator(Generator::new("Rev9"))
        .entry(
            Entry::default()
                .title(Text::new("Troubleshoot AKS cluster issues with AKS Diagnostics and AKS Periscope".into()))
                .summary(Text::new("<p>Yun Jung Choi shows Scott Hanselman...".into()))
                .link(Link::new("https://channel9.msdn.com/Shows/Azure-Friday/Troubleshoot-AKS-cluster-issues-with-AKS-Diagnostics-and-AKS-Periscope".into()))
                .published_rfc2822("Fri, 26 Feb 2021 20:00:00 GMT")
                .updated_rfc2822("Sat, 27 Feb 2021 06:55:01 GMT")
                .id("https://channel9.msdn.com/Shows/Azure-Friday/Troubleshoot-AKS-cluster-issues-with-AKS-Diagnostics-and-AKS-Periscope")
                .author(Person::new("Scott Hanselman, Rob Caron"))
                .category(Category::new("Azure"))
                .category(Category::new("Kubernetes"))
                .category(Category::new("aft"))
                // <media:group>
                .media(MediaObject::default()
                    .content(MediaContent::new().url("https://rev9.blob.core.windows.net/mfupload/04b236b5-e824-4091-85d8-acd90155d4b0_20210124205102.mp4".into()).duration(Duration::from_secs(867)).size(1).content_type("video/mp4"))
                    .content(MediaContent::new().url("https://sec.ch9.ms/ch9/075d/6e61e6c6-3890-4172-a617-fa0c4b38075d/azfr663.mp3".into()).duration(Duration::from_secs(867)).size(13878646).content_type("audio/mp3"))
                    .content(MediaContent::new().url("https://sec.ch9.ms/ch9/075d/6e61e6c6-3890-4172-a617-fa0c4b38075d/azfr663.mp4".into()).duration(Duration::from_secs(867)).size(20450133).content_type("video/mp4"))
                    .content(MediaContent::new().url("https://sec.ch9.ms/ch9/075d/6e61e6c6-3890-4172-a617-fa0c4b38075d/azfr663_high.mp4".into()).duration(Duration::from_secs(867)).size(126659374).content_type("video/mp4"))
                    .content(MediaContent::new().url("https://sec.ch9.ms/ch9/075d/6e61e6c6-3890-4172-a617-fa0c4b38075d/azfr663_mid.mp4".into()).duration(Duration::from_secs(867)).size(49241848).content_type("video/mp4"))
                    .content(MediaContent::new().url("https://www.youtube-nocookie.com/embed/E-XqYb88hUY?enablejsapi=1".into()).duration(Duration::from_secs(867)).size(1))
                )
                // <enclosure>
                .media(MediaObject::default()
                    .content(MediaContent::new().url("https://sec.ch9.ms/ch9/075d/6e61e6c6-3890-4172-a617-fa0c4b38075d/azfr663_high.mp4".into()).size(126659374).content_type("video/mp4"))
                )
                // <media:*>
                .media(MediaObject::default()
                    .description("Yun Jung Choi shows Scott Hanselman how to use AKS Diagnostics...")
                    .duration(Duration::from_secs(867))
                    .thumbnail(MediaThumbnail::new(Image::new("https://sec.ch9.ms/ch9/3724/8609074c-2b7b-41ae-9345-f49973543724/azfr663_100.jpg".into()).height(56).width(100)))
                    .thumbnail(MediaThumbnail::new(Image::new("https://sec.ch9.ms/ch9/3724/8609074c-2b7b-41ae-9345-f49973543724/azfr663_220.jpg".into()).height(123).width(220)))
                    .thumbnail(MediaThumbnail::new(Image::new("https://sec.ch9.ms/ch9/3724/8609074c-2b7b-41ae-9345-f49973543724/azfr663_512.jpg".into()).height(288).width(512)))
                    .thumbnail(MediaThumbnail::new(Image::new("https://sec.ch9.ms/ch9/3724/8609074c-2b7b-41ae-9345-f49973543724/azfr663_960.jpg".into()).height(540).width(960)))
                    .credit("Scott Hanselman, Rob Caron")
                )
        );

    // Check
    assert_eq!(actual, expected);
}
