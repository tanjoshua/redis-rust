#![allow(unused_imports)]

mod resp;
mod server;
mod store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::run().await
}
