//use tauri::{AppHandle, Emitter};
use std::net::SocketAddr;
use crate::ai::hebbian_local::ai;
use crate::ai::state::{MY_VEC, W1, W2, SERVER_LIST, ServerInfo};

use crate::fileIO::create_lyric::create_lyric;

pub fn process_format(
    format: [u8; 2],
    session_id: [u8; 16],
    _chunk: [u8; 8],
    data_vec: [u8; 14],
    data_payload: Vec<u8>,
    addr: SocketAddr,
    port: String,
) -> String {
    println!("format:{:x?}", format);
    println!("data_vec:{:x?}", data_vec);

    //ほんとはSIMDでやりたい
    //こういう記法はむしろIOTで扱えるフォーマットを絞ってやる
    let received_data_display = match format {
        [0, 0] => {
            String::from_utf8_lossy(&data_payload).to_string()
        }
        [0, 1] => {

            println!("  Discovered Server IP: {}", addr);
            hex::encode(&data_payload)
        }
        [0, 2] => {
            println!("receive data");
            let input_vec = data_vec;
            let mut my_vec_guard = MY_VEC.lock().unwrap();
            let mut w1_guard = W1.lock().unwrap();
            let mut w2_guard = W2.lock().unwrap(); 

            let (new_vec, is_trusted) = ai(*my_vec_guard, input_vec, &mut *w1_guard, &mut *w2_guard);
            if is_trusted {
                *my_vec_guard = new_vec;
                //app_handle.emit("trusted_node", "学習データを更新しました").unwrap();
            } else {
                //app_handle.emit("enemy_detected", "敵シグナルを検知").unwrap();
            }

            //vocaloid logic
            let text = String::from_utf8_lossy(&data_payload).to_string();
            if input_vec[0] > 4 {
                create_lyric(&text, input_vec);
                println!("lyric created");
            }else{
                println!("lyric didnot created");
            }
            format!("AI check result: trusted={}", is_trusted)
        }

        [0xFF, 0xFF] => {
            println!("--- Signal ---");
            println!("  IP: {} PORT: {}", addr, port);
            println!("  Session ID: {:x?}", session_id);
            println!("  Data Vec: {:x?}", data_vec);

            let ip_str = addr.ip().to_string();
            let port_num = port.parse::<u16>().unwrap_or(0);

            let server_info = ServerInfo {
                addr: ip_str,
                port: port_num,
            };

            let mut list = SERVER_LIST.lock().unwrap();
            if !list.contains(&server_info) {
                list.insert(server_info.clone());
                println!("not duplicate server:{}", server_info.addr); 
            }else{
                println!("dup server:{}:{}", server_info.addr, port);
            }

            if let Ok(msg) = String::from_utf8(data_payload) {
                format!("Server Discovered: {} Msg({})", addr, msg)
            } else {
                format!("Server Discovered: {}", addr)
            }
        }

        _ => {
            format!("unsupported format: {:x?}", format)
        }
    };

    println!("data:{}", received_data_display);
    received_data_display
}

