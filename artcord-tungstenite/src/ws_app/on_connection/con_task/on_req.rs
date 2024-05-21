use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::ClientThresholdMiddleware;
use artcord_state::model::ws_statistics::TempConIdType;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::task::TaskTracker;
use tracing::{debug, trace};

use crate::ws::{ws_statistic::WsStatsMsg, WsAppMsg};

use self::req_task::req_task;

use super::ConMsg;

pub mod req_task;

pub async fn on_req(
    result: Option<Result<Message, tokio_tungstenite::tungstenite::error::Error>>,
    // mut client_in: SplitStream<WebSocketStream<TcpStream>>,
    user_task_tracker: &TaskTracker,
    db: &Arc<DB>,
    connection_task_tx: &mpsc::Sender<ConMsg>,
    admin_ws_stats_tx: &mpsc::Sender<WsStatsMsg>,
    ws_app_tx: &mpsc::Sender<WsAppMsg>,
    connection_key: &TempConIdType,
    addr: &SocketAddr,
    ip: &IpAddr,
    get_threshold: &(impl ClientThresholdMiddleware + Send + Clone + Sync + 'static),
) -> bool {
    let Some(result) = result else {
        trace!("read.next() returned None");
        return true;
    };

    let client_msg = match result {
        Ok(result) => result,
        Err(err) => {
            debug!("recv msg error: {}", err);
            return false;
        }
    };

    // user_task_tracker.spawn(req_task(
    //     client_msg,
    //     db.clone(),
    //     connection_task_tx.clone(),
    //     admin_ws_stats_tx.clone(),
    //     ws_app_tx.clone(),
    //     connection_key.clone(),
    //     addr.clone(),
    //     ip.clone(),
    //     get_threshold.clone(),
    // ));

    false
}
