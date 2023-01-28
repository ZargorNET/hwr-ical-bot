use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;
use icalendar::{Calendar, CalendarComponent, Component, Event};
use serenity::http::Http;

use serenity::model::id::ChannelId;
use crate::embed_builder::build_embed;

const FILE_LOCATION: &str = "prevcal.ics";

pub struct RunConfig {
    pub channel_id: ChannelId,
    pub discord_http: Arc<Http>,
    pub ics_url: String,
}

pub fn run(config: RunConfig) {
    let mut interval = tokio::time::interval(Duration::from_secs(10 * 60));

    tokio::spawn(async move {
        loop {
            let result: anyhow::Result<()> = try {
                interval.tick().await;

                let new_calendar = fetch_ics(&config.ics_url).await?;
                let prev_calendar = get_prev_calendar().await?.unwrap_or(Calendar::new());

                let diff = compare_calendars(&new_calendar, &prev_calendar);

                if !diff.is_empty() {
                    let embed = build_embed(diff);

                    config.channel_id.send_message(&config.discord_http, |builder| {
                        builder.set_embed(embed)
                    }).await?;
                }


                save_calendar(&new_calendar).await?;
            };

            if let Err(e) = result {
                eprintln!("Error while executing loop: {}", e);
            }
        }
    });
}

async fn fetch_ics(url: &str) -> anyhow::Result<Calendar> {
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Err(anyhow!("received status code {}", &response.status()));
    }

    Ok(Calendar::from_str(&response.text().await?).map_err(|e| anyhow!(e))?)
}

pub enum Diff<'a> {
    Created(&'a Event),
    Changed {
        new: &'a Event,
        old: &'a Event,
    },
    Removed(&'a Event),
}

fn compare_calendars<'a>(new: &'a Calendar, old: &'a Calendar) -> Vec<Diff<'a>> {
    let mut diff = Vec::new();

    // Check for new or changed events
    for component in new.components.iter() {
        if let CalendarComponent::Event(e) = component {
            if e.get_uid().is_none() {
                println!("Warn: Event {} has no UID", e.get_summary().unwrap_or_default());
                continue;
            }

            let uid = e.get_uid().unwrap();

            let other = get_event_by_uid(old, uid);

            match other {
                None => diff.push(Diff::Created(e)),
                Some(other) => {
                    if e != other {
                        diff.push(Diff::Changed {
                            new: e,
                            old: other,
                        });
                    }
                }
            }
        }
    }

    // Check for removed events

    old.components.iter()
        .filter_map(|c| c.as_event())
        .filter(|e| e.get_uid().is_some())
        .filter(|e| get_event_by_uid(new, e.get_uid().unwrap()).is_none())
        .for_each(|e| diff.push(Diff::Removed(e)));

    diff
}

fn get_event_by_uid<'a>(cal: &'a Calendar, uid: &str) -> Option<&'a Event> {
    cal.components.iter()
        .filter_map(|c| c.as_event())
        .filter(|e| e.get_uid().is_some())
        .find(|e| e.get_uid().unwrap() == uid)
}

async fn get_prev_calendar() -> anyhow::Result<Option<Calendar>> {
    if !std::path::Path::new(FILE_LOCATION).exists() {
        return Ok(None);
    }

    let file = tokio::fs::read_to_string(FILE_LOCATION).await?;

    Ok(Some(Calendar::from_str(&file).map_err(|e| anyhow!(e))?))
}

async fn save_calendar(calendar: &Calendar) -> anyhow::Result<()> {
    tokio::fs::write(FILE_LOCATION, calendar.to_string()).await?;

    Ok(())
}
