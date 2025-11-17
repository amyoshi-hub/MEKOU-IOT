use tauri::command;
use tauri::Emitter;
use tauri::{AppHandle, Manager};

use crate::AI::state::{SERVER_LIST, ServerInfo};

#[tauri::command]
pub fn request_server_list(app: AppHandle) {
    let list = SERVER_LIST.lock().unwrap();

    for server in list.iter() {
       if let Err(e) = app.emit("add_server", server) {
            eprintln!("Failed to emit server list");
       }

    }
}
