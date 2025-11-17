//use tauri::Emitter;
use pnet::packet::MutablePacket;
use pnet::packet::udp::MutableUdpPacket;
//use tokio::task;

const END_SIG: u64 = 0xFFFFFFFFFFFFFFFF;
//const CHUNK_SIZE: usize = 1472 - 8 - 16 - 8 - 2 - 14;

fn build_udp_packet<'a>(
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

pub async fn send_text() -> Result<String, String> {
    use std::net::Ipv4Addr;
    use pnet::transport::{transport_channel, TransportChannelType::Layer4, TransportProtocol};
    use pnet::packet::ip::IpNextHeaderProtocols;
    use tokio::task::spawn;
    use std::fs::File;
    use std::io::Read;

    let src_ip = "127.0.0.1";
    let src_port: u16 = 1234;
    let dst_ip = "127.0.0.1";
    let dst_port: u16 = 1234;
    let filename = "ex.txt";

    let _src_ip: Ipv4Addr = src_ip.parse().map_err(|e| format!("Invalid src_ip: {}", e))?;
    let dst_ip: Ipv4Addr = dst_ip.parse().map_err(|e| format!("Invalid dst_ip: {}", e))?;

    // ここで固定値や適当な値を用意
    let session_id = [0u8; 16];       // 本当はランダム等にする
    let format_signal = [0u8; 2];     // 適宜設定
    let data_vec = [0u8; 14];         // 適宜設定

    // filenameをcloneしてmoveに持ち込む
    let filename_clone = filename;

    //本当はasyncのほうが良いのかもしれないが
    spawn( async move {
        let protocol = TransportProtocol::Ipv4(IpNextHeaderProtocols::Udp);
        println!("create protocol");
        let (mut tx, _) = match transport_channel(4096, Layer4(protocol)) {
            Ok((tx, rx)) => {
                (tx, rx)
            }
            Err(_e) => {
                println!("thread error");
                return;
            }
        };

        let mut file = match File::open(&filename_clone) {
            Ok(f) => f,
            Err(_e) => {
                return;
            }
        };

        let mut buffer = [0u8; 1024];
        let mut chunk_id = 0u32;

        loop {
            let read_bytes = match file.read(&mut buffer) {
                Ok(n) => n,
                Err(_e) => {
                    return;
                }
            };

            if read_bytes == 0 {
                break;
            }

            // chunkを8バイトの配列に格納（例としてu64をBEで）
            let chunk = (chunk_id as u64).to_be_bytes();

            // 送信データ（ファイルの一部）
            let data = &buffer[..read_bytes];

            // UDPペイロードサイズ計算
            let payload_len = 16 + 8 + 2 + 14 + data.len();
            let mut packet_buffer = vec![0u8; 8 + payload_len];  // UDPヘッダー8バイト + payload

            let mut packet = build_udp_packet(
                &mut packet_buffer,
                src_port,
                dst_port,
                &session_id,
                &chunk,
                &format_signal,
                &data_vec,
                data,
            );

            println!("Sending packet: {:?}", packet);
            println!("To destination: {}", dst_ip);


            if let Err(_e) = tx.send_to(packet, std::net::IpAddr::V4(dst_ip)) {
                return;
            }

            chunk_id += 1;
        }

        // 終了パケット送信（chunk=END_SIG、dataなし）
        let chunk = END_SIG.to_be_bytes();
        let mut end_packet_buffer = vec![0u8; 8 + 16 + 8 + 2 + 14];
        let mut end_packet = build_udp_packet(
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
    });

    Ok(format!("Started sending file: {}", filename))
}
