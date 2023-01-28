use std::str::FromStr;
use chrono::{Local, NaiveDateTime, TimeZone};
use icalendar::{CalendarDateTime, Component, DatePerhapsTime, Event};
use serenity::builder::{CreateEmbed};
use serenity::utils::Color;
use crate::fetcher::Diff;

const DATE_FORMAT: &str = "%a, %d.%m.%y %R";
const MAX_EVENT_LIMIT: usize = 10;

pub fn build_embed(diff: Vec<Diff>) -> CreateEmbed {
    let date_and_time = Local::now().format(DATE_FORMAT).to_string();
    let title = format!("Stundenplanänderungen {}", &date_and_time);

    let mut changes = String::from("```diff\n");

    let mut created = Vec::new();
    let mut removed = Vec::new();

    for x in diff {
        match x {
            Diff::Created(e) => created.push(format!("+{}\n\n", format_event(e))),
            Diff::Changed { new, old } => {
                created.push(format!("+{}\n\n", format_event(new)));
                removed.push(format!("-{}\n\n", format_event(old)));
            }
            Diff::Removed(e) => removed.push(format!("-{}\n\n", format_event(e))),
        }
    }

    truncate_event_vec(&mut created);
    truncate_event_vec(&mut removed);

    created.into_iter().for_each(|s| changes.push_str(&s));
    removed.into_iter().for_each(|s| changes.push_str(&s));

    changes.push_str("```");


    let mut embed = CreateEmbed::default();
    embed
        .title(title)
        .color(Color::new(0x33cd09)) // green
        .description(changes);

    embed
}

fn truncate_event_vec(vec: &mut Vec<String>) {
    let len = vec.len();

    if len > MAX_EVENT_LIMIT {
        vec.truncate(MAX_EVENT_LIMIT);
        vec.push(format!("... und {} weitere", len - vec.len()));
    }
}

fn format_event(e: &Event) -> String {
    let start = e.get_start().map_or("???".to_string(), |d| parse_date_time(d));
    let end = e.get_end().map_or("???".to_string(), |d| parse_date_time(d));

    format!("{} von {} bis {}", e.get_summary().unwrap_or("???"), start, end)
}

fn parse_date_time(date: DatePerhapsTime) -> String {
    match date {
        DatePerhapsTime::DateTime(dt) => match dt {
            CalendarDateTime::Floating(f) => f.format(DATE_FORMAT).to_string(),
            CalendarDateTime::Utc(utc) => utc.with_timezone(&Local).format(DATE_FORMAT).to_string(),
            CalendarDateTime::WithTimezone { date_time, tzid } => {
                chrono_tz::Tz::from_str(&tzid).map_or("???".to_string(), |tz| {
                    Local.from_local_datetime(&date_time)
                        .unwrap()
                        .with_timezone(&tz)
                        .format(DATE_FORMAT).to_string()
                })
            }
        }
        DatePerhapsTime::Date(d) => d.format(DATE_FORMAT).to_string()
    }
}
