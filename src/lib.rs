extern crate chrono;
extern crate dotenv;
extern crate envy;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate slack_hook;

use chrono::prelude::*;
use chrono::Duration;
use slack_hook::{Attachment, AttachmentBuilder};

const BASE_URL: &str = "https://ctftime.org";

#[derive(Deserialize,Debug)]
pub struct Config {
    pub webhook_url: String,
    pub days_into_future: i64,
    pub color_jeopardy: String,
    pub color_attack_defense: String,
    pub bot_icon: Option<String>,
    pub always_show_ctfs: Vec<usize>,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        dotenv::dotenv().expect("Failed to read .env file");
        envy::from_env::<Config>().expect("Couldn't read config")
    };
}


#[derive(Debug,Deserialize)]
pub struct CtfEvent {
    /// Event title, this is specific to one event, e.g. "FAUST CTF 2017"
    title: String,
    /// Link to CTF time page of event
    ctftime_url: String,
    /// Event id
    id: usize,
    /// Start time
    #[serde(rename = "start")]
    start_date: DateTime<FixedOffset>,
    /// End time
    #[serde(rename = "finish")]
    finish_date: DateTime<FixedOffset>,
    /// URL of logo
    #[serde(rename = "logo", deserialize_with = "deserialize_string_empty_as_none")]
    logo_url: Option<String>,
    /// Link to the event page
    #[serde(deserialize_with = "deserialize_string_empty_as_none")]
    url: Option<String>,
    /// format style of CTF, most common Jeopardy or AttackDefense
    format: CtfFormat,
    /// Determines if the public is allowed to vote for the final weight
    public_votable: bool,
    /// The weight of the event
    weight: f32,
    /// A link to the live feed of the event
    #[serde(deserialize_with = "deserialize_string_empty_as_none")]
    live_feed: Option<String>,
    /// Access restrictions for this event
    restrictions: CtfRestrictions,
    /// Location of an onsite CTF. Should be set if `onsite` is true.
    #[serde(deserialize_with = "deserialize_string_empty_as_none")]
    location: Option<String>,
    /// Specifies that the event is at a specific location, `location` should be set in this case
    onsite: bool,
    /// List of all the organizer teams
    organizers: Vec<CtfTeam>,
    /// ID of the general event
    ctf_id: usize,
    /// Number of teams who want to participate
    participants: usize,
}

fn format_duration(d: &Duration) -> String {
    let mut d = *d;
    let mut tmp = Vec::with_capacity(4);
    if d.num_hours() > 48 {
        tmp.push(format!("{} days", d.num_days()));
        d = d + Duration::days(-d.num_days());
    }
    if d.num_hours() > 0 {
        tmp.push(format!("{} hours", d.num_hours()));
        d = d + Duration::hours(-d.num_hours());
    }
    if d.num_minutes() > 0 {
        tmp.push(format!("{} minutes", d.num_minutes()));
        d = d + Duration::minutes(-d.num_minutes());
    }
    if d.num_seconds() > 0 {
        tmp.push(format!("{} seconds", d.num_seconds()));
    }
    tmp.join(" ")
}

impl CtfEvent {
    pub fn to_slack(&self) -> Attachment {
        let duration = format_duration(&self.finish_date.signed_duration_since(self.start_date));
        let title = format!("{} — {}", self.title, self.format.to_string());
        let organizers = ((&self.organizers)
                              .into_iter()
                              .map(|x| x.to_string())
                              .collect::<Vec<_>>())
                .join(", ");
        let url = self.url.clone().unwrap_or_else(|| self.ctftime_url.clone());

        let mut text = format!(r#"**Date:** {} for {}
**Organizers:** {}
[{url:}]({url:})

"#,
                               self.start_date.with_timezone(&Local).format("%A, %F %R"),
                               duration,
                               organizers,
                               url = url);

        if self.onsite {
            if let Some(ref location) = self.location {
                text += &format!("**Location:** {}\n", location);
            }
        }
        if self.restrictions == CtfRestrictions::Prequalified {
            text += "Prequalified teams only\n"
        }

        let fallback = format!("{}\nDate: {} for {}\n{}",
                               title,
                               self.start_date.with_timezone(&Local).naive_local(),
                               duration,
                               url);

        let mut builder = AttachmentBuilder::new(fallback)
            .title(title)
            .text(text.trim().to_string())
            .color(if self.format == CtfFormat::AttackDefense {
                        CONFIG.color_attack_defense.clone()
                   } else {
                        CONFIG.color_jeopardy.clone()
                   });

        if let Some(ref url) = self.logo_url {
            builder = builder.thumb_url(url.as_ref());
        }

        builder.build().unwrap()
    }

    /// Determines if this event should be printed
    ///
    /// Reasons to exclude it are it is too far in the future or it is not availble online.
    pub fn should_print_event(&self) -> bool {
        if CONFIG.always_show_ctfs.contains(&self.ctf_id) {
            return true;
        }

        if self.restrictions != CtfRestrictions::Open && self.restrictions != CtfRestrictions::Academic {
            return false;
        }
        let days_into_future = (self.start_date.signed_duration_since(UTC::now().with_timezone(&UTC.fix()))).num_days();
        !self.onsite && days_into_future <= CONFIG.days_into_future
    }
}

#[derive(Clone,Copy,Debug,Deserialize,Eq,PartialEq)]
pub enum CtfRestrictions {
    Open,
    Prequalified,
    Academic,
    Invited,
    #[serde(rename = "High-school")]
    HighSchool,
}

/// What type of CTF, e.g. `AttackDefense`
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum CtfFormat {
    Jeopardy,
    AttackDefense,
    HackQuest,
    Unknown,
}

impl<'de> serde::de::Deserialize<'de> for CtfFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
    use serde::de::*;
    struct CtfFormatVisitor;

    impl<'de> serde::de::Visitor<'de> for CtfFormatVisitor {
        type Value = CtfFormat;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("one of `Jeopardy`, `Attack-Defense`, `Hack quest`, ``")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: serde::de::Error
        {
            use CtfFormat::*;
            match value {
                "Jeopardy" => Ok(Jeopardy),
                "Attack-Defense" => Ok(AttackDefense),
                "Hack quest" => Ok(HackQuest),
                "" => Ok(Unknown),
                _ => Err(Error::invalid_value(Unexpected::Str(value), &self))
            }
        }
    }

    deserializer.deserialize_str(CtfFormatVisitor)
    }
}

impl CtfFormat {
    fn to_string(&self) -> &str {
        match *self {
            CtfFormat::Jeopardy => "Jeopardy",
            CtfFormat::AttackDefense => "Attack-Defense",
            CtfFormat::HackQuest => "Hack-Quest",
            CtfFormat::Unknown => "Unknown",
        }
    }
}

/// Represent a team within ctftime
#[derive(Clone,Debug,Deserialize,Eq,PartialEq)]
pub struct CtfTeam {
    id: usize,
    name: String,
}

impl CtfTeam {
    pub fn to_string(&self) -> String {
        format!("[{}]({}/team/{})", self.name, BASE_URL, self.id)
    }
}

/// Deserialize a `Option<String>` type while transforming the empty string to `None`
fn deserialize_string_empty_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where D: serde::de::Deserializer<'de>
{
    struct OptionStringEmptyNone;
    impl<'de> serde::de::Visitor<'de> for OptionStringEmptyNone {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("any string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: serde::de::Error
        {
            match value {
                "" => Ok(None),
                _ => Ok(Some(value.to_string()))
            }
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where E: serde::de::Error
        {
            match &*value {
                "" => Ok(None),
                _ => Ok(Some(value))
            }
        }

        // handles the `null` case
        fn visit_unit<E>(self) -> Result<Self::Value, E>
            where E: serde::de::Error
        {
            Ok(None)
        }
    }

    deserializer.deserialize_string(OptionStringEmptyNone)
}

#[test]
fn test_deserialize_ctf_event() {
    use std::fs::File;
    let json = File::open("./tests/ctfs.json").unwrap();

    let res: Vec<CtfEvent> = serde_json::from_reader(json).unwrap();
    assert_eq!(res.len(), 442);

    let event = res.iter().last().unwrap();
    assert_eq!(event.onsite, true);
    assert_eq!(event.weight, 0.0);
    assert_eq!(event.title, "GreHack CTF 2017");
    assert_eq!(event.url, Some("https://www.grehack.fr/".to_string()));
    assert_eq!(event.restrictions, CtfRestrictions::Open);
    assert_eq!(event.format, CtfFormat::Jeopardy);
    assert_eq!(event.participants, 20);
    assert_eq!(event.ctftime_url, "https://ctftime.org/event/426/");
    assert_eq!(event.location, Some("Grenoble, France".to_string()));
    assert_eq!(event.live_feed, None);
    assert_eq!(event.public_votable, false);
    assert_eq!(event.logo_url, Some("https://ctftime.org/media/events/2016_ctftime.png".to_string()));
    assert_eq!(event.id, 426);
    assert_eq!(event.ctf_id, 42);
}