use leptos::*;
use artcord_state::global;

use crate::app::{global_state::GlobalState, hooks::use_ws_live_stats::WebStatPathType};
use crate::app::hooks::use_ws_live_stats::use_ws_live_stats;

use super::WsPathTableHeaderView;
use strum::{EnumCount, IntoEnumIterator};

#[component]
pub fn WsLive() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let ws = global_state.ws;
    
    let page = global_state.pages.admin;
    
    let live_stats = page.live_connections;
    //use_ws_live_stats(ws, live_stats);

    

    let live_connection_count_view = move |count: WebStatPathType| {
        (0..global::ClientMsg::COUNT)
            .map(|path| {
                let count = count.get(&path).cloned();
                view! {
                    <th class="whitespace-nowrap">//{
                        <div class="flex justify-center gap-2 whitespace-nowrap">
                            {
                                match count {
                                    Some(stats) => {
                                        view! {
                                            <span>{ move || stats.total_allowed.get() }</span>
                                            <span>{ move || stats.total_blocked.get() }</span>
                                            <span>{ move || stats.total_banned.get() }</span>
                                            <span>{ move || stats.total_already_banned.get() }</span>
                                        }
                                    }
                                    None => {
                                        view! {
                                            <span>{ "0" }</span>
                                            <span>{ "0" }</span>
                                            <span>{ "0" }</span>
                                            <span>{ "0" }</span>
                                        }
                                    }
                                }
                            }
                        </div>
                        
                    //     move || count.map(|count| view! { 
                    //     <span>{ move || "count.total_allowed.get().to_string()" }</span>
                    //     <span>{ move || "count.total_blocked.get().to_string()" }</span>
                    //     <span>{ move || "count.total_banned.get().to_string()" }</span>
                    //  } ).unwrap_or( view! {
                    //     <span>"0"</span>
                    //     <span>"0"</span>
                    //     <span>"0"</span>  })}
                    </th>
                }
            })
            .collect_view()
    };

    view! {
        <div class="grid grid-rows-[auto_1fr] overflow-y-hidden">
            <div>"Live WebSocket Connections"</div>
            <div class="overflow-y-scroll ">
                <table class="text-center">
                    <tr class="sticky top-0 left-0 bg-mid-purple ">
                        <th class="whitespace-nowrap">"ip"</th>
                        <th class="whitespace-nowrap">"ip ban"</th>
                        <WsPathTableHeaderView/>
                    </tr>
                    <For each=move || live_stats.stats.get().into_iter() key=|item| item.0.clone() let:item>
                        <tr>
                            <td class="whitespace-nowrap">{item.1.addr}</td>
                            <td class="whitespace-nowrap">{move || {
                                match item.1.banned_until.get() {
                                    Some((date, reason)) => format!("{:?} - {}", reason, date),
                                    None => "None".to_string(),
                                }
                            }}</td>
                            { move || live_connection_count_view(item.1.paths.get()) }
                        </tr>
                    </For>
                </table>

            </div>
        </div>
    }
}
