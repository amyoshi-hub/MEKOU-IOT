use crate::OSAI;
use crate::IOT::mem::{FileIO, create_wav};
use std::io;
use std::path::Path;
use std::fs;
use std::error::Error;
use chrono::{Local, NaiveDateTime, Duration};
use serde::{Serialize, Deserialize};
use reqwest::Client;
use tokio::time::sleep;
use std::env::var;

// 定数定義 (main.rsから移動)
const TASK_FILE: &str = "scheduled_tasks.json";
const LYRIC_FILE: &str = "lyric.txt";
const API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-preview-09-2025:generateContent";
const EMOTION_PARAMS: &str = "5,5,5,5,5,5,5,5,5,5,5,5,5,5"; 

// タスクを保存・ロードするための構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub datetime: String, 
    pub name: String,
    pub notified: bool, // 通知済みフラグ
}

// --- Gemini API Call ---

pub async fn gemini_call(query: &str) -> Result<String, Box<dyn Error>> {
    
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            // キーが設定されていない場合、エラーを返して処理を中断
            return Err("Error: GEMINI_API_KEY environment variable not set. Please set your API key.".into());
        }
    };

    let payload = serde_json::json!({
        "contents": [{ "parts": [{ "text": query }] }],
        "tools": [{ "google_search": {} }],
        "systemInstruction": {
            "parts": [{ "text": "Act as a helpful and friendly small AI assistant. Respond concisely and clearly in Japanese." }]
        },
    });

    let client = Client::new();
    let url = format!("{}?key={}", API_URL, api_key);

    for i in 0..3 {
        match client.post(&url).json(&payload).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let json_response: serde_json::Value = response.json().await?;
                    if let Some(text) = json_response["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                        return Ok(text.to_string());
                    } else {
                        return Err("API response format error: missing text content.".into());
                    }
                } else {
                    let status = response.status();
                    let error_body = response.text().await.unwrap_or_else(|_| "No body".to_string());
                    eprintln!("API Error (Attempt {}): Status: {}, Body: {}", i + 1, status, error_body);
                    if i == 2 {
                        return Err(format!("Gemini API failed after 3 attempts. Status: {}", status).into());
                    }
                }
            }
            Err(e) => {
                eprintln!("Network Error (Attempt {}): {}", i + 1, e);
                if i == 2 {
                    return Err(e.into());
                }
            }
        }
        sleep(tokio::time::Duration::from_secs(1 << i)).await;
    }
    
    Err("Failed to get response from Gemini API after all retries.".into())
}


// Geminiからの応答とパラメータを結合し、lyric.txtに保存する
pub async fn generate_and_save_lyric(task_name: &str, task_time_str: &str) -> Result<(), Box<dyn Error>> {
    
    // 1. Task Doc (Gemini呼び出し)
    let user_query = format!(
        "タスク: {} が{}にあります。このタスクの内容を要約し、実行5分前であることを含めて、私に親切に教えてください。絵文字は使わない", 
        task_name, 
        task_time_str
    );

    let gemini_text = match gemini_call(&user_query).await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Gemini Task Error (using fallback): {}", e);
            format!("{}に{}があります。5分前です。起きるです。", task_time_str, task_name)
        }
    };
    
    // 2. Write Task (パラメータの追加とファイルへの書き込み)
    
    // Vocaloidエンジンが単語とパラメータを認識できるフォーマット
    let formatted_text = format!("{},{}", gemini_text, EMOTION_PARAMS);
    
    // lyric.txt に書き込み
    let file_io = FileIO::new(LYRIC_FILE);
    file_io.write_text(&formatted_text)?;
    
    Ok(())
}


// --- Task File Management ---

pub fn load_tasks() -> Vec<Task> {
    if Path::new(TASK_FILE).exists() {
        match fs::read_to_string(TASK_FILE) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| {
                eprintln!("Warning: Failed to parse tasks, starting with empty list.");
                Vec::new()
            }),
            Err(e) => {
                eprintln!("Warning: Failed to read task file: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    }
}

pub fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn Error>> {
    let data = serde_json::to_string_pretty(tasks)?;
    fs::write(TASK_FILE, data)?;
    Ok(())
}

pub fn add_new_task(args: &str) -> Result<Task, Box<dyn Error>> {
    // 入力全体を最大3つの部分に分割します: [日付, 時刻(HH:MM), タスク名]
    // 注: chronoのパースのために、日付と時刻を結合して試行する
    let parts: Vec<&str> = args.split(':').collect();

    // 最低限 "YYYY-MM-DD", "HH", "MM", "NAME" が必要なので、partsの要素は4つ以上。
    // 例: "2025-11-19:08:00:起床" -> ["2025-11-19", "08", "00", "起床"]
    if parts.len() < 4 {
        return Err("Invalid argument format. Use YYYY-MM-DD:HH:MM:Task Name.".into());
    }

    let date_str = parts[0];
    let hour_str = parts[1];
    let minute_str = parts[2];

    // 4番目以降の要素を全て結合してタスク名とする (タスク名にコロンが含まれてもOKにする)
    let name_parts = &parts[3..];
    let name = name_parts.join(":").trim().to_string();

    // 日付と時刻を結合して、パースできる形にする (秒は00として補完)
    let datetime_str_to_parse = format!("{}:{}:{}:00", date_str, hour_str, minute_str);

    // YYYY-MM-DD:HH:MM:SS の形式でパースを試みる
    let format = "%Y-%m-%d:%H:%M:%S";

    match NaiveDateTime::parse_from_str(&datetime_str_to_parse, format) {
        Ok(_) => {
            // パースが成功したら、Taskを構築 (datetimeは YYYY-MM-DD:HH:MM 形式で保存)
            let full_datetime_str = format!("{}:{}:{}", date_str, hour_str, minute_str);
            Ok(Task {
                datetime: full_datetime_str,
                name,
                notified: false, // <-- 修正: 初期値として false を設定
            })
        }
        Err(_) => {
            // パース失敗時、具体的なエラーメッセージを返す
            Err("Invalid date/time format. Ensure date is YYYY-MM-DD and time is HH:MM.".into())
        }
    }
}

pub fn display_tasks(tasks: Vec<Task>) -> String {
    if tasks.is_empty() {
        return "No scheduled tasks.".to_string();
    }
    let mut output = String::from("--- Scheduled Tasks ---\n");
    for (i, task) in tasks.iter().enumerate() {
        let status = if task.notified { "[DONE]" } else { "[PENDING]" };
        output.push_str(&format!("{}. {} | {} {}\n", i + 1, task.datetime, task.name, status));
    }
    output.push_str("-----------------------");
    output
}


// --- Task Scheduler Core ---

pub async fn run_task_scheduler(osai: OSAI, mut initial_tasks: Vec<Task>) {
    
    let mut tasks = initial_tasks;

    // 定数を再利用
    const NOTIFICATION_WINDOW: Duration = Duration::minutes(5); 

    loop {
        // 5秒ごとにチェック
        sleep(tokio::time::Duration::from_secs(5)).await;

        let now = Local::now().naive_local();
        let mut needs_save = false;
        
        // ファイルから最新のタスクをロードし直す
        tasks = load_tasks();

        for task in tasks.iter_mut() {
            if task.notified {
                continue;
            }

            // `NaiveDateTime::parse_from_str` のフォーマットを修正 (":"で結合された形式に対応)
            if let Ok(task_dt) = NaiveDateTime::parse_from_str(&task.datetime, "%Y-%m-%d:%H:%M") { 
                
                let five_min_before = task_dt - NOTIFICATION_WINDOW;
                
                // 5分前ウィンドウに入り、まだ通知されていないかチェック
                if now >= five_min_before && now < task_dt && !task.notified {
                    
                    let task_time_str = task_dt.format("%H時%M分").to_string();

                    // 1. Geminiからの応答を生成し、lyric.txtに保存
                    let generate_result = generate_and_save_lyric(&task.name, &task_time_str).await;

                    // 2. lyric.txt を読み込んで音声合成を実行
                    let vocaloid_result = match generate_result {
                        Ok(_) => osai.emotion_vocaloid(), // OSAIメソッドを使用
                        Err(e) => {
                            eprintln!("Task Preparation Error (Gemini/File Write): {}", e);
                            continue;
                        }
                    };
                    
                    // 3. Aplay.sh コマンドを実行（音声再生）
                    if vocaloid_result.is_ok() {
                        osai.cmd("sh Aplay.sh");
                        println!("\n[TASK ALERT: 5 Min] -> Task written to lyric.txt and Vocaloid triggered.");
                    } else if let Err(e) = vocaloid_result {
                        eprintln!("\n[TASK ALERT ERROR] Vocaloid failed to run: {}", e);
                    }
                    
                    task.notified = true; 
                    needs_save = true;
                }
            }
        }
        
        // 状態が変更されたら保存
        if needs_save {
            let _ = save_tasks(&tasks).map_err(|e| eprintln!("Error saving tasks: {}", e));
        }
    }
}
