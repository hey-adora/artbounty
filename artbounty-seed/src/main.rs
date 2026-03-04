use std::{collections::HashMap, env, time::Duration};

use ab_glyph::{FontRef, PxScale};
use artbounty::{
    api::{Api, ApiNative, ServerReqImg},
    path::{
        PATH_API_SEND_EMAIL_INVITE, PATH_API_LOGIN, PATH_API_POST_ADD, PATH_API_REGISTER, PATH_API_USER,
    },
};
use clap::{Command, arg};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text;
use rand::Rng;
use tokio::fs;
use tracing::{info, trace};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .unwrap();

    let command = Command::new("seed")
        .about("data seeder")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("user")
                .about("manage users")
                .arg(arg!(--"token" <TOKEN>).required(true))
                .arg(arg!(--"count" <COUNT>).required(true))
                .arg_required_else_help(true),
        );
    let matches = command.get_matches();
    match matches.subcommand() {
        Some(("user", sub_matches)) => {
            let token = sub_matches.get_one::<String>("token").unwrap();
            let count = sub_matches.get_one::<String>("count").unwrap();
            let count = u32::from_str_radix(count, 10).unwrap();
            let path = "/tmp/img.png";
            let api = ApiNative::new("http://localhost:3000");
            let mut rng = rand::rng();

            for i in 0..count {

                let mut image = RgbImage::new(200, 200);
                let r = rng.random_range(200u8..255);
                let g = rng.random_range(200u8..255);
                let b = rng.random_range(200u8..255);
                for (x, y, pixel) in image.enumerate_pixels_mut() {
                    *pixel = image::Rgb([r, g, b]);
                }
                let height = 64.0;
                let scale = PxScale {
                    x: height * 2.0,
                    y: height,
                };

                let font = FontRef::try_from_slice(include_bytes!("../../assets/noto_sans.ttf")).unwrap();
                let img = draw_text(
                    &mut image,
                    Rgb([0u8, 0u8, 0u8]),
                    0,
                    50,
                    scale,
                    &font,
                    &i.to_string(),
                );
                img.save(path).unwrap();

                let img = fs::read(path).await.unwrap();

                let result = api
                    .add_post(
                        "title1",
                        "wow",
                        Vec::from([ServerReqImg {
                            path: path.to_string(),
                            data: img.clone(),
                        }]),
                    )
                    .send_native_with_token(token.clone())
                    .await
                    .unwrap();
            }
        }
        _ => unreachable!(),
    }
}
