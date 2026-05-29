#![allow(unused_imports)]

mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::run().await
}
