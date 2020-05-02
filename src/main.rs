use chrono::Utc;
use ctftimebot::{CtfEvent, CONFIG};
use log::{error, info};
use slack_hook::{PayloadBuilder, Slack};
use std::io::Read;

fn main() {
    env_logger::init();

    let today = Utc::now().timestamp();
    let end = today + 100 * (3600 * 24);
    let url = format!(
        "https://ctftime.org/api/v1/events/?limit=50&start={}&finish={}",
        today, end
    );
    let mut resp = reqwest::blocking::get(&url).unwrap();
    let mut data = String::new();
    resp.read_to_string(&mut data).unwrap();
    let events: Vec<CtfEvent> = serde_json::from_str(&data).unwrap();
    let events: Vec<_> = events
        .into_iter()
        .filter(CtfEvent::should_print_event)
        .map(|x| x.to_slack())
        .collect();
    if events.is_empty() {
        info!("No CTFs in the specified time frame. Exiting...");
        // early exit in case there is no upcoming CTF
        return;
    }
    info!("Found {} events in the specified time frame.", events.len());

    let slack = Slack::new(CONFIG.webhook_url.as_ref()).unwrap();
    let mut p = PayloadBuilder::new()
        .username("Upcoming CTFs")
        .text("[Upcoming CTFs](https://ctftime.org/event/list/upcoming)")
        .attachments(events);
    if let Some(ref c) = CONFIG.mattermost_channel {
        p = p.channel(c.to_string());
    }

    if let Some(ref url) = CONFIG.bot_icon {
        p = p.icon_url(url.as_ref())
    }
    let p = p.build().unwrap();

    let res = slack.send(&p);
    if let Err(x) = res {
        error!("ERR: {:?}", x)
    }
}
