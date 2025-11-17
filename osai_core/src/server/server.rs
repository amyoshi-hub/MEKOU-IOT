use tokio::net::UdpSocket;
use tokio::task;
use std::sync::Arc;
use crate::server::server_signal;
use crate::server::format_handler::process_format;

fn parse_packet(payload: &[u8]) -> Option<([u8; 16], [u8; 8], [u8; 2], [u8; 14], Vec<u8>)> {
    if payload.len() < 40 {
        return None;
    }
    let session_id = payload[0..16].try_into().ok()?;
    let chunk = payload[16..24].try_into().ok()?;
    let format = payload[24..26].try_into().ok()?;
    let data_vec = payload[26..40].try_into().ok()?;
    let data = payload[40..].to_vec();

    Some((session_id, chunk, format, data_vec, data))
}

async fn server_launch_signal(port: &str) -> Result<(), String> {
    let mut send_buf = [0u8; 1024];
    let local_port = port.parse::<u16>().map_err(|e| format!("Invalid port: {}", e))?;

    let packet_len = server_signal::build_server_announce_packet(
        &mut send_buf,
        local_port,
        local_port,
    );

    let send_socket = UdpSocket::bind("0.0.0.0:0").await
        .map_err(|e| format!("Failed to bind signal socket: {}", e))?;

    let broadcast_addr = format!("255.255.255.255:{}", local_port);
    send_socket.set_broadcast(true)
        .map_err(|e| format!("Failed to enable broadcast: {}", e))?;

    send_socket.send_to(&send_buf[..packet_len], &broadcast_addr).await
        .map_err(|e| format!("Failed to send announce: {}", e))?;

    println!("Signal sent to {}", broadcast_addr);
    Ok(())
}

pub async fn start_server(port: String) -> Result<String, String> {
    let address = format!("0.0.0.0:{}", port);
    println!("Binding UDP server to: {}", address);

    let socket = Arc::new(
    tokio::net::UdpSocket::bind(&address)
        .await
        .map_err(|e| e.to_string())?
    );
    println!("UDP Server bound to: {}", address);

    let port_for_signal = port.clone();
    task::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            if let Err(e) = server_launch_signal(&port_for_signal).await {
                eprintln!("Signal error: {}", e);
            }
        }
    });

    let socket_for_recv = Arc::clone(&socket);
    loop {
        let mut buf = [0u8; 1024];
        match socket_for_recv.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                if let Some((session_id, chunk, format, data_vec, data_payload)) = parse_packet(&buf[..len]) {
                    let _ = process_format(format, session_id, chunk, data_vec, data_payload, addr, port.clone());
                } else {
                    eprintln!("Parse error");
                }
            }
            Err(e) => {
                eprintln!("Recv error: {}", e);
                break;
            }
        }
    }

    Ok(format!("Server started on {}", address))
}
