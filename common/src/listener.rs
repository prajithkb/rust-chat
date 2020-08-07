//! Listens for incoming connections and handles them

use crate::broker::broker_loop;
use crate::Result;
use crate::Sender;
use crate::Void;
use crate::{
    events::Command,
    events::Event,
    models::{Message, Participant},
    spawn_and_log_error, tid,
};
use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};
use futures::{channel::mpsc, SinkExt};
use std::sync::Arc;

pub async fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;

    let (broker_sender, broker_receiver) = mpsc::unbounded();
    let broker = spawn_and_log_error(broker_loop(broker_receiver), "Listener".into());
    let mut incoming = listener.incoming();
    println!("Waiting for incoming connections.");
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let id = stream.peer_addr()?;
        println!("Accepting from: {}", id);
        spawn_and_log_error(
            connection_loop(broker_sender.clone(), stream),
            format!("connection-[{}]", id),
        );
    }
    drop(broker_sender);
    broker.await;
    Ok(())
}

async fn connection_loop(mut broker: Sender<Event>, stream: TcpStream) -> Result<()> {
    println!("[{:?}] Starting connection loop.", tid());
    let stream = Arc::new(stream);
    let reader = BufReader::new(&*stream);
    let mut lines = reader.lines();
    let addr = stream.peer_addr()?.to_string();
    println!("Waiting for incoming messages.");
    let name = match lines.next().await {
        None => {
            return Err(format!("{} disconnected", addr).into());
        }
        Some(line) => line?,
    };
    let (_shutdown_sender, shutdown_receiver) = mpsc::unbounded::<Void>();
    let participant = Participant {
        name: name.clone(),
        id: addr,
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

    loop {
        match lines.next().await {
            Some(line) => {
                let line = line?;
                let event = parse(line, participant.clone());
                println!("[{:?}] Received {:?} ", tid(), event);
                broker.send(event).await.unwrap();
            }
            None => {
                break;
            }
        }
    }
    Ok(())
}

fn parse(line: String, participant: Participant) -> Event {
    if line.starts_with("@") {
        Event::Command {
            cmd: parse_command(&line[1..]),
            from: participant,
        }
    } else {
        Event::Message {
            msg: Message {
                from: participant,
                msg: line.trim().to_string(),
                display_rgb: None,
            },
        }
    }
}

fn parse_command(line: &str) -> Command {
    match line {
        "list" => Command::List,
        "help" => Command::Help,
        _ => Command::Invalid,
    }
}
