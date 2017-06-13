use chrono::{NaiveDateTime};
use entry::Entry;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Feed {
    pub id:           String,
    pub title:        Option<String>,
    pub description:  Option<String>,
    pub language:     Option<String>,

    pub website:      Option<String>,
    pub topics:       Option<Vec<String>>,
    pub last_updated: Option<NaiveDateTime>,

    pub visual_url:   Option<String>,
    pub icon_url:     Option<String>,
    pub cover_url:    Option<String>,

    pub entries:     Vec<Entry>,
}

impl Feed {
    pub fn new() -> Feed {
        Feed {
            id:           String::from(""),
            title:        None,
            description:  None,
            language:     None,
            website:      None,
            topics:       None,
            last_updated: None,
            visual_url:   None,
            icon_url:     None,
            cover_url:    None,

            entries:      vec![],
        }
    }
}
