use tauri::{command};
use reqwest::Client;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[command]
pub async fn download_file(url: String, file_name: String) -> Result<String, String> {
    // ダウンロードディレクトリのパスを取得
    let download_path = download_dir().ok_or_else(|| "ダウンロードディレクトリが見つかりません".to_string())?;
    
    // 保存先のファイルパスを構築
    let mut dest_path = download_path;
    dest_path.push(&file_name); // 指定されたファイル名を追加

    // HTTPクライアントを作成
    let client = Client::new();

    // ダウンロードURLへのGETリクエスト
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("URLへの接続に失敗しました: {}", e))?;

    // レスポンスのステータスコードをチェック
    if !response.status().is_success() {
        return Err(format!("ダウンロードに失敗しました: HTTPステータス {}", response.status()));
    }

    // ファイルを非同期でダウンロード
    let mut file = File::create(&dest_path)
        .await
        .map_err(|e| format!("ファイルの作成に失敗しました: {}", e))?;

    // レスポンスボディをストリームでファイルに書き込む
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await { // `stream.next()`には`futures::StreamExt`トレイトが必要
        let chunk = chunk.map_err(|e| format!("チャンクの読み込みエラー: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("ファイルへの書き込みエラー: {}", e))?;
    }
    
    Ok(format!("{} にダウンロードが完了しました", dest_path.display()))
}

// main.rs でコマンドを登録
// #[tokio::main] // main関数をtokioランタイムで実行
// fn main() {
//     tauri::Builder::default()
//         .invoke_handler(tauri::generate_handler![download_file]) // ここにコマンドを追加
//         .run(tauri::generate_context!())
//         .expect("error while running tauri application");
// }
