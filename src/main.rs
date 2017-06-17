extern crate chrono;
extern crate ctftimebot;
extern crate reqwest;
extern crate serde_json;
extern crate slack_hook;

use chrono::UTC;
use ctftimebot::{CONFIG, CtfEvent};
use slack_hook::{Slack, PayloadBuilder};
use std::io::Read;

fn main() {
    let today = UTC::now().timestamp();
    let end = today + 100 * (3600 * 24);
    let url = format!("https://ctftime.org/api/v1/events/?limit=100&start={}&finish={}", today, end);
    let mut resp = reqwest::get(&url).unwrap();
    let mut data = String::new();
    resp.read_to_string(&mut data).unwrap();
    let event: Vec<CtfEvent> = serde_json::from_str(&data).unwrap();

    let slack = Slack::new(CONFIG.webhook_url.as_ref()).unwrap();
    let mut p = PayloadBuilder::new()
        .username("Upcoming CTFs")
        .text("[Upcoming CTFs](https://ctftime.org/event/oldlist/upcoming)")
        .attachments(event
                         .into_iter()
                         .filter(|x| x.should_print_event())
                         .map(|x| x.to_slack())
                         .collect::<Vec<_>>());

    if let Some(ref url) = CONFIG.bot_icon {
        p = p.icon_url(url.as_ref())
    }
    let p = p.build().unwrap();

    let res = slack.send(&p);
    if let Err(x) = res {
        println!("ERR: {:?}", x)
    }
}

