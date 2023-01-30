#![feature(try_blocks)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use serenity::{async_trait, Client};
use serenity::client::{Context, EventHandler};
use serenity::model::gateway::Ready;
use serenity::model::prelude::Activity;
use serenity::prelude::GatewayIntents;

mod fetcher;
mod embed_builder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = Path::new("config.toml");
    if !config_path.exists() {
        println!("Created config file.");
        tokio::fs::write(config_path, toml::to_string(&Config::default())?).await?;

        return Ok(());
    }

    let config: Config = toml::from_str(&tokio::fs::read_to_string(config_path).await?)?;

    let mut client = Client::builder(&config.bot_token, GatewayIntents::non_privileged())
        .event_handler(SerenityEventHandler)
        .await?;

    fetcher::run(client.cache_and_http.http.clone(), config);

    client.start().await?;

    Ok(())
}

struct SerenityEventHandler;

#[async_trait]
impl EventHandler for SerenityEventHandler {
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        ctx.set_activity(Activity::listening("hwrical.zrgr.pw")).await;
    }
}


#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Config {
    bot_token: String,
    endpoints: Vec<Endpoint>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Endpoint {
    channel_id: u64,
    ics_url: String,
    display_name: String,
}
