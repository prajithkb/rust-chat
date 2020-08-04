use crate::models::Message;
use crate::Void;
use crate::Receiver;
use crate::Result;
use std::{
    sync::Arc,
};

use futures::{select, FutureExt};

use async_std::{
    net::{TcpStream},
    prelude::*,
};

pub(crate)  async fn connection_writer_loop(
    messages: &mut Receiver<Message>,
    stream: Arc<TcpStream>,
    mut shutdown: Receiver<Void>,
) -> Result<()> {
    let mut stream = &*stream;
    loop {
        select! {
            msg = messages.next().fuse() => match msg {
                Some(msg) => {
                    println!("Writing {:?} to {}", msg, stream.peer_addr()?.to_string());
                    let text = serde_json::to_string(&msg)?;
                    stream.write_all(text.as_bytes()).await?;
                    // Every message needs to be line terminated
                    stream.write_all(b"\n").await?;
                },
                None => break,
            },
            void = shutdown.next().fuse() => match void {
                Some(void) => match void {},
                None => break,
            }
        }
    }
    Ok(())
}