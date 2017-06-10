use chrono::{NaiveDateTime, UTC};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub id:          String,
    pub title:       Option<String>,
    pub content:     Option<String>,
    pub summary:     Option<String>,
    pub author:      Option<String>,
    pub published:   NaiveDateTime,
    pub updated:     Option<NaiveDateTime>,
    pub alternate:   Vec<Link>,
    pub keywords:    Vec<String>,
    pub enclosure:   Vec<Link>,
    pub fingerprint: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Link {
    pub href:      String,
    #[serde(rename = "type")]
    pub mime_type: String,
    pub length:    Option<i64>,
}

impl Entry {
    pub fn new() -> Entry {
        Entry {
            id:          "".to_string(),
            title:       None,
            content:     None,
            summary:     None,
            author:      None,
            published:   UTC::now().naive_utc(),
            updated:     None,
            alternate:   vec![],
            keywords:    vec![],
            enclosure:   vec![],
            fingerprint: "".to_string(),
        }
    }
}

impl Link {
    pub fn new(mime_type: &str, href: String) -> Link {
        Link {
            mime_type: mime_type.to_string(),
            href:      href,
            length:    None,
        }
    }
    pub fn enc(mime_type: String, length: i64, href: String) -> Link {
        Link {
            mime_type: mime_type,
            href:      href,
            length:    Some(length),
        }
    }
}
