// use common::tid;
use common::Result;
use common::listener::accept_loop;
use async_std::task;
// use async_std::prelude::*;
// use std::time::Duration;
const ADDRESS: &str= "127.0.0.1:8080";


pub(crate) fn main() -> Result<()> {
    println!("Starting (group) chat, listening @ {}", ADDRESS);
    task::block_on(accept_loop(ADDRESS))
    // let mut futures = Vec::new();
    // for i in 0..10 {
    //     futures.push(async move {
    //         std::thread::sleep(Duration::from_secs(2));
    //         println!("{} {:?} Slept", i, tid());
    //     });
    // }
    // let mut results = Vec::new();
    // for fut in futures {
    //     results.push(task::spawn(fut));
    // }
    // for fut in results {
    //     task::block_on(fut);
    // }
    // Ok(())
}
