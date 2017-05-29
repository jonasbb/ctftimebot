extern crate chrono;
#[macro_use]
extern crate derive_builder;
extern crate xml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate slack_hook;
extern crate dotenv;
// extern crate envy;
#[macro_use]
extern crate lazy_static;

use std::env;
use std::str::FromStr;
use chrono::prelude::*;
use chrono::Duration;
use std::io::prelude::*;
use xml::reader::{EventReader, XmlEvent};
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
    // static ref CONFIG: Config = {
    //     dotenv::dotenv().expect("Failed to read .env file");
    //     envy::from_env::<Config>().expect("Couldn't read config")
    // };
    pub static ref CONFIG: Config = {
        dotenv::dotenv().expect("Failed to read .env file");
        Config {
            webhook_url: env::var("WEBHOOK_URL").expect("You need to define the WEBHOOK_URL environment variable."),
            days_into_future: env::var("DAYS_INTO_FUTURE").unwrap_or_else(|_| "21".into()).parse::<i64>().expect("Cannot parse the DAYS_INTO_FUTURE environment variable."),
            color_jeopardy: env::var("COLOR_JEOPARDY").unwrap_or_else(|_| "#0099e1".into()),
            color_attack_defense: env::var("COLOR_ATTACK_DEFENSE").unwrap_or_else(|_| "#da5422".into()),
            bot_icon: env::var("BOT_ICON").ok(),
            always_show_ctfs: env::var("ALWAYS_SHOW_CTFS").and_then(|x| Ok(x.split(',').map(|x| x.parse::<usize>().unwrap_or(0)).collect::<Vec<_>>())).unwrap_or_else(|_| Vec::new())
        }
    };
}


#[derive(Builder,Debug)]
pub struct CtfEvent {
    /// Event title, this is specific to one event, e.g. "FAUST CTF 2017"
    title: String,
    /// Link to CTF time page of event
    link: String,
    /// GUID, contains same value as link
    guid: String,
    /// Start time
    start_date: DateTime<FixedOffset>,
    /// End time
    finish_date: DateTime<FixedOffset>,
    /// URL of logo
    #[builder(default="None")]
    logo_url: Option<String>,
    /// Link to the event page
    #[builder(default="None")]
    url: Option<String>,
    /// format style of CTF, most common Jeopardy or AttackDefense
    format: CtfFormat,
    /// Determines if the public is allowed to vote for the final weight
    public_votable: bool,
    /// The weight of the event
    weight: f32,
    /// A link to the live feed of the event
    #[builder(default="None")]
    live_feed: Option<String>,
    /// Access restrictions for this event
    restrictions: CtfRestrictions,
    /// Location of an onsite CTF. Should be set if `onsite` is true.
    #[builder(default="None")]
    location: Option<String>,
    /// Specifies that the event is at a specific location, `location` should be set in this case
    onsite: bool,
    /// List of all the organizer teams
    organizers: Vec<CtfTeam>,
    /// ID of the general event
    ctf_id: usize,
    /// Name of the general event, e.g. "FAUST CTF"
    ctf_name: String,
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
        let url = self.url.clone().unwrap_or_else(|| self.link.clone());

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

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum CtfRestrictions {
    Open,
    Prequalified,
    Academic,
    Invited,
    HighSchool,
}

impl FromStr for CtfRestrictions {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Open" => Ok(CtfRestrictions::Open),
            "Prequalified" => Ok(CtfRestrictions::Prequalified),
            "High-school" => Ok(CtfRestrictions::HighSchool),
            "Academic" => Ok(CtfRestrictions::Academic),
            "Invited" => Ok(CtfRestrictions::Invited),
            _ => Err(format!("Unknown Restrictions: {}", s)),
        }
    }
}

/// What type of CTF, e.g. `AttackDefense`
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum CtfFormat {
    Jeopardy,
    AttackDefense,
    HackQuest,
    Unknown,
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

impl From<isize> for CtfFormat {
    fn from(value: isize) -> Self {
        match value {
            1 => CtfFormat::Jeopardy,
            2 => CtfFormat::AttackDefense,
            3 => CtfFormat::HackQuest,
            _ => CtfFormat::Unknown,
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

/// Parse the ctftime RSS feed for the special `CtfEvent` related
/// fields and construct and array of `CtfEvent` instances.
pub fn parse_ctftime_feed<R: Read>(reader: R) -> Vec<CtfEvent> {
    let mut parser = EventReader::new(reader);

    let mut result = Vec::new();
    let mut in_item = false;
    let mut builder = CtfEventBuilder::default();
    let mut data: Option<String> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                if name.local_name == "item" {
                    in_item = true;
                };
            }
            Ok(XmlEvent::EndElement { name }) => {
                match &*name.local_name {
                    "item" => {
                        in_item = false;
                        // finalize CtfEvent
                        result.push(builder.build().unwrap());
                        builder = CtfEventBuilder::default();
                    }

                    // Elements of a CtfEvent
                    "title" if data.is_some() => {
                        builder.title(data.unwrap());
                    }
                    "link" if data.is_some() => {
                        builder.link(data.unwrap());
                    }
                    "guid" if data.is_some() => {
                        builder.guid(data.unwrap());
                    }
                    "start_date" if data.is_some() => {
                        builder.start_date(DateTime::parse_from_str(&(data.unwrap() + "+0000"),
                                                                    "%Y%m%dT%H%M%S%z")
                                                   .unwrap());
                    }
                    "finish_date" if data.is_some() => {
                        builder.finish_date(DateTime::parse_from_str(&(data.unwrap() + "+0000"),
                                                                     "%Y%m%dT%H%M%S%z")
                                                    .unwrap());
                    }
                    "logo_url" if data.is_some() => {
                        builder.logo_url(Some(BASE_URL.to_string() + &data.unwrap()));
                    }
                    "url" if data.is_some() => {
                        builder.url(Some(data.unwrap()));
                    }
                    "format" if data.is_some() => {
                        builder.format(data.unwrap().parse::<isize>().unwrap().into());
                    }
                    "public_votable" if data.is_some() => {
                        builder.public_votable(parse_bool(data.unwrap()));
                    }
                    "weight" if data.is_some() => {
                        builder.weight(data.unwrap().parse::<f32>().unwrap());
                    }
                    "live_feed" if data.is_some() => {
                        builder.live_feed(Some(data.unwrap()));
                    }
                    "restrictions" if data.is_some() => {
                        builder.restrictions(CtfRestrictions::from_str(&data.unwrap()).unwrap());
                    }
                    "location" if data.is_some() => {
                        builder.location(Some(data.unwrap()));
                    }
                    "onsite" if data.is_some() => {
                        builder.onsite(parse_bool(data.unwrap()));
                    }
                    "organizers" if data.is_some() => {
                        builder.organizers(serde_json::from_str(&data.unwrap()).unwrap());
                    }
                    "ctf_id" if data.is_some() => {
                        builder.ctf_id(data.unwrap().parse::<usize>().unwrap());
                    }
                    "ctf_name" if data.is_some() => {
                        builder.ctf_name(data.unwrap());
                    }
                    _ => {}
                };
                // remove old data
                data = None;
            }
            Ok(XmlEvent::Characters(d)) => {
                if in_item {
                    data = Some(d);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            Ok(XmlEvent::EndDocument) => {
                break;
            }
            _ => {}
        }
    }
    result
}

/// Parse a string into a bool value.
fn parse_bool<T: AsRef<str>>(value: T) -> bool {
    match value.as_ref() {
        "false" | "False" => false,
        "true" | "True" | _ => true,
    }
}