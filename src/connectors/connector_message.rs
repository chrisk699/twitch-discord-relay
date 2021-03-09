pub enum MessageSource {
    TWITCH,
    DISCORD,
}

pub struct ConnectorMessage {
	pub author: String,
	pub message: String,
	pub source: MessageSource
}