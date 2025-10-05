use artbounty::server::server;
use leptos::{logging, prelude::*};
use tracing::trace;

#[tokio::main]
async fn main() {
    server().await;
}

// pub mod api2 {}
// pub mod middleware2 {
//     pub mod auth {
//         use axum::body::Body;
//         use tracing::trace;
//
//         pub async fn auth(
//             _req: axum::extract::Request,
//             _next: axum::middleware::Next,
//         ) -> axum::response::Response {
//             let r2 = axum::response::Response::builder()
//                 .status(403)
//                 .body(Body::empty())
//                 .unwrap();
//             trace!("hello666");
//
//             r2
//         }
//     }
// }
