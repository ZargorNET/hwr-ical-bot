#![feature(try_blocks)]

use dotenvy::dotenv;
use std::env;
use std::str::FromStr;
use serenity::{async_trait, Client};
use serenity::client::{Context, EventHandler};
use serenity::model::gateway::Ready;
use serenity::model::prelude::{Activity, ChannelId};
use serenity::prelude::GatewayIntents;
use crate::fetcher::RunConfig;

mod fetcher;
mod embed_builder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().unwrap();

    let bot_token = env::var("BOT_TOKEN").expect("BOT_TOKEN not found");
    let channel_id = env::var("CHANNEL_ID").expect("CHANNEL_ID not found");
    let ics_url = env::var("ICS_URL").expect("ICS_URL");

    let mut client = Client::builder(bot_token, GatewayIntents::non_privileged())
        .event_handler(SerenityEventHandler)
        .await?;

    fetcher::run(RunConfig {
        channel_id: ChannelId(u64::from_str(&channel_id)?),
        discord_http: client.cache_and_http.http.clone(),
        ics_url,
    });

    client.start().await?;

    Ok(())
}

struct SerenityEventHandler;

#[async_trait]
impl EventHandler for SerenityEventHandler {
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        ctx.set_activity(Activity::competing("hwrical.zrgr.pw")).await;
    }
}
