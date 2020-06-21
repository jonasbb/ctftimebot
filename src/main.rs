use chrono::Utc;
use ctftimebot::{mattermost_hook_api::Message, CtfEvent, CONFIG};
use log::{error, info};
use std::io::Read;

fn main() {
    env_logger::init();

    let today = Utc::now().timestamp();
    let end = today + 100 * (3600 * 24);
    let url = format!(
        "https://ctftime.org/api/v1/events/?limit=30&start={}&finish={}",
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

    let mut message = Message {
        username: Some("Upcoming CTFs".to_string()),
        text: Some("[Upcoming CTFs](https://ctftime.org/event/list/upcoming)".to_string()),
        attachments: events,
        ..Default::default()
    };
    if let Some(ref c) = CONFIG.mattermost_channel {
        message.channel = Some(c.to_string());
    }
    if let Some(ref url) = CONFIG.bot_icon {
        message.icon_url = Some(url.clone())
    }

    let res = reqwest::blocking::Client::new()
        .post(&CONFIG.webhook_url)
        .json(&message)
        .send();
    if let Err(x) = res {
        error!("ERR: {:?}", x)
    }
}
