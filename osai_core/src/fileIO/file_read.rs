use std::fs;
//use std::path::Path;

#[tauri::command]
pub fn read_file_content(file_path: String) -> Result<String, String> {
    fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {}", e))
}
