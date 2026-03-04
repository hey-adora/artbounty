#![recursion_limit = "512"]

use artbounty::server::server;

#[tokio::main]
async fn main() {
    server().await;
}
