use futures::{channel::mpsc, SinkExt};
use tokio::io::{AsyncBufReadExt, BufReader, Lines, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::stream::StreamExt;

use crate::prelude::*;

pub struct Client {}

impl Client {
	pub async fn new(stream: TcpStream, channel_sender: Sender<Event>) -> (String, Sender<String>) {
		let (reader, writer) = tokio::io::split(stream);
		let reader = BufReader::new(reader);
		let mut lines = reader.lines();
		let nick = match lines.next().await {
			None => String::new(),
			Some(line) => line.unwrap(),
		};

		let (sender, receiver): (Sender<String>, Receiver<String>) = mpsc::unbounded();

		tokio::spawn(Self::reader(lines, channel_sender, nick.clone()));
		tokio::spawn(Self::writer(writer, receiver));

		(nick, sender)
	}

	async fn reader(
		mut lines: Lines<BufReader<ReadHalf<TcpStream>>>,
		mut channel_sender: Sender<Event>,
		nick: String,
	) {
		while let Some(msg) = lines.next().await {
			let msg = msg.unwrap().trim().to_string();

			if msg.is_empty() {
				continue;
			}

			let event = Event::Message {
				nick: nick.clone(),
				msg,
			};
			channel_sender.send(event).await.unwrap();
		}
	}
	async fn writer(mut writer: WriteHalf<TcpStream>, mut receiver: Receiver<String>) {
		while let Some(msg) = receiver.next().await {
			writer.write_all(msg.as_bytes()).await.unwrap();
		}
	}
}
