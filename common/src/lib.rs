pub mod broker;
pub mod events;
pub mod listener;
pub mod models;
pub mod writer;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

use async_std::{prelude::*, task};
use futures::channel::mpsc;

#[derive(Debug)]
pub enum Void {}

pub fn spawn_and_log_error<F>(fut: F, name: String) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    let n = name.clone();
    task::Builder::new().name(name).spawn(async move {
        println!("Running task {} in {:?}", n, tid());
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    }).expect("Unable to create task")
}

pub fn tid() -> std::thread::ThreadId {
    std::thread::current().id()
}
