use std::io::{Write};
use common::models::Message;
use futures::select;
use futures::FutureExt;


use async_std::{
    io::{stdin, BufReader},
    net::{TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};
use common::Result;

pub(crate) fn main() -> Result<()> {
    task::block_on(try_main("127.0.0.1:8080"))
}

fn prompt(name: &str) {
    print!("[{}]:>", name);
    flush_stdout();
}
fn flush_stdout() {
    std::io::stdout().flush().unwrap();
}

fn print_server_message(msg: &Message) -> Result<()> {
    print!("{}", termion::color::Fg(termion::color::Reset));
    // let rgb = msg.display_rgb
    //                                             .map(| (r,g, b)| termion::color::Rgb(r, g, b))
    //                                             // .map(|rgb| rgb.fg_string())
    //                                             .map(|rgb| rgb)
    //                                             .unwrap_or(termion::color::Rgb(0,0,0));
    println!("{}\r{}[{}]:> {}", 
            termion::clear::CurrentLine, 
            termion::color::Fg(termion::color::LightBlack), 
            msg.from.name, 
            msg.msg);
    print!("{}", termion::color::Fg(termion::color::Reset));
    flush_stdout();
    Ok(())
}

async fn send(str: &str, mut writer: &TcpStream) -> Result<()> {
    writer.write_all(str.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    Ok(())
}

async fn try_main(addr: impl ToSocketAddrs) -> Result<()> {
    let stream = TcpStream::connect(addr).await?;
    let (reader, mut writer) = (&stream, &stream);
    let reader = BufReader::new(reader);
    let mut lines_from_server = futures::StreamExt::fuse(reader.lines());

    let stdin = BufReader::new(stdin());
    print!("Enter your name: ");
    flush_stdout();
    let mut lines_from_stdin = futures::StreamExt::fuse(stdin.lines());
    let name = lines_from_stdin.next().await.unwrap()?;
    println!("Welcome [{}], connecting to server..", name);
    send(&name, & mut writer).await?;
    loop {
        prompt(&name);
        select! {
            line = lines_from_server.next().fuse() => match line {
                Some(line) => {
                    let line = line?;
                    let message: Message = serde_json::from_str(&line)?;
                    print_server_message(&message)?;
                },
                None => break,
            },
            line = lines_from_stdin.next().fuse() => match line {
                Some(line) => {
                    let line = line?;
                    send(&line, & mut writer).await?;
                }
                None => break,
            },

        }
    }
    Ok(())
}