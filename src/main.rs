#![allow(unused_imports)]

mod resp;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::run().await
}
