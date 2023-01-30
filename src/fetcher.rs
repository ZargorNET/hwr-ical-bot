use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use chrono::{DateTime, Local, TimeZone};
use chrono_tz::Tz;
use icalendar::{Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime, Event};
use serenity::http::Http;
use serenity::model::id::ChannelId;

use crate::Config;
use crate::embed_builder::build_embed;

pub fn run(http: Arc<Http>, config: Config) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10 * 60));

    tokio::spawn(async move {
        loop {
            let result: anyhow::Result<()> = try {
                interval.tick().await;

                for endpoint in &config.endpoints {
                    let path = format!("data/prev_cal_{}.ics", endpoint.display_name.replace(" ", "_"));

                    let new_calendar = fetch_ics(&endpoint.ics_url).await?;
                    let prev_calendar = get_prev_calendar(&path).await?.unwrap_or(Calendar::new());

                    let diff = compare_calendars(&new_calendar, &prev_calendar);

                    if !diff.is_empty() {
                        let embeds = build_embed(diff, &endpoint.display_name);

                        for embed in embeds {
                            ChannelId(endpoint.channel_id).send_message(&http, |builder| {
                                builder.set_embed(embed)
                            }).await?;
                        }
                    }


                    save_calendar(&new_calendar, &path).await?;
                }
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
    let now = Local::now();
    let mut diff = Vec::new();

    // Check for new or changed events
    for component in new.components.iter() {
        if let CalendarComponent::Event(e) = component {
            if e.get_uid().is_none() {
                println!("Warn: Event {} has no UID", e.get_summary().unwrap_or_default());
                continue;
            }

            if !is_in_future(&e, &now) {
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
        .filter(|e| is_in_future(e, &now))
        .for_each(|e| diff.push(Diff::Removed(e)));

    diff
}

fn get_event_by_uid<'a>(cal: &'a Calendar, uid: &str) -> Option<&'a Event> {
    cal.components.iter()
        .filter_map(|c| c.as_event())
        .filter(|e| e.get_uid().is_some())
        .find(|e| e.get_uid().unwrap() == uid)
}

async fn get_prev_calendar(path: impl AsRef<Path>) -> anyhow::Result<Option<Calendar>> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(None);
    }

    let file = tokio::fs::read_to_string(path).await?;

    Ok(Some(Calendar::from_str(&file).map_err(|e| anyhow!(e))?))
}

async fn save_calendar(calendar: &Calendar, path: impl AsRef<Path>) -> anyhow::Result<()> {
    tokio::fs::write(path, calendar.to_string()).await?;

    Ok(())
}

fn is_in_future(event: &Event, now: &DateTime<Local>) -> bool {
    let Some(end_time) = event.get_end() else { return true; };
    let Some(date_time) = parse_date_time(&end_time) else { return true; };

    date_time >= *now
}

fn parse_date_time(date: &DatePerhapsTime) -> Option<DateTime<Local>> {
    match date {
        DatePerhapsTime::DateTime(dt) => match dt {
            CalendarDateTime::Floating(f) => Some(f.and_local_timezone(Local).unwrap()),
            CalendarDateTime::Utc(utc) => Some(utc.with_timezone(&Local)),
            CalendarDateTime::WithTimezone { date_time, tzid } => {
                Tz::from_str(&tzid).map_or(None, |tz| {
                    Some(tz.from_local_datetime(&date_time).unwrap().with_timezone(&Local))
                })
            }
        }
        DatePerhapsTime::Date(_) => None
    }
}
