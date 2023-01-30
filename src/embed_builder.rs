use std::str::FromStr;

use chrono::{Local, TimeZone};
use icalendar::{CalendarDateTime, Component, DatePerhapsTime, Event};
use serenity::builder::CreateEmbed;
use serenity::utils::Color;

use crate::fetcher::Diff;

const DATE_FORMAT: &str = "%a, %d.%m.%y %R";
const MAX_EVENT_LIMIT: usize = 10;

pub fn build_embed(diff: Vec<Diff>, title_course: &str) -> Vec<CreateEmbed> {
    let now = Local::now();
    let date_and_time = now.format(DATE_FORMAT).to_string();
    let title = format!("Stundenplanänderungen {} {}", title_course, &date_and_time);


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


    let mut embed = CreateEmbed::default();
    embed
        .title(title)
        .timestamp(now)
        .footer(|f| f.text("Stundenplan in Google Kalendar: https://hwrical.zrgr.pw"));

    let removed_string = create_end_string(removed);
    let created_string = create_end_string(created);

    let mut removed_embed = embed.clone();
    removed_embed
        .description(&removed_string)
        .color(Color::new(0xFF0000));

    let mut created_embed = embed.clone();
    created_embed
        .description(&created_string)
        .color(Color::new(0x00FF00));

    let mut result_vec = Vec::with_capacity(2);
    if !removed_string.is_empty() {
        result_vec.push(removed_embed)
    }
    if !created_string.is_empty() {
        result_vec.push(created_embed);
    }

    result_vec
}

fn create_end_string(diff_strings: Vec<String>) -> String {
    if diff_strings.is_empty() {
        return String::new();
    }

    let mut changes = String::from("```diff\n");
    diff_strings.into_iter().for_each(|s| changes.push_str(&s));
    changes.push_str("```");

    changes
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
