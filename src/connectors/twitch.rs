use crate::connectors::irc_parser::IrcMessage;
use crate::connectors::irc_parser::IrcParser;
use crate::ConnectorMessage;
use crate::MessageSource;
use std::net::{TcpStream};
use std::io::{Write, BufRead, BufReader};
use tokio::task;
use std::sync::Arc;
use tokio::sync::mpsc::{Sender};
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio::time::sleep;

pub struct TwitchConfig {
    pub user: &'static str,
    pub pass: &'static str
}

pub struct TwitchConnector {
    config: TwitchConfig,
    reader: Arc<tokio::sync::Mutex<TcpStream>>,
    writer: Arc<tokio::sync::Mutex<TcpStream>>,
    tx_arc: Arc<tokio::sync::Mutex<Sender<ConnectorMessage>>>
}

impl TwitchConnector {

    pub fn new(config: TwitchConfig, tx: Arc<Mutex<Sender<ConnectorMessage>>>) -> TwitchConnector {
        let reader;
        let writer;

        match TcpStream::connect("irc.twitch.tv:6667") {
            Ok(stream) => {
                println!("TWITCH >>> Successfully connected to server in port 6667");

                reader = stream.try_clone().expect("Could not clone stream!");
                writer = stream.try_clone().expect("Could not clone stream!"); 
            },
            Err(e) => {
                panic!("TWITCH >>> Failed to setup twitch connection! Error: {}", e);
            }
        }

        TwitchConnector{
            config: config,
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            tx_arc: tx
        }

    }

    pub fn start_reader_thread(&self) {

        let cloned_stream = self.reader.clone();
        let cloned_tx = self.tx_arc.clone();
        task::spawn(async move {
            let stream = &*cloned_stream.lock().await;
            let tx = &*cloned_tx.lock().await;
            let parser = IrcParser::new();

            loop {

                TwitchConnector::get_reply(stream, &parser, tx).await;
                sleep(Duration::from_millis(10)).await;
            }
        });
    }

    async fn get_reply(stream: &TcpStream, parser: &IrcParser, sender: &Sender<ConnectorMessage>) {
        let reader = BufReader::new(stream);
        
        for res in reader.lines() {
            match res {
                Ok(line) => {
                    let irc_msg: IrcMessage = parser.parse(line.to_string());

                    if irc_msg.nick != "tmi.twitch.tv" && irc_msg.nick != "twtichdiscordrelay.tmi.twitch.tv" && irc_msg.nick != "twtichdiscordrelay" {

                        if irc_msg.command == "PRIVMSG" {
                            if let Err(why) = sender.send(
                                ConnectorMessage{
                                    author: irc_msg.nick,
                                    message: irc_msg.params.get(1).unwrap().to_string(),
                                    source: MessageSource::TWITCH
                                }
                            ).await {
                                println!("TWITCH >>> Error sending twitch message over channel: {}", why);
                            }
                        }
                    }

                },
                _ => {},
            }
        }
    }

    pub async fn auth_user(&self) {
        let mut stream = &*self.writer.lock().await;

        let mut command: String = "PASS ".to_owned();
        command.push_str(self.config.pass);
        command.push_str("\r\n");
        stream.write(command.as_bytes()).unwrap();
        println!("TWITCH >>> Sent PASS Command..");

        let mut command: String = "NICK ".to_owned();
        command.push_str(self.config.user);
        command.push_str("\r\n");
        stream.write(command.as_bytes()).unwrap();
        println!("TWITCH >>> Sent NICK Command..");

        stream.write(b"CAP REQ :twitch.tv/membership\r\n").unwrap();
        println!("TWITCH >>> Sent CAP REQ Command..");
    }

    pub async fn send_msg(&self, receiver: &str, msg: &str) {
        let mut stream = &*self.writer.lock().await;

        if msg == "\r\n" {
            return;
        }

        let mut msg_copy = String::from(msg.clone());
        msg_copy.truncate(msg_copy.len() -2);

        let mut command: String = "PRIVMSG ".to_owned();
        command.push_str(receiver);
        command.push_str(" :");
        command.push_str(msg);
        command.push_str("\r\n");

        stream.write(command.as_bytes()).unwrap();
    }

    pub async fn join_channel(&self, channel: &str) {
        let mut stream = &*self.writer.lock().await;

        let mut msg_command: String = "JOIN #".to_owned();
        msg_command.push_str(channel);
        msg_command.push_str("\r\n");

        stream.write(msg_command.as_bytes()).unwrap();
        println!("TWITCH >>> Sent JOIN #{chan} Command..", chan=channel);
    }
}


