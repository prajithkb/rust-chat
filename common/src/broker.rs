use crate::models::Participant;
use crate::spawn_and_log_error;
use crate::writer::connection_writer_loop;
use crate::Sender;
use crate::{models::Message, Command, Event, Receiver};
use std::collections::hash_map::{Entry, HashMap};

use futures::{channel::mpsc, select, FutureExt, SinkExt};

use async_std::prelude::*;
const SERVER: &str = "Server";
pub async fn broker_loop(mut events: Receiver<Event>) {
    let (disconnect_sender, mut disconnect_receiver) =
        mpsc::unbounded::<(String, Receiver<Message>)>();
    let mut peers: HashMap<String, (Participant, Sender<Message>)> = HashMap::new();
    loop {
        let event = select! {
            event = events.next().fuse() => match event {
                None => break,
                Some(event) => event,
            },
            disconnect = disconnect_receiver.next().fuse() => {
                let (name, _pending_messages) = disconnect.unwrap();
                assert!(peers.remove(&name).is_some());
                continue;
            },
        };
        match event {
            Event::Command { cmd, from } => match cmd {
                Command::Help => {
                    send_to(&from.id, message_from_server("Possible commands: [list|help]"), &mut peers).await
                }
                Command::List => {
                    send_to(&from.id, message_from_server(&list(&peers)), &mut peers).await
                }
                Command::Invalid => {
                    send_to(&from.id, message_from_server("Invalid command"), &mut peers).await
                }
            },
            Event::Message { msg } => {
                broadcast(msg, &mut peers).await;
            }
            Event::NewPeer {
                participant,
                stream,
                shutdown,
            } => match peers.entry(participant.id.clone()) {
                Entry::Occupied(..) => (),
                Entry::Vacant(entry) => {
                    let (client_sender, mut client_receiver) = mpsc::unbounded();
                    let id = entry.key().to_string();
                    let (participant, client_sender) = entry.insert((participant, client_sender));
                    let mut disconnect_sender = disconnect_sender.clone();
                    spawn_and_log_error(async move {
                        let res =
                            connection_writer_loop(&mut client_receiver, stream, shutdown).await;
                        disconnect_sender.send((id, client_receiver)).await.unwrap();
                        res
                    });
                    let Participant { name, id } = &participant;
                    client_sender
                        .send(message_from_server(&format!(
                            "Welcome [{}]! type @<command> for running commands.",
                            name
                        )))
                        .await
                        .unwrap();
                    broadcast(
                        message(&format!("[{}] Connected.", name), SERVER, id),
                        &mut peers,
                    )
                    .await;
                }
            },
        }
    }
    drop(peers);
    drop(disconnect_sender);
    while let Some((_name, _pending_messages)) = disconnect_receiver.next().await {}
}

async fn broadcast(msg: Message, peers: &mut HashMap<String, (Participant, Sender<Message>)>) {
    for (participant_id, (_, sender)) in peers.iter_mut() {
        // Send only to other peers (exclude self)
        if *participant_id != msg.from.id {
            let m = msg.clone();
            sender.send(m).await.unwrap();
        }
    }
}

fn message_from_server(msg: &str) -> Message {
    message(msg, SERVER, SERVER)
}


fn message(msg: &str, from: &str, id: &str) -> Message {
    Message {
        from: Participant {
            name: from.into(),
            id: id.into(),
        },
        msg: msg.into(),
        display_rgb: Some((18, 128, 0)),
    }
}

fn list(peers: &HashMap<String, (Participant, Sender<Message>)>) -> String {
    let peers = peers
        .values()
        .map(|(participant, _)| &*participant.name)
        .collect::<Vec<&str>>()
        .join(",");
    return format!("List or participants: [{}]", peers);
}

async fn send_to(
    participant_id: &str,
    msg: Message,
    peers: &mut HashMap<String, (Participant, Sender<Message>)>,
) {
    let (_, sender): &mut (Participant, Sender<Message>) =
        peers.get_mut(participant_id).expect("msg: &str");
    sender.send(msg).await.unwrap();
}
