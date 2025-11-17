//use tauri::{AppHandle};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
//use tauri::Emitter;

// WebSocketサーバーをバックグラウンドで開始するTauriコマンド
pub async fn start_websocket_server(ip: String, port: String) -> Result<String, String> {
    let addr = format!("{}:{}", ip, port);
    println!("Attempting to start WebSocket server at: {}", addr);

    // Clone 'addr' here so that a copy can be moved into the spawned task,
    // while the original 'addr' remains available for the `Ok` return.
    let addr_clone_for_spawn = addr.clone(); // <--- Add this line

    // WebSocketサーバーをバックグラウンドで実行するためのtokio::spawn
    tokio::spawn(async move {
        // Use the cloned version 'addr_clone_for_spawn' inside this block
        let listener = match TcpListener::bind(&addr_clone_for_spawn).await { // <-- Use addr_clone_for_spawn
            Ok(l) => l,
            Err(e) => {
                let error_msg = format!("Failed to bind WebSocket server to {}: {}", addr_clone_for_spawn, e); // <-- Use addr_clone_for_spawn
                eprintln!("{}", error_msg);
                // フロントエンドへのエラー通知
                return;
            }
        };
        println!("WebSocket server listening on: {}", addr_clone_for_spawn); // <-- Use addr_clone_for_spawn

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    println!("New WebSocket connection from: {}", peer_addr);

                    // 各接続を非同期タスクで処理
                    tokio::spawn(handle_websocket_connection(stream, peer_addr));
                }
                Err(e) => {
                    eprintln!("Error accepting WebSocket connection: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    });

    // The original 'addr' is still available here
    Ok(format!("Attempting to start WebSocket server on {}", addr)) // <-- Use the original 'addr'
}

async fn handle_websocket_connection(stream: tokio::net::TcpStream, peer_addr: std::net::SocketAddr) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake from {}: {}", peer_addr, e);
            return;
        }
    };
    println!("WebSocket connection established with: {}", peer_addr);

    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                match msg {
                    tokio_tungstenite::tungstenite::Message::Text(text) => {
                        println!("Received text message from {}: {}", peer_addr, text);

                        // Ensure 'into()' is used for String to Message conversion if it's not implicit
                        if let Err(e) = write.send(tokio_tungstenite::tungstenite::Message::Text(format!("Echo: {}", text).into())).await {
                             // Note: format! returns a String, which can usually be implicitly converted to Message::Text
                             // but if you have a specific error, adding .into() can sometimes help clarify.
                             // However, the original error was unrelated to this line.
                            eprintln!("Error sending echo to {}: {}", peer_addr, e);
                            break;
                        }
                    },
                    tokio_tungstenite::tungstenite::Message::Binary(bin) => {
                        println!("Received binary message from {}: len={}", peer_addr, bin.len());
                    },
                    tokio_tungstenite::tungstenite::Message::Ping(_ping_data) => {
                        println!("Received Ping from {}", peer_addr);
                    },
                    tokio_tungstenite::tungstenite::Message::Pong(_) => {
                        println!("Received Pong from {}", peer_addr);
                    },
                    tokio_tungstenite::tungstenite::Message::Close(close_frame) => {
                        println!("Connection closed by {} with frame: {:?}", peer_addr, close_frame);
                        break;
                    },
                    tokio_tungstenite::tungstenite::Message::Frame(_) => {
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving message from {}: {}", peer_addr, e);
                break;
            }
        }
    }
    println!("WebSocket connection to {} closed.", peer_addr);
}
