use common::Result;
use common::listener::accept_loop;
use async_std::task;

pub(crate) fn main() -> Result<()> {
    task::block_on(accept_loop("127.0.0.1:8080"))
}
