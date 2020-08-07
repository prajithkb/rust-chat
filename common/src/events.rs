
use crate::Receiver;
use async_std::sync::Arc;
use crate::models::Message;
use crate::{Void, models::Participant};
use async_std::net::TcpStream;

#[derive(Debug)]
pub enum Event {
    NewPeer {
        participant: Participant,
        stream: Arc<TcpStream>,
        shutdown: Receiver<Void>,
    },
    Message {
        msg: Message,
    },
    Command {
        cmd: Command,
        from: Participant
    }
}
#[derive(Debug)]
pub enum Command {
    Help,
    List,
    Invalid
}
