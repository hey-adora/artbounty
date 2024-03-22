use artcord_state::aggregation::server_msg_img::AggImg;
use artcord_state::misc::img_quality::ImgQuality;
use artcord_state::model::user::User;
use chrono::Utc;
use leptos::*;
use leptos::{window, RwSignal, SignalGetUntracked};
use std::fmt::Debug;
use wasm_bindgen::JsValue;
use web_sys::Location;

// pub mod server_msg_wrap;
// pub mod client_msg_wrap;
// pub mod ws_runtime;

// use crate::bot::img_quality::ImgQuality;
// use crate::database::models::user::User;
// use crate::message::server_msg_img::AggImg;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LoadingNotFound {
    NotLoaded,
    Loading,
    Loaded,
    NotFound,
    Error,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScrollSection {
    Home,
    About,
    Gallery,
    UserProfile,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ServerMsgImgResized {
    pub id: String,
    // pub id: u128,
    pub quality: ImgQuality,
    pub display_high: String,
    pub display_preview: String,
    pub user: User,
    pub user_id: String,
    pub org_hash: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub new_width: RwSignal<f32>,
    pub new_height: RwSignal<f32>,
    pub top: RwSignal<f32>,
    pub left: RwSignal<f32>,
    pub has_high: bool,
    pub has_medium: bool,
    pub has_low: bool,
    pub modified_at: i64,
    pub created_at: i64,
}

// Hi

impl Default for ServerMsgImgResized {
    fn default() -> Self {
        Self {
            id: String::from("1177244237021073450"),
            quality: ImgQuality::Org,
            display_preview: String::from(
                "/assets/gallery/org_2552bd2db66978a9b3675721e95d1cbd.png",
            ),
            display_high: String::from("/assets/gallery/org_2552bd2db66978a9b3675721e95d1cbd.png"),
            user: User {
                id: String::from("id"),
                guild_id: String::from("1159766826620817419"),
                name: String::from("name"),
                pfp_hash: Some(String::from("pfp_hash")),
                modified_at: Utc::now().timestamp_millis(),
                created_at: Utc::now().timestamp_millis(),
            },
            user_id: String::from("1159037321283375174"),
            org_hash: String::from("2552bd2db66978a9b3675721e95d1cbd"),
            format: String::from("png"),
            width: 233,
            height: 161,
            new_width: RwSignal::new(233.0),
            new_height: RwSignal::new(161.0),
            top: RwSignal::new(0.0),
            left: RwSignal::new(0.0),
            has_high: false,
            has_medium: false,
            has_low: false,
            modified_at: Utc::now().timestamp_millis(),
            created_at: Utc::now().timestamp_millis(),
        }
    }
}

impl GalleryImg for ServerMsgImgResized {
 
    fn set_pos(&mut self, left: f32, top: f32, new_width: f32, new_height: f32) {
        self.left.set(left);
        self.top.set(top);
        self.new_width.set(new_width);
        self.new_height.set(new_height);
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn get_pos(&self) -> (f32, f32) {
        (self.left.get_untracked(), self.top.get_untracked())
    }
}

impl From<AggImg> for ServerMsgImgResized {
    fn from(value: AggImg) -> Self {
        let quality = value.pick_quality();
        let display_preview = quality.gen_link_preview(&value.org_hash, &value.format);
        Self {
            id: value.id,
            quality,
            display_preview,
            // id: rand::thread_rng().gen::<u128>(),
            display_high: ImgQuality::gen_link_org(&value.org_hash, &value.format),
            user: value.user,
            new_width: RwSignal::new(value.width as f32),
            new_height: RwSignal::new(value.height as f32),
            top: RwSignal::new(0.0),
            left: RwSignal::new(0.0),
            user_id: value.user_id,
            org_hash: value.org_hash,
            format: value.format,
            width: value.width,
            height: value.height,
            has_high: value.has_high,
            has_medium: value.has_medium,
            has_low: value.has_low,
            modified_at: value.modified_at,
            created_at: value.created_at,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PageProfileState {
    // pub not_found: RwSignal<bool>,
    pub user: RwSignal<Option<User>>,
    pub gallery_imgs: RwSignal<Vec<ServerMsgImgResized>>,
    pub gallery_loaded: RwSignal<LoadingNotFound>,
}

impl PageProfileState {
    pub fn new() -> Self {
        Self {
            user: RwSignal::new(None),
            gallery_imgs: RwSignal::new(Vec::new()),
            gallery_loaded: RwSignal::new(LoadingNotFound::NotLoaded),
        }
    }
}

pub fn get_window_path() -> String {
    let location: Location = window().location();
    let path: Result<String, JsValue> = location.pathname();
    let hash: Result<String, JsValue> = location.hash();
    if let (Ok(path), Ok(hash)) = (path, hash) {
        format!("{}{}", path, hash)
    } else {
        String::from("/")
    }
}

#[derive(Clone)]
pub struct SelectedImg {
    pub org_url: String,
    pub author_name: String,
    pub author_pfp: String,
    pub author_id: String,
    pub width: u32,
    pub height: u32,
}

pub const NEW_IMG_HEIGHT: u32 = 250;

pub trait GalleryImg {
    fn get_size(&self) -> (u32, u32);
    fn get_pos(&self) -> (f32, f32);
    fn set_pos(&mut self, left: f32, top: f32, new_width: f32, new_height: f32);
}

pub fn resize_img<T: GalleryImg + Debug>(
    top: &mut f32,
    max_width: u32,
    new_row_start: usize,
    new_row_end: usize,
    imgs: &mut [T],
) {
    let mut total_ratio: f32 = 0f32;

    for i in new_row_start..(new_row_end + 1) {
        let (width, height) = imgs[i].get_size();
        total_ratio += width as f32 / height as f32;
    }
    let optimal_height: f32 = max_width as f32 / total_ratio;
    let mut left: f32 = 0.0;

    for i in new_row_start..(new_row_end + 1) {
        let (width, height) = imgs[i].get_size();
        let new_width = optimal_height * (width as f32 / height as f32);
        let new_height = optimal_height;
        imgs[i].set_pos(left, *top, new_width, new_height);
        left += new_width;
    }
    *top += optimal_height;
}

pub fn resize_img2<T: GalleryImg + Debug>(
    top: &mut f32,
    max_width: u32,
    new_row_start: usize,
    new_row_end: usize,
    imgs: &mut [T],
) {
    let mut optimal_count =
        (max_width as i32 / NEW_IMG_HEIGHT as i32) - (new_row_end - new_row_start) as i32;
    if optimal_count < 0 {
        optimal_count = 0;
    }
    let mut total_ratio: f32 = optimal_count as f32;
    if max_width < NEW_IMG_HEIGHT * 3 {
        total_ratio = 0.0;
    }

    for i in new_row_start..(new_row_end + 1) {
        let (width, height) = imgs[i].get_size();
        total_ratio += width as f32 / height as f32;
    }
    let optimal_height: f32 = max_width as f32 / total_ratio;
    let mut left: f32 = 0.0;

    for i in new_row_start..(new_row_end + 1) {
        let (width, height) = imgs[i].get_size();
        let new_width = optimal_height * (width as f32 / height as f32);
        let new_height = optimal_height;
        imgs[i].set_pos(left, *top, new_width, new_height);
        left += new_width;
    }

    *top += optimal_height;
}

pub fn resize_imgs<T: GalleryImg + Debug>(new_height: u32, max_width: u32, imgs: &mut [T]) -> () {
    let loop_start = 0;
    let loop_end = imgs.len();
    let mut new_row_start: usize = 0;
    let mut new_row_end: usize = if loop_end > 0 { loop_end - 1 } else { 0 };
    let mut current_row_filled_width: u32 = 0;
    let mut top: f32 = 0.0;

    for index in loop_start..loop_end {
        let org_img = &mut imgs[index];
        let (width, height) = org_img.get_size();
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: u32 = if height < new_height {
            0
        } else {
            height - new_height
        };
        let new_width: u32 = width - (height_diff as f32 * ratio) as u32;
        if (current_row_filled_width + new_width) <= max_width {
            current_row_filled_width += new_width;
            new_row_end = index;
            if index == loop_end - 1 {
                resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
        } else {
            if index != 0 {
                resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
            new_row_start = index;
            new_row_end = index;
            current_row_filled_width = new_width;
            if index == loop_end - 1 {
                resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
        }
    }
}

pub fn calc_fit_count(width: u32, height: u32) -> u32 {
    (width * height) / (NEW_IMG_HEIGHT * NEW_IMG_HEIGHT)
}

pub fn create_client_test_imgs() -> Vec<ServerMsgImgResized> {
    let mut new_imgs: Vec<ServerMsgImgResized> = Vec::new();
    for _ in 0..25 {
        new_imgs.push(ServerMsgImgResized::default());
    }
    new_imgs
}
