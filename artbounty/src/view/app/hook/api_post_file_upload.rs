use crate::view::toolbox::prelude::*;
use leptos::{ev::EventDescriptor, prelude::*, task::spawn_local};
use std::fmt::Debug;
use tracing::{error, trace};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{Array, Function, JsString, Number, Promise, Reflect};
use web_sys::{
    Blob, Element, File, FormData, HtmlElement, HtmlInputElement, HtmlTextAreaElement,
    MutationObserver, MutationRecord, ProgressEvent, RequestInit, RequestMode, SubmitEvent,
    TransformStream, TransformStreamDefaultController, Transformer, XmlHttpRequest,
};

#[derive(Clone, Copy)]
pub struct FileUpload {
    pub post_files: RwSignal<Vec<PostFile>, LocalStorage>,
}

#[derive(Clone)]
pub struct PostFile {
    pub state: UploadProgressState,
    pub file: File,
    pub file_name: String,
    pub completed_bytes: usize,
    pub total_size_bytes: usize,
}

#[derive(Clone)]
pub enum UploadProgressState {
    Selected,
    Uploading,
    Completed,
}

impl PostFile {
    pub fn new(file: File) -> Self {
        let file_name = file.name();
        let total_size_bytes = file.size() as usize;
        Self {
            state: UploadProgressState::Selected,
            file,
            file_name,
            completed_bytes: 0,
            total_size_bytes,
        }
    }
}

impl PostFile {
    pub fn get_upload_percentage(&self) -> u8 {
        trace!(
            "get_upload_percentage {} {}",
            self.completed_bytes, self.total_size_bytes
        );
        percentage(self.completed_bytes, self.total_size_bytes)
        // if self.completed_bytes == 0 || self.total_size_bytes == 0 {
        //     return 0;
        // }
        // ((self.total_size_bytes as f64 / self.completed_bytes as f64) * 100.0) as u8
    }
    //
}

pub fn percentage(completed: usize, total: usize) -> u8 {
    if completed == 0 || total == 0 {
        return 0;
    }
    (100.0 / (total as f64 / completed as f64)) as u8
}

// impl UploadProgress {
//     pub fn is_name_eq(&self, name: impl AsRef<str>) -> bool {
//         let name = name.as_ref();
//         match self {
//             UploadProgress::Selected { file_name, .. } => file_name == name,
//             UploadProgress::Uploading { file_name, .. } => file_name == name,
//             UploadProgress::Completed { file_name, .. } => file_name == name,
//         }
//     }
//
//     pub fn name(&self) -> &str {
//         match self {
//             UploadProgress::Selected { file_name, .. } => file_name,
//             UploadProgress::Uploading { file_name, .. } => file_name,
//             UploadProgress::Completed { file_name, .. } => file_name,
//         }
//     }
// }

impl FileUpload {
    pub fn new() -> Self {
        Self {
            post_files: RwSignal::new_local(Vec::new()),
        }
    }

    // fn add_progress(&self, post_file: PostFile) {
    //     self.progress.update(|v| {
    //         let Some(pos) = v.iter().position(|v| v.file_name == progress.file_name) else {
    //             v.push(progress);
    //             return;
    //         };
    //         v[pos] = progress;
    //     });
    // }

    pub fn clear(&self) {
        self.post_files.update(|v| {
            v.clear();
        });
    }

    // pub fn select(&self, files: &[File]) {
    //     if files.is_empty() {
    //         return;
    //     }

    //     self.post_files.update(|v| {
    //         v.clear();
    //         for file in files {
    //             v.push(PostFile::new(file.clone()));
    //             // self.add_progress(UploadProgress::new(file, UploadProgressState::Selected));
    //         }
    //     });
    //     // self.progress.set(UploadProgress::Selected {
    //     //     file_name: file.name(),
    //     //     total_size_bytes: file.size() as usize,
    //     // });
    // }

    pub fn upload(&self, files: &[File]) {
        // let progress = self.progress;
        // self.add_progress(UploadProgress::Uploading {
        //     file_name: file.name(),
        //     completed_bytes: 0,
        //     total_size_bytes: file.size() as usize,
        // });
        // progress.set(UploadProgress::Uploading {
        //     file_name: file.name(),
        //     completed_bytes: 0,
        //     total_size_bytes: file.size() as usize,
        // });
        // let post_files = ;
        //
        self.post_files.update(|current_files| {
            for file in files {
                current_files.push(PostFile::new(file.clone()));
            }
        });
        // for file in files {
        //     v.push(PostFile::new(file.clone()));
        //     // self.add_progress(UploadProgress::new(file, UploadProgressState::Selected));
        // }
        let post_files = self.post_files.clone();

        for (index, post_file) in post_files.get_untracked().iter().enumerate() {
            let file = post_file.file.clone();
            spawn_local(async move {
                let result = JsFuture::from(Promise::new(
                    &mut move |resolve: Function, reject: Function| {
                        let req = XmlHttpRequest::new().unwrap();
                        let req_upload = req.upload().unwrap();

                        req_upload
                            .add_event_listener_with_callback(
                                "progress",
                                &Closure::<dyn FnMut(_)>::new(move |event: ProgressEvent| {
                                    post_files.update(|v| {
                                        let Some(file) = v.get_mut(index) else {
                                            return;
                                        };
                                        file.completed_bytes = event.loaded() as usize;
                                    });
                                    trace!("uploading... {}/{}", event.loaded(), event.total());
                                    //
                                })
                                .into_js_value()
                                .unchecked_into(),
                            )
                            .unwrap();

                        req.add_event_listener_with_callback(
                            "progress",
                            &Closure::<dyn FnMut(_)>::new(move |event: ProgressEvent| {
                                trace!("downloading... {}/{}", event.loaded(), event.total());
                                //
                            })
                            .into_js_value()
                            .unchecked_into(),
                        )
                        .unwrap();

                        req.add_event_listener_with_callback(
                            "loaded",
                            &Closure::<dyn FnMut()>::new(move || {
                                trace!("complete");
                                resolve.call1(&JsValue::NULL, &"done".into()).unwrap();
                            })
                            .into_js_value()
                            .unchecked_into(),
                        )
                        .unwrap();

                        let form = FormData::new().unwrap();
                        form.set_with_blob("upload", file.unchecked_ref()).unwrap();
                        // form.set_with_str("what", "nooooooooo").unwrap();

                        req.open_with_async(
                            "POST",
                            "http://localhost:3000/api/post/oqmp4g1iiswwp8nixrae/add_file",
                            true,
                        )
                        .unwrap();

                        // req.set_request_header("Content-Type", "application/x-www-form-urlencoded")
                        //     .unwrap();

                        // req.set_request_header("Content-Type", "multipart/form-data")
                        //     .unwrap();

                        // req.set_request_header("Content-Type", "application/octet-stream")
                        //     .unwrap();
                        // req.send_with_opt_str(Some("hello")).unwrap();
                        // req.send_with_opt_blob(Some(&file)).unwrap();

                        req.send_with_opt_form_data(Some(&form)).unwrap();
                    },
                ))
                .await
                .unwrap()
                .as_string()
                .unwrap();
            });
        }
    }
    //
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum UploadErr {
    #[error("wrong credentials")]
    Exploded,
}

#[cfg(test)]
mod tests {
    use crate::view::app::hook::api_post_file_upload::percentage;

    #[test]
    fn percent_calc() {
        let percent = percentage(10, 20);
        assert_eq!(percent, 50);
    }
}
