async fn handle_websocket_connection(app_handle: AppHandle, stream: tokio::net::TcpStream, peer_addr: std::net::SocketAddr) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake from {}: {}", peer_addr, e);
            let _ = app_handle.emit("server_status", format!("WebSocket handshake failed from {}: {}", peer_addr, e));
            return;
        }
    };
    println!("WebSocket connection established with: {}", peer_addr);

    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                // JSONでリクエスト解析
                if let Ok(request) = serde_json::from_str::<serde_json::Value>(&text) {
                    match request["type"].as_str() {
                        Some("send_file") => {
                            // ファイル送信リクエスト
                            if let Some(file_path) = request["filePath"].as_str() {
                                match std::fs::read(file_path) {
                                    Ok(file_content) => {
                                        let encoded_data = base64::encode(file_content);
                                        let response = serde_json::json!({
                                            "type": "file_data",
                                            "fileName": file_path.split('/').last().unwrap(),
                                            "data": encoded_data
                                        });
                                        write.send(tokio_tungstenite::tungstenite::Message::Text(response.to_string())).await.unwrap();
                                    }
                                    Err(err) => {
                                        let error_response = serde_json::json!({
                                            "type": "error",
                                            "message": format!("Failed to read file: {}", err)
                                        });
                                        write.send(tokio_tungstenite::tungstenite::Message::Text(error_response.to_string())).await.unwrap();
                                    }
                                }
                            }
                        }
                        Some("receive_file") => {
                            // ファイル受信リクエスト
                            if let Some(encoded_data) = request["data"].as_str() {
                                if let Some(file_name) = request["fileName"].as_str() {
                                    let decoded_data = base64::decode(encoded_data).unwrap();
                                    let save_path = format!("received/{}", file_name);
                                    match std::fs::write(&save_path, decoded_data) {
                                        Ok(_) => {
                                            let success_response = serde_json::json!({
                                                "type": "success",
                                                "message": format!("File saved at {}", save_path)
                                            });
                                            write.send(tokio_tungstenite::tungstenite::Message::Text(success_response.to_string())).await.unwrap();
                                        }
                                        Err(err) => {
                                            let error_response = serde_json::json!({
                                                "type": "error",
                                                "message": format!("Failed to save file: {}", err)
                                            });
                                            write.send(tokio_tungstenite::tungstenite::Message::Text(error_response.to_string())).await.unwrap();
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            println!("Unknown request type");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving message from {}: {}", peer_addr, e);
                break;
            }
            _ => {}
        }
    }
    println!("WebSocket connection to {} closed.", peer_addr);
}

