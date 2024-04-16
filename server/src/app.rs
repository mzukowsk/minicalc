use futures::SinkExt;
use futures::StreamExt;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use crate::functions;
use crate::sheet::CellUpdateRequest;
use crate::sheet::FuncDef;
use crate::sheet::Sheet;

type AsyncResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const SERVER_ADDR: &str = "127.0.0.1:9123";

pub async fn run() -> AsyncResult<()> {
    let functions = functions::functions();

    let listener = TcpListener::bind(SERVER_ADDR).await?;

    println!("WebSocket server listening on {}", SERVER_ADDR);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, functions.clone()));
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, functions: HashMap<String, FuncDef>) -> AsyncResult<()> {
    let peer_addr = stream.peer_addr().map_or("unknown".to_string(), |addr| addr.to_string());

    let ws_stream = accept_async(stream).await?;
    let (mut sender, mut receiver) = ws_stream.split();

    println!("Connection from {} accepted", peer_addr);

    let mut sheet = Sheet::new(functions);

    while let Some(message) = receiver.next().await {
        let message = message?;
        if message.is_text() {
            println!("Received a message from {}", peer_addr);
            let message_text = message.into_text()?;
            let request: CellUpdateRequest = serde_json::from_str(&message_text)?;
            let response = sheet.set_cell_expression(request);
            let serialized_response = serde_json::to_string(&response)?;
            sender.send(Message::Text(serialized_response)).await?;
            sender.flush().await?;
        } else if message.is_close() {
            break;
        }
    }

    println!("Connection from {} closed", peer_addr);

    Ok(())
}
