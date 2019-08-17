use crate::model::{Entry, Person, Link, Feed, Text, Generator, Image, Category, Content};
use crate::parser;
use crate::util::test;

// Verify we can parse a more complete example than the one provided in the standard
#[test]
fn test_example_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("atom_example_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::new()
        .title(Text::new("dive into mark".to_owned()))
        .description(Text::new("A <em>lot</em> of effort\n        went into making this effortless".to_owned())
            .content_type("text/html"))
        .updated_rfc3339("2005-07-31T12:29:29Z")
        .id("tag:example.org,2003:3")
        .link(Link::new("http://example.org/".to_owned())
            .rel("alternate")
            .media_type("text/html")
            .href_lang("en"))
        .link(Link::new("http://example.org/feed.atom".to_owned())
            .rel("self")
            .media_type("application/atom+xml"))
        .rights(Text::new("Copyright (c) 2003, Mark Pilgrim".to_owned()))
        .generator(Generator::new("Example Toolkit".to_owned())
            .uri("http://www.example.com/")
            .version("1.0"))
        .entry(Entry::new()
            .id("tag:example.org,2003:3.2397")
            .title(Text::new("Atom draft-07 snapshot".to_owned()))
            .updated_rfc3339("2005-07-31T12:29:29Z")
            .author(Person::new("Mark Pilgrim".to_owned())
                .uri("http://example.org/")
                .email("f8dy@example.com"))
            .link(Link::new("http://example.org/2005/04/02/atom".to_owned())
                .rel("alternate")
                .media_type("text/html"))
            .link(Link::new("http://example.org/audio/ph34r_my_podcast.mp3".to_owned())
                .rel("enclosure")
                .media_type("audio/mpeg")
                .length(1337))
            .contributor(Person::new("Sam Ruby".to_owned()))
            .contributor(Person::new("Joe Gregorio".to_string()))
            .published_rfc3339("2003-12-13T08:29:29-04:00"));

    // Check
    assert_eq!(actual, expected);
}

// Real-life example from the Register - https://www.theregister.co.uk/science/headlines.atom
#[test]
fn test_example_2() {
    // Parse the feed
    let test_data = test::fixture_as_string("atom_example_2.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    let expected = Feed::new()
        .id("tag:theregister.co.uk,2005:feed/theregister.co.uk/science/")
        .title(Text::new("The Register - Science".to_owned()))
        .link(Link::new("https://www.theregister.co.uk/science/headlines.atom".to_owned())
            .rel("self")
            .media_type("application/atom+xml"))
        .link(Link::new("https://www.theregister.co.uk/science/".to_owned())
            .rel("alternate")
            .media_type("text/html"))
        .rights(Text::new("Copyright © 2019, Situation Publishing".to_owned()))
        .author(Person::new("Team Register".to_owned())
            .email("webmaster@theregister.co.uk")
            .uri("https://www.theregister.co.uk/odds/about/contact/"))
        .icon(Image::new("https://www.theregister.co.uk/Design/graphics/icons/favicon.png".to_owned()))
        .description(Text::new("Biting the hand that feeds IT — sci/tech news and views for the world".to_owned()))
        .logo(Image::new("https://www.theregister.co.uk/Design/graphics/Reg_default/The_Register_r.png".to_owned()))
        .updated_rfc3339("2019-07-31T11:54:28Z")
        .entry(Entry::new()
            .id("tag:theregister.co.uk,2005:story204156")
            .updated_rfc3339("2019-07-31T11:54:28Z")
            .author(Person::new("Richard Speed".to_owned())
                .uri("https://search.theregister.co.uk/?author=Richard%20Speed"))
            .link(Link::new("http://go.theregister.com/feed/www.theregister.co.uk/2019/07/31/orbitbeyond_drops_nasa_moon_contract/".to_owned())
                .rel("alternate")
                .media_type("text/html"))
            .title(Text::new("Will someone plz dump our shizz on the Moon, NASA begs as one of the space biz vendors drops out".to_owned())
                .content_type("text/html"))
            .summary(Text::new("<h4>OrbitBeyond begone: Getting to the Moon is <i>hard</i></h4> <p>NASA made a slew of announcements yesterday aimed at bigging up the agency's efforts to get commercial companies involved with its deep space ambitions – despite one vendor dumping plans for a 2020 lunar landing.…</p>".to_owned())
                .content_type("text/html")))
        .entry(Entry::new()
            .id("tag:theregister.co.uk,2005:story204131")
            .updated_rfc3339("2019-07-30T05:41:09Z")
            .author(Person::new("Kieren McCarthy".to_owned())
                .uri("https://search.theregister.co.uk/?author=Kieren%20McCarthy"))
            .link(Link::new("http://go.theregister.com/feed/www.theregister.co.uk/2019/07/30/french_arming_satellites/".to_owned())
                .rel("alternate")
                .media_type("text/html"))
            .title(Text::new("Satellites with lasers and machine guns coming! China's new plans? Trump's Space Force? Nope, the French".to_owned())
                .content_type("text/html"))
            .summary(Text::new(r#"<h4>After all, what could possibly go wrong, apart from everything?</h4> <p>France is threatening to stick submachine guns on its next generation of satellites as part of an "active space defense" strategy that would enable it to shoot down other space hardware.…</p>"#.to_owned())
                .content_type("text/html")));

    // Check
    assert_eq!(actual, expected);
}

// Real-life example from Akamai (includes categories and elements from a different namespace, along with locally declared namespaces on atom 1.0 elements)
// TODO test we ignore elements with the same name as Atom element but in a different namespace
#[test]
fn test_example_3() {
    // Parse the feed
    let test_data = test::fixture_as_string("atom_example_3.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    let expected = Feed::new()
        .title(Text::new("The Akamai Blog".to_owned()))
        .link(Link::new("https://blogs.akamai.com/".to_owned())
            .rel("alternate")
            .media_type("text/html"))
        .id("tag:blogs.akamai.com,2019-07-30://2")
        .updated_rfc3339("2019-07-30T15:02:05Z")
        .generator(Generator::new("Movable Type Pro 5.2.13".to_owned())
            .uri("http://www.sixapart.com/movabletype/"))
        .link(Link::new("http://feeds.feedburner.com/TheAkamaiBlog".to_owned())
            .rel("self")
            .media_type("application/atom+xml"))
        .link(Link::new("http://pubsubhubbub.appspot.com/".to_owned())
            .rel("hub"))
        .entry(Entry::new()
            .title(Text::new("Time to Transfer Risk: Why Security Complexity & VPNs Are No Longer Sustainable".to_owned()))
            .link(Link::new("http://feedproxy.google.com/~r/TheAkamaiBlog/~3/NnQEuqRSyug/time-to-transfer-risk-why-security-complexity-vpns-are-no-longer-sustainable.html".to_owned())
                .rel("alternate")
                .media_type("text/html"))
            .id("tag:blogs.akamai.com,2019://2.3337")
            .published_rfc3339("2019-07-30T16:00:00Z")
            .updated_rfc3339("2019-07-30T15:02:05Z")
            .summary(Text::new("Now, there are many reasons to isolate your infrastructure from the Internet. Minimizing the number of exposed things not only reduces risk, it also reduces operational complexity. VPNs are counter to this. VPNs make it so you aren't exposing all of your applications publicly in a DMZ, which is good. But for the most part, they still provide access to the corporate network to get access to corporate apps. Definitely bad. At this point, I think we all agree that moats and castles belong in the past.".to_owned()))
            .author(Person::new("Lorenz Jakober".to_owned()))
            .category(Category::new("Zero Trust".to_owned())
                .scheme("http://www.sixapart.com/ns/types#category"))
            .category(Category::new("ssl".to_owned())
                .label("SSL")
                .scheme("http://www.sixapart.com/ns/types#tag"))
            .category(Category::new("zerotrust".to_owned())
                .label("Zero Trust")
                .scheme("http://www.sixapart.com/ns/types#tag"))
            .content(Content::new()
                .body(r#"<p>We all heed the gospel of patching, but as recent incidents made clear, even cutting-edge disruptors struggle to patch everything, everywhere, and all the time.</p>
        <img src="http://feeds.feedburner.com/~r/TheAkamaiBlog/~4/NnQEuqRSyug" height="1" width="1" alt=""/>"#)
                .content_type("text/html")));

    // Check
    assert_eq!(actual, expected);
}

// Real-life example from ebmpapst (CDATA text elements, unusual category representation)
#[test]
fn test_example_4() {
    // Parse the feed
    let test_data = test::fixture_as_string("atom_example_4.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    let expected = Feed::new()
        .author(Person::new("ebm-papst".to_owned()))
        .link(Link::new("http://www.ebmpapst.com/en/ebmpapst_productnews_atom_feed.xml".to_owned())
            .rel("self")
            .media_type("application/atom+xml"))
        .title(Text::new("ebm-papst product news".to_owned()))
        .id("tag:ebmpapst.com,2011-06-30:1309426729931")
        .updated_rfc3339("2019-07-29T09:41:09Z")
        .entry(Entry::new()
            .title(Text::new("Connection with future".to_owned()))
            .link(Link::new("https://idt.ebmpapst.com/de/en/idt/campaign/simatic-micro-drive.html".to_owned())
                .rel("alternate"))
            .id("tag:ebmpapst.com,2019-07-17:0310161724098")
            .updated_rfc3339("2019-07-17T03:10:16Z")
            .summary(Text::new(r#"<a href="https://idt.ebmpapst.com/de/en/idt/campaign/simatic-micro-drive.html"><img src="http://www.ebmpapst.com//media/content/homepage/currenttopic/ads_cd2013/FF_ep_keyvisual_100px.jpg" border="0" align="right"></a> Working in perfect harmony: the ebm-papst drive solutions for SIMATIC MICRO-DRIVE drive regulators from Siemens."#.to_owned())
                .content_type("text/html")));

    // Check
    assert_eq!(actual, expected);
}

// Real-life example from USGS (CDATA, different namespaces)
#[test]
fn test_example_5() {
    // Parse the feed
    let test_data = test::fixture_as_string("atom_example_5.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    let expected = Feed::new()
        .title(Text::new("USGS Magnitude 2.5+ Earthquakes, Past Hour".to_owned()))
        .updated_rfc3339("2019-07-31T13:17:27Z")
        .author(Person::new("U.S. Geological Survey".to_owned())
            .uri("https://earthquake.usgs.gov/"))
        .id("https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/2.5_hour.atom")
        .link(Link::new("https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/2.5_hour.atom".to_owned())
            .rel("self"))
        .icon(Image::new("https://earthquake.usgs.gov/favicon.ico".to_owned()))
        .entry(Entry::new()
            .id("urn:earthquake-usgs-gov:nc:73239366")
            .title(Text::new("M 3.6 - 15km W of Petrolia, CA".to_owned()))
            .updated_rfc3339("2019-07-31T13:07:31.364Z")
            .link(Link::new("https://earthquake.usgs.gov/earthquakes/eventpage/nc73239366".to_owned())
                .rel("alternate")
                .media_type("text/html"))
            .summary(Text::new(r#"<p class="quicksummary"><a href="https://earthquake.usgs.gov/earthquakes/eventpage/nc73239366#shakemap" title="ShakeMap maximum estimated intensity" class="mmi-II">ShakeMap - <strong class="roman">II</strong></a> <a href="https://earthquake.usgs.gov/earthquakes/eventpage/nc73239366#dyfi" class="mmi-IV" title="Did You Feel It? maximum reported intensity (4 reports)">DYFI? - <strong class="roman">IV</strong></a></p><dl><dt>Time</dt><dd>2019-07-31 12:26:15 UTC</dd><dd>2019-07-31 04:26:15 -08:00 at epicenter</dd><dt>Location</dt><dd>40.347&deg;N 124.460&deg;W</dd><dt>Depth</dt><dd>29.35 km (18.24 mi)</dd></dl>"#.to_owned())
                .content_type("text/html"))
            .category(Category::new("Past Hour".to_owned())
                .label("Age"))
            .category(Category::new("Magnitude 3".to_owned())
                .label("Magnitude"))
            .category(Category::new("nc".to_owned())
                .label("Contributor"))
            .category(Category::new("nc".to_owned())
                .label("Author")));

    // Check
    assert_eq!(actual, expected);
}

// Verify we can parse the example contained in the Atom specification
#[test]
fn test_spec_1() {
    // Parse the feed
    let test_data = test::fixture_as_string("atom_spec_1.xml");
    let actual = parser::parse(test_data.as_bytes()).unwrap();

    // Expected feed
    let expected = Feed::new()
        .id("urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6")
        .title(Text::new("Example Feed".to_owned()))
        .link(Link::new("http://example.org/".to_owned())
            .rel("alternate"))
        .updated_rfc3339("2003-12-13T18:30:02Z")
        .author(Person::new("John Doe".to_owned()))
        .entry(Entry::new()
            .id("urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a")
            .title(Text::new("Atom-Powered Robots Run Amok".to_owned()))
            .updated_rfc3339("2003-12-13T18:30:02Z")
            .summary(Text::new("Some text.".to_owned()))
            .link(Link::new("http://example.org/2003/12/13/atom03".to_owned())
                .rel("alternate")));

    // Check
    assert_eq!(actual, expected);
}
