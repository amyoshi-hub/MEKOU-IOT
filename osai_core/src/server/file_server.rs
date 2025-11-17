use std::path::{PathBuf};
use std::error::Error;
use std::fs;

// "share" ディレクトリの作成または取得
fn get_or_create_share_dir() -> Result<String, String> {
    let base_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Could not get exe dir")?
        .join("share");

    // ディレクトリがなければ作成
    if !base_dir.exists() {
        std::fs::create_dir(&base_dir).map_err(|e| e.to_string())?;
    }
    Ok(base_dir.to_string_lossy().to_string())
}

// 指定されたディレクトリ内のファイル一覧を取得
fn read_dir(dir_path: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let dir = fs::read_dir(dir_path)?;
    let mut files: Vec<PathBuf> = Vec::new();
    for item in dir {
        files.push(item?.path());
    }
    Ok(files)
}

pub async fn get_file_list() -> Result<Vec<String>, String> {
    // "share" ディレクトリのパスを取得または作成
    let base_dir = get_or_create_share_dir()
        .map_err(|e| format!("Failed to get or create share directory: {}", e))?;
    println!("{:?}", base_dir);

    // ディレクトリ内のファイル一覧を取得
    match read_dir(&base_dir) {
        Ok(paths) => {
            let file_list: Vec<String> = paths
                .iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect();
            Ok(file_list)
        }
        Err(e) => Err(format!("Failed to read directory: {}", e)),
    }
}

