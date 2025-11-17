//use tauri::Emitter;
use pnet::packet::MutablePacket;
use pnet::packet::udp::MutableUdpPacket;

const END_SIG: u64 = 0xFFFFFFFFFFFFFFFF;
//const CHUNK_SIZE: usize = 1472 - 8 - 16 - 8 - 2 - 14;
const CHUNK_SIZE: u64 = 1024;

pub fn build_udp_packet<'a>(
    buffer: &'a mut [u8],
    src_port: u16,
    dst_port: u16,
    session_id: &[u8;16],
    chunk: &[u8;8],
    format_signal: &[u8;2],
    data_vec: &[u8;14],
    data: &[u8],
) -> MutableUdpPacket<'a> {
    let mut packet = MutableUdpPacket::new(buffer).expect("Failed to create UDP packet");
    packet.set_source(src_port);
    packet.set_destination(dst_port);
    // UDP length: header(8) + payload length
    packet.set_length((8 + session_id.len() + chunk.len() + format_signal.len() + data_vec.len() + data.len()) as u16);
    packet.set_checksum(0);
    let packet_payload = packet.payload_mut();

    let mut offset = 0;
    packet_payload[offset..offset + 16].copy_from_slice(session_id);
    offset += 16;

    packet_payload[offset..offset + 8].copy_from_slice(chunk);
    offset += 8;

    packet_payload[offset..offset + 2].copy_from_slice(format_signal);
    offset += 2;

    packet_payload[offset..offset + 14].copy_from_slice(data_vec);
    offset += 14;

    packet_payload[offset..offset + data.len()].copy_from_slice(data);

    packet
}

pub async fn send_text(
    dst_ip: String,
    dst_port: u16,
    text: String,
) -> Result<String, String> {
    use std::net::Ipv4Addr;
    use pnet::transport::{transport_channel, TransportChannelType::Layer4, TransportProtocol};
    use pnet::packet::ip::IpNextHeaderProtocols;
    use std::fs::File;
    use std::io::Read;

    let src_ip = "127.0.0.1";
    let src_port: u16 = 1234;

    let _src_ip: Ipv4Addr = src_ip.parse().map_err(|e| format!("Invalid src_ip: {}", e))?;
    let dst_ip: Ipv4Addr = dst_ip.parse().map_err(|e| format!("Invalid dst_ip: {}", e))?;

    let session_id = [0u8; 16];
    let format_signal = [0, 2];
    let data_vec = [5u8; 14];

    let protocol = TransportProtocol::Ipv4(IpNextHeaderProtocols::Udp);
    println!("create protocol");
    let (mut tx, _) = transport_channel(4096, Layer4(protocol))
        .map_err(|e| format!("Failed to create channel: {e}"))?;

    let mut chunk_id = 0u32;
    let data_chunks = text.as_bytes().chunks(CHUNK_SIZE as usize);

    for data_chunk in data_chunks {
        let chunk = (chunk_id as u64).to_be_bytes();
        let payload_len = session_id.len() + chunk.len() + format_signal.len() + data_vec.len() + data_chunk.len();
        let mut packet_buffer = vec![0u8; 8 + payload_len];

        let packet = build_udp_packet(
            &mut packet_buffer,
            src_port,
            dst_port,
            &session_id,
            &chunk,
            &format_signal,
            &data_vec,
            data_chunk,
        );

        if let Err(e) = tx.send_to(packet, std::net::IpAddr::V4(dst_ip)) {
            eprintln!("Error sending packet: {e}");
        }

        chunk_id += 1;
    }

    // 終了パケット送信
    let chunk = END_SIG.to_be_bytes();
    let mut end_packet_buffer = vec![0u8; 8 + 16 + 8 + 2 + 14];
    let end_packet = build_udp_packet(
        &mut end_packet_buffer,
        src_port,
        dst_port,
        &session_id,
        &chunk,
        &format_signal,
        &data_vec,
        &[],
    );
    let _ = tx.send_to(end_packet, std::net::IpAddr::V4(dst_ip));

    println!("text send");
    Ok(format!("Started sending text: {}", text))
}

