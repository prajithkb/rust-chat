//! Listens for incoming connections and handles them

use crate::broker::broker_loop;
use crate::Void;
use async_std::{
    io::BufReader,
    net::{TcpStream, ToSocketAddrs, TcpListener},
    prelude::*,
    task,
};
use futures::{SinkExt, channel::mpsc};
use crate::Result;
use crate::Sender;
use crate::{Event, spawn_and_log_error, models::{Message, Participant}, Command};
use std::{sync::Arc};

pub async fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;

    let (broker_sender, broker_receiver) = mpsc::unbounded();
    let broker = task::spawn(broker_loop(broker_receiver));
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        println!("Accepting from: {}", stream.peer_addr()?);
        spawn_and_log_error(connection_loop(broker_sender.clone(), stream));
    }
    drop(broker_sender);
    broker.await;
    Ok(())
}

async fn connection_loop(mut broker: Sender<Event>, stream: TcpStream) -> Result<()> {
    let stream = Arc::new(stream);
    let reader = BufReader::new(&*stream);
    let mut lines = reader.lines();
    let addr = stream.peer_addr()?.to_string();
    let name = match lines.next().await {
        None => return Err(format!("{} disconnected", addr).into()),
        Some(line) => line?,
    };
    let (_shutdown_sender, shutdown_receiver) = mpsc::unbounded::<Void>();
    let participant = Participant {
        name: name.clone(),
        id: addr
    };
    println!("{:?} connected", participant);
    let p = participant.clone();
    broker
        .send(Event::NewPeer {
            participant: p,
            stream: Arc::clone(&stream),
            shutdown: shutdown_receiver,
        })
        .await
        .unwrap();

    while let Some(line) = lines.next().await {
        let line = line?;
        let event = parse(line, participant.clone());
        println!("Received {:?} ", event);
        broker
            .send(event)
            .await
            .unwrap();
    }

    Ok(())
}

fn parse(line: String, participant: Participant) -> Event {
    if line.starts_with("@") {
        Event::Command {
            cmd: parse_command(&line[1..]),
            from: participant
        }
    } else {
        Event::Message {
            msg: Message {
                from: participant,
                msg: line.trim().to_string(),
                display_rgb: None
            }
        }
    }
}

fn parse_command(line: &str) -> Command {
    match line {
        "list" => Command::List,
        "help" => Command::Help,
        _ => Command::Invalid
    }
}
