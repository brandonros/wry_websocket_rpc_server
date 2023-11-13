use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use async_std::sync::Mutex;
use futures::prelude::*;
use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    future, pin_mut
};

use async_std::net::{TcpListener, TcpStream};
use async_tungstenite::tungstenite::protocol::Message;
use once_cell::sync::Lazy;

use crate::request_handler;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

static PEER_MAP: Lazy<PeerMap> = Lazy::new(|| {
    PeerMap::new(Mutex::new(HashMap::new()))
});

async fn handle_client(client_tcp_stream: TcpStream, addr: SocketAddr) {
    log::info!("Incoming TCP connection from: {}", addr);
    let client_ws_stream = async_tungstenite::accept_async(client_tcp_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    log::info!("WebSocket connection established: {}", addr);
    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    PEER_MAP.lock().await.insert(addr, tx.clone());
    let (client_sink, client_stream) = client_ws_stream.split();
    let client_sink = Arc::new(Mutex::new(client_sink));
    // rx from client
    let incoming_future = client_stream
        .try_filter(|msg| {
            // filter out closes
            future::ready(!msg.is_close())
        })
        .try_for_each(move |msg| {
            let tx = tx.clone();
            async move {
                match msg {
                    Message::Text(_) => todo!(),
                    Message::Binary(binary_msg) => {
                        request_handler::on_client_message(tx, binary_msg.clone()).await;
                        Ok(())
                    },
                    Message::Ping(_) => todo!(),
                    Message::Pong(_) => todo!(),
                    Message::Close(_) => todo!(),
                    Message::Frame(_) => todo!(),
                }
            }
        });
    // rx from channel, tx to client
    let outgoing_future = rx.for_each(move |msg| {
        let client_sink = client_sink.clone();
        async move {
            let mut client_sink = client_sink.lock().await;
            match client_sink.send(msg).await {
                Ok(_) => {},
                Err(_) => todo!(),
            }
        }
    });
    // TODO: why do we need to pin_mut?
    pin_mut!(
        outgoing_future, 
        incoming_future
    );
    // wait on both futures in parallel
    future::select(
        outgoing_future, 
        incoming_future
    ).await;
    // remove from map on disconnect
    log::info!("{} disconnected", &addr);
    PEER_MAP.lock().await.remove(&addr);
}

pub async fn start(addr: &str)
{
    // Create the event loop and TCP listener we'll accept connections on.
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    log::info!("Listening on: {}", addr);
    // Let's spawn the handling of each connection in a separate task.
    while let Ok((client_stream, addr)) = listener.accept().await {
        async_std::task::spawn(handle_client(client_stream, addr));
    }
}
