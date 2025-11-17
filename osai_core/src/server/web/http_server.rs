//use std::convert::Infallible;
use local_ip_address::local_ip;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use warp::Filter;
use reqwest;
use serde::Deserialize;
//use std::fs;
//use std::path::Path;
use base64::engine::general_purpose;
use base64::Engine;

pub async fn http_server() -> Result<(), warp::Error>{
    let dir = "/share";
    let files = warp::fs::dir(dir);

    // ローカルIPアドレスを取得
    let ip: IpAddr = match local_ip() {
        Ok(ip) => ip,
        Err(_) => {
            eprintln!("Failed to get local IP address. Using 127.0.0.1");
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        }
    };

    println!("Starting HTTP file server at http://{}:1234/", ip);

    warp::serve(files)
        .run((ip, 1234))
        .await;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct FileList {
    pub files: Vec<String>,
}

pub async fn fetch_file_list(url: String) -> Result<Vec<String>, String> {
    // GETリクエスト
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("リクエスト失敗: {}", e))?;

    // レスポンス本文（文字列）
    let body = resp.text()
        .await
        .map_err(|e| format!("レスポンス読み込み失敗: {}", e))?;

    // JSON形式でファイル名配列をパースする想定
    // 例: {"files": ["a.txt", "b.png", "c.json"]}
    let file_list: FileList = serde_json::from_str(&body)
        .map_err(|e| format!("JSONパース失敗: {}", e))?;

    Ok(file_list.files)
}

pub async fn request_file(file_name: String, ip: String) -> Result<String, String> {
    let file_url = format!("http://{}:1234/share/{}", ip, file_name);

    // HTTP GET リクエストを送信
    let response = reqwest::get(&file_url)
        .await
        .map_err(|e| format!("HTTP リクエスト失敗: {}", e))?;

    // ステータスコードを確認
    if !response.status().is_success() {
        return Err(format!("HTTP エラー: {}", response.status()));
    }

    // バイナリデータを取得
    //let bytes = response.bytes().await.map(|bytes| bytes.to_vec()).map_err(|e| format!("レスポンス読み込み失敗: {}", e));
    let bytes = response.bytes().await.map_err(|e| format!("レスポンス読み込み失敗: {}", e))?;
    let base64_encoded = general_purpose::STANDARD.encode(&bytes);
    Ok(base64_encoded)
}

