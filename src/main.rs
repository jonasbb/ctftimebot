extern crate slack_hook;
extern crate reqwest;
extern crate ctftimebot;

use ctftimebot::{CONFIG, parse_ctftime_feed};
use slack_hook::{Slack, PayloadBuilder};

fn main() {
    let resp = reqwest::get("https://ctftime.org/event/list/upcoming/rss/").unwrap();
    let event = parse_ctftime_feed(resp);

    let slack = Slack::new(CONFIG.webhook_url.as_ref()).unwrap();
    let mut p = PayloadBuilder::new()
        .username("Upcoming CTFs")
        .text("@all: [Upcoming CTFs](https://ctftime.org/event/oldlist/upcoming)")
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

