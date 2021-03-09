use crate::connectors::connector_message::ConnectorMessage;
use crate::connectors::connector_message::MessageSource;
use crate::connectors::discord::DiscordConnector;
use crate::connectors::twitch::TwitchConfig;
use crate::connectors::twitch::TwitchConnector;

use std::env;
use std::io::{self};
use tokio::task;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio::time::sleep;

mod connectors;

const DISCORD_TOKEN: &str = "DISCORD_TOKEN HERE";
const DISCORD_BOT_CHANNEL: u64 = 0000000; // DISCORD CHANNEL ID HERE

const TWITCH_CONFIG: TwitchConfig = TwitchConfig {
    user: "TWITCH USER NAME HERE",
    pass: "TWITCH OAUTH TOKEN HERE"
};

pub async fn process_console_input(twitch_arc_input: Arc<Mutex<TwitchConnector>>) {
    let mut input_buffer = String::new();
    io::stdin().read_line(&mut input_buffer).expect("Failed to read from stdin");
    let twitch_ref = &*twitch_arc_input.lock().await;
    twitch_ref.send_msg("#arresteverybody", &input_buffer).await;
}

#[tokio::main]
async fn main() {
    // Create sharable mpsc channel
    let (tx, mut rx): (Sender<ConnectorMessage>, Receiver<ConnectorMessage>) = mpsc::channel(50);
    let discord_arc_tx: Arc<Mutex<Sender<ConnectorMessage>>> = Arc::new(Mutex::new(tx.clone()));
    let twitch_arc_tx: Arc<Mutex<Sender<ConnectorMessage>>> = Arc::new(Mutex::new(tx.clone()));

    // Setup env with discord token and bot channel
    env::set_var("DISCORD_TOKEN", DISCORD_TOKEN);
    env::set_var("DISCORD_BOT_CHANNEL", DISCORD_BOT_CHANNEL.to_string());
    // Specify discord channel id that is going to be used by the bot
    
    // Start discord handler thread
    connectors::discord::start(discord_arc_tx).await;
    // Get discord connector in order to send messages later
    let discord: DiscordConnector = DiscordConnector::new();

    // Setup twitch connector
    let twitch = connectors::twitch::TwitchConnector::new(TWITCH_CONFIG, twitch_arc_tx);
    let twitch_arc: Arc<Mutex<TwitchConnector>> = Arc::new(Mutex::new(twitch));

    // Send user auth commands
    twitch_arc.lock().await.auth_user().await; 
    // Join specified channel           
    twitch_arc.lock().await.join_channel("arresteverybody").await;
    // Start reader thread
    twitch_arc.lock().await.start_reader_thread();

    // Spawn console input processor thread
    println!("Starting input handler thread.");
    let twitch_arc_input = twitch_arc.clone();
    task::spawn(async move {
        
        loop {

            process_console_input(twitch_arc_input.clone()).await;
        }
    });

    // Spawn mpsc receiver thread that will handle coordination between twitch and discord
    println!("Started Channel receiver.");
    let bot_channel = env::var("DISCORD_BOT_CHANNEL")
        .expect("Expected bot channel in the environment")
        .parse::<u64>()
        .expect("Could not parse bot channel from ENV.");

    // Loop endlessly and handle coordination
    loop {
       
        // Receive channel messages
        if let Some(msg) = rx.recv().await {

            /*
            * Handle message relay
            *   From Twitch -> To Discord
            *   From Discord -> To Twitch
            */
            match msg.source {
                MessageSource::TWITCH => discord.send_msg(bot_channel, format!("{}: {}", msg.author, msg.message)).await,
                MessageSource::DISCORD => twitch_arc.lock().await.send_msg("#arresteverybody", &format!("{}: {}", msg.author, msg.message)).await
            }
        }

        sleep(Duration::from_millis(10)).await;
    }
}