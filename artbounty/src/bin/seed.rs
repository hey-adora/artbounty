use std::{collections::HashMap, env, time::Duration};

use ab_glyph::{FontRef, PxScale};
use artbounty::{
    api::{Api, ApiNative, ServerReqImg},
    path::{PATH_API_INVITE, PATH_API_LOGIN, PATH_API_POST_ADD, PATH_API_REGISTER, PATH_API_USER},
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

    // let args: Vec<String> = env::args().collect();
    // let help = "available commands: api invite";
    // let command1 = args.get(1).cloned().unwrap_or_default();
    // let command2 = args.get(2).cloned().unwrap_or_default();
    // let command3 = args.get(3).cloned().unwrap_or_default();
    // let command4 = args.get(4).cloned().unwrap_or_default();
    // let command5 = args.get(5).cloned().unwrap_or_default();
    // let settings = Settings::new_from_file();
    //
    // match (command1.as_str(), command2.as_str()) {
    //     ("api", "invite") => {
    //         let name = command3;
    //         if name.len() < 1 {
    //             panic!("invalid command, example: api invite name1");
    //         }
    //         // let user = {
    //         //     info!("enter username: ");
    //         //     let mut line = String::new();
    //         //     // std::io::stdin().read_line(&mut line).unwrap();
    //         //     line.trim().to_string()
    //         // };
    //         let email = format!("{}@hey.com", name);
    //
    //         let res = send_native::<
    //             controller::auth::route::invite::ServerOutput,
    //             controller::auth::route::invite::ServerErr,
    //         >(
    //             &settings.site.address,
    //             PATH_API_INVITE,
    //             None::<&str>,
    //             &controller::auth::route::invite::Input {
    //                 email: email.clone(),
    //             },
    //         )
    //         .await
    //         .unwrap();
    //
    //         trace!("res: {res:#?}");
    //     }
    //     ("api", "register") => {
    //         // let exp = time + Duration::from_secs(60 * 30);
    //         // let invite = InviteToken::new("hey2@hey.com", time, exp);
    //         // let invite_token = encode_token(&settings.auth.secret, invite).unwrap();
    //         let name = command3;
    //         if name.len() < 1 {
    //             panic!("invalid name arg, example: api register name1 token");
    //         }
    //         let token = command4;
    //         if token.len() < 1 {
    //             panic!("invalid token arg, example: api register name1 token");
    //         }
    //
    //         let email = format!("{}@hey.com", name);
    //
    //         // let token = {
    //         //     info!("enter token: ");
    //         //     let mut line = String::new();
    //         //     std::io::stdin().read_line(&mut line).unwrap();
    //         //     line.trim().to_string()
    //         // };
    //
    //         info!("token entered: {token}");
    //
    //         let res = send_native::<
    //             controller::auth::route::register::ServerOutput,
    //             controller::auth::route::register::ServerErr,
    //         >(
    //             &settings.site.address,
    //             PATH_API_REGISTER,
    //             None::<&str>,
    //             &controller::auth::route::register::Input {
    //                 username: name.clone(),
    //                 email_token: token.clone(),
    //                 password: email,
    //             },
    //         )
    //         .await
    //         .unwrap();
    //
    //         trace!("res: {res:#?}");
    //     }
    //     ("api", "login") => {
    //         // let exp = time + Duration::from_secs(60 * 30);
    //         // let invite = InviteToken::new("hey2@hey.com", time, exp);
    //         // let invite_token = encode_token(&settings.auth.secret, invite).unwrap();
    //         let name = command3;
    //         if name.len() < 1 {
    //             panic!("invalid name arg, example: api login name1");
    //         }
    //
    //         let email = format!("{}@hey.com", name);
    //
    //         // let token = {
    //         //     info!("enter token: ");
    //         //     let mut line = String::new();
    //         //     std::io::stdin().read_line(&mut line).unwrap();
    //         //     line.trim().to_string()
    //         // };
    //
    //         let res = send_native::<
    //             controller::auth::route::login::ServerOutput,
    //             controller::auth::route::login::ServerErr,
    //         >(
    //             &settings.site.address,
    //             PATH_API_LOGIN,
    //             None::<&str>,
    //             &controller::auth::route::login::Input {
    //                 email: email.clone(),
    //                 password: email,
    //             },
    //         )
    //         .await
    //         .unwrap();
    //
    //         trace!("res: {res:#?}");
    //     }
    //     ("api", "post") => {
    //         // let user = command3;
    //         // if user.len() < 1 {
    //         //     panic!("invalid user arg, example: api post user token rootdir");
    //         // }
    //
    //         let token = command3;
    //         if token.len() < 1 {
    //             panic!("invalid token arg, example: api post token rootdir");
    //         }
    //
    //         let rootdir = command4;
    //         if rootdir.len() < 1 {
    //             panic!("invalid rootdir arg, example: api post token rootdir");
    //         }
    //
    //         let mut responses: HashMap<String, Vec<u8>> = HashMap::new();
    //         let mut queue = std::collections::VecDeque::from([String::new()]);
    //
    //         while let Some(path) = queue.pop_front() {
    //             let dir_path = std::path::Path::new(&rootdir).join(&path);
    //             let mut dir = tokio::fs::read_dir(&dir_path).await.unwrap();
    //             while let Some(entry) = dir.next_entry().await.unwrap() {
    //                 let kind = entry.file_type().await.unwrap();
    //                 if kind.is_dir() {
    //                     let sub_assets_dir = std::path::Path::new(&path).join(entry.file_name());
    //                     let sub_assets_dir = sub_assets_dir.to_str().unwrap();
    //                     trace!("reading: {sub_assets_dir}");
    //                     queue.push_back(sub_assets_dir.to_string());
    //                 } else if kind.is_file() {
    //                     let name = entry.file_name();
    //                     let name = name.to_str().unwrap();
    //                     let Some(extension) = std::path::Path::new(name)
    //                         .extension()
    //                         .map(|v| v.to_str())
    //                         .flatten()
    //                     else {
    //                         continue;
    //                     };
    //
    //                     if !(extension == "jpg" || extension == "png" || extension == "webp") {
    //                         continue;
    //                     }
    //
    //                     let full_path = dir_path.join(name);
    //                     let mut files = Vec::new();
    //                     let data = fs::read(&full_path).await.unwrap();
    //                     files.push(data);
    //
    //                     let _ = send_native::<
    //                         controller::post::route::add::ServerOutput,
    //                         controller::post::route::add::ServerErr,
    //                     >(
    //                         &settings.site.address,
    //                         PATH_API_POST_ADD,
    //                         Some(token.clone()),
    //                         &controller::post::route::add::Input {
    //                             title: name.to_string(),
    //                             description: full_path.to_str().unwrap().to_string(),
    //                             files,
    //                         },
    //                     )
    //                     .await;
    //
    //                     trace!("{extension} {full_path:?}");
    //
    //                     // match get_asset(asset_path.to_str().unwrap(), extension).await {
    //                     //     Ok(asset) => {
    //                     //         let route = std::path::Path::new("/").join(&path).join(name);
    //                     //         responses.insert(route.to_str().unwrap().to_string(), asset);
    //                     //     }
    //                     //     Err(err) => {
    //                     //         debug!("getting asset err: {}", err);
    //                     //     }
    //                     // }
    //                 }
    //             }
    //         }
    //     }
    //     _ => panic!("{}", help),
    // }
    // trace!("{args:?}");
    //
    let command = Command::new("seed")
        .about("data seeder")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("user")
                .about("manage users")
                .arg(arg!(--"token" <TOKEN>).required(true))
                // .arg(arg!(--"name" <NAME>).required(true))
                .arg(arg!(--"count" <COUNT>).required(true))
                // .arg(arg!(<NAME>).last(true))
                .arg_required_else_help(true),
        );
    let matches = command.get_matches();
    match matches.subcommand() {
        Some(("user", sub_matches)) => {
            let token = sub_matches.get_one::<String>("token").unwrap();
            // let name = sub_matches.get_one::<String>("name").unwrap();
            let count = sub_matches.get_one::<String>("count").unwrap();
            let count = u32::from_str_radix(count, 10).unwrap();
            // trace!("is this working or no?: {name}");
            let path = "/tmp/img.png";
            let api = ApiNative::new("http://localhost:3000");
            let mut rng = rand::rng();

            for i in 0..count {
                // let mut image = RgbImage::new(200, 200);

                // let mut imgbuf = image::ImageBuffer::new(250, 250);
                // // Iterate over the coordinates and pixels of the image
                // for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
                //     let r = (0.3 * x as f32) as u8;
                //     let b = (0.3 * y as f32) as u8;
                //     *pixel = image::Rgb([r, i as u8, b]);
                // }

                let mut image = RgbImage::new(200, 200);
                let r = rng.random_range(200u8..255);
                let g = rng.random_range(200u8..255);
                let b = rng.random_range(200u8..255);
                for (x, y, pixel) in image.enumerate_pixels_mut() {
                    // let r = (0.3 * x as f32) as u8;
                    // let b = (0.3 * y as f32) as u8;
                    *pixel = image::Rgb([r, g, b]);
                }
                let height = 64.0;
                let scale = PxScale {
                    x: height * 2.0,
                    y: height,
                };
                let font = FontRef::try_from_slice(include_bytes!("/nix/store/5qc9yxwnfa074zrfsymmabhdbhg1rp7d-home-manager-path/share/fonts/noto/NotoSansMono[wdth,wght].ttf")).unwrap();
                let img = draw_text(
                    &mut image,
                    Rgb([0u8, 0u8, 0u8]),
                    // Rgb([255u8, 255u8, 255u8]),
                    0,
                    50,
                    scale,
                    &font,
                    &i.to_string(),
                );
                img.save(path).unwrap();

                // imgbuf.save(path).unwrap();
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

    //
    // // let time = get_timestamp();
    //
    //
    // let mut responses: HashMap<String, Vec<u8>> = HashMap::new();
    // let mut queue = std::collections::VecDeque::from([String::new()]);

    // while let Some(path) = queue.pop_front() {
    //     let dir_path = std::path::Path::new(&root_dir).join(&path);
    //     let mut dir = tokio::fs::read_dir(&dir_path).await.unwrap();
    //     while let Some(entry) = dir.next_entry().await.unwrap() {
    //         let kind = entry.file_type().await.unwrap();
    //         if kind.is_dir() {
    //             let sub_assets_dir = std::path::Path::new(&path).join(entry.file_name());
    //             let sub_assets_dir = sub_assets_dir.to_str().unwrap();
    //             trace!("reading: {sub_assets_dir}");
    //             queue.push_back(sub_assets_dir.to_string());
    //         } else if kind.is_file() {
    //             let name = entry.file_name();
    //             let name = name.to_str().unwrap();
    //             let Some(extension) = std::path::Path::new(name)
    //                 .extension()
    //                 .map(|v| v.to_str())
    //                 .flatten()
    //             else {
    //                 continue;
    //             };
    //
    //             let asset_path = dir_path.join(name);
    //
    //             trace!("{extension} {path}");
    //
    //             // match get_asset(asset_path.to_str().unwrap(), extension).await {
    //             //     Ok(asset) => {
    //             //         let route = std::path::Path::new("/").join(&path).join(name);
    //             //         responses.insert(route.to_str().unwrap().to_string(), asset);
    //             //     }
    //             //     Err(err) => {
    //             //         debug!("getting asset err: {}", err);
    //             //     }
    //             // }
    //         }
    //     }
    // }
}
