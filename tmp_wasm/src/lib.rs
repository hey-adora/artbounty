// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {

    artbounty::view::app::hook::api_post_comments::tests::comments_hook();
    // console_error_panic_hook::set_once();
    // logger::simple_web_logger_init();
    // tracing::debug!("yo wtf");
    // leptos::mount::hydrate_body(App);

    // let document = document();
    // let test_wrapper = document.create_element("section").unwrap();
    // let _ = document.body().unwrap().append_child(&test_wrapper);
    //
    // // start by rendering our counter and mounting it to the DOM
    // // note that we start at the initial value of 10
    // let _dispose = mount_to(
    //     test_wrapper.clone().unchecked_into(),
    //     || view! {  },
    // );

}
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
