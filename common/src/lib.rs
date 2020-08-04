pub mod listener;
pub mod broker;
pub mod writer;
pub mod models;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

use async_std::sync::Arc;
use async_std::net::TcpStream;
use futures::channel::mpsc;
use async_std::{
    prelude::*,
    task,
};
use models::{Message, Participant};

#[derive(Debug)]
pub enum Void {}
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


pub fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    })
}