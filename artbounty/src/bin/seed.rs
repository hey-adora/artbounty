use artbounty::{
    controller::{self, encode::send_native},
    path::PATH_API_USER,
};

#[tokio::main]
async fn main() {
    let res = send_native::<
        controller::auth::route::invite::ServerOutput,
        controller::auth::route::invite::ServerErr,
    >(
        "http://localhost:3000",
        PATH_API_USER,
        None::<&str>,
        &controller::auth::route::invite::Input {
            email: "hey1@hey.com".to_string(),
        },
    )
    .await
    .unwrap();
    println!("huh");
}
