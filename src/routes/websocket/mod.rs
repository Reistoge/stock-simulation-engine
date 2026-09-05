use std::time::Duration;

use axum::{
    Router, extract::{
        Path, WebSocketUpgrade,
        ws::{Message, WebSocket},
    }, response::Response, routing::any,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

pub fn init() -> Router {
     Router::new().route("/ws/{*wildcard}", any(handler))
}
async fn handler(Path(wildcard): Path<String>, ws: WebSocketUpgrade) -> Response {
    // 3. Pass the variable into the socket handler using a move closure
    ws.on_upgrade(move |socket| handle_socket(socket, Path(wildcard)))
}

async fn handle_socket(socket: WebSocket, Path(wildcard): Path<String>) {
    println!("Connection Established with socket {:?}", wildcard);
    // 1. Split the socket into a sender and a receiver
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // 2. Create an MPSC channel to funnel outgoing messages
    let (tx, mut rx) = mpsc::channel::<Message>(32);

    // 3. Task: Forward messages from the MPSC channel to the WebSocket client
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Task: Background loop sending a message continuously
    let tx_background = tx.clone();
    let mut var1: u64 = 0;
    tokio::spawn(async move {
        loop {
            var1 = var1 + 1;
            if tx_background
                .send(Message::Text(var1.to_string().into()))
                .await
                .is_err()
            {
                break; // Stop if the receiver drops
            }
            // A sleep is required to prevent CPU locking in an infinite loop

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    // Main Loop: Receive messages from client and respond
    while let Some(Ok(msg)) = ws_receiver.next().await {
        println!("{:?}", msg.to_text().ok());
        let message = "Hiii";
        if tx.send(Message::Text(message.into())).await.is_err() {
            break; // Client disconnected
        }
    }
}
