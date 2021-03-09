use crate::ConnectorMessage;
use crate::MessageSource;
use serenity::http::client::Http;
use std::env;
use tokio::task;
use tokio::sync::mpsc::{Sender};
use tokio::sync::Mutex;
use std::sync::Arc;
use serenity::model::prelude::ChannelId;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

struct Handler {
    tx: Arc<Mutex<Sender<ConnectorMessage>>>
}

pub struct DiscordConnector {
    http: Http
}

impl DiscordConnector {

    pub fn new() -> DiscordConnector {
        let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

        let http = Http::new_with_token(&token);

        DiscordConnector {
            http: http 
        }
    }

    pub async fn send_msg(&self, c_id: u64, msg: String) {
        ChannelId(c_id as u64).say(&self.http, msg).await.expect("Error sending discord message!");
    }
}

#[async_trait]
    impl EventHandler for Handler {

        async fn message(&self, _ctx: Context, msg: Message) {

            if msg.author.name != "twtichdiscordrelay" && msg.author.name != "TwitchDiscordRelay" {
                let tx = &*self.tx.lock().await;
                if let Err(why) = tx.send(
                    ConnectorMessage{
                        author: msg.author.name,
                        message: msg.content,
                        source: MessageSource::DISCORD
                    }
                ).await {
                    println!("DISCORD >>> Error sending discord message over channel: {}", why);
                }
            }

        }

        async fn ready(&self, _: Context, ready: Ready) {
            println!("DISCORD >>> {} is connected!", ready.user.name);
        }
    }

pub async fn start(tx: Arc<Mutex<Sender<ConnectorMessage>>>) {
    task::spawn(setup(tx));
}

async fn setup(tx: Arc<Mutex<Sender<ConnectorMessage>>>) {

    println!("Discord setup function called..");

    let handler: Handler = Handler {
        tx: tx
    };

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = Client::builder(&token)
        .event_handler(handler)
        .await
        .expect("DISCORD >>> Err creating client");

    if let Err(why) = client.start().await {
        println!("DISCORD >>> Client error: {:?}", why);
    }
}