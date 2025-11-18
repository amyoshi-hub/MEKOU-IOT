use osai_core::OSAI;
use std::io::{self, stdout, Write, Read};
use std::process;
use tokio::io::{AsyncBufReadExt, BufReader};
use chrono::{Local, NaiveDateTime, Duration};
use std::fs::{self, File};
use std::path::Path;
use serde::{Serialize, Deserialize};
use reqwest::{Client, Error as ReqwestError};

// 定数定義
const TASK_FILE: &str = "scheduled_tasks.json";
const LYRIC_FILE: &str = "lyric.txt";
// APIキーは空文字で設定すると、この環境のランタイムが自動で提供されます。
const API_KEY: &str = ""; 
const API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-preview-09-2025:generateContent";
// ユーザーの擬似コードにあった固定の感情パラメータ
const EMOTION_PARAMS: &str = "5,5,5,5,5,5,5,5,5,5,5,5,5,5"; 

// --- 構造体定義 ---

// タスクを保存・ロードするための構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Task {
    datetime: String, 
    name: String,
    notified: bool, 
}

// ユーザーのコードに合わせたファイルI/O構造体の仮実装
struct FileIO {
    filepath: String,
    contents: Vec<String>,
}

impl FileIO {
    fn new(filepath: &str) -> Self {
        FileIO {
            filepath: filepath.to_string(),
            contents: Vec::new(),
        }
    }

    // ファイルから行ごとに読み込み
    fn read_lines(&mut self) -> Result<(), io::Error> {
        let mut file = File::open(&self.filepath)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        self.contents = content.lines().map(|s| s.to_string()).collect();
        Ok(())
    }

    // テキストをファイルに書き込み
    fn write_text(&self, text: &str) -> Result<(), io::Error> {
        fs::write(&self.filepath, text)?;
        Ok(())
    }

    // 文字列を区切り文字で分割するヘルパー関数
    fn phaser(line: &str, separators: &[&str]) -> Vec<String> {
        let mut parts = vec![line.to_string()];
        for sep in separators {
            let mut new_parts = Vec::new();
            for part in parts {
                new_parts.extend(part.split(sep).map(|s| s.to_string()));
            }
            parts = new_parts;
        }
        parts.into_iter().filter(|s| !s.is_empty()).collect()
    }
}

// --- メイン関数 ---

// 戻り値の型を、エラー時に Box<dyn std::error::Error> を返すように修正します。
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // OSAI インスタンスの初期化
    let osai = OSAI::new();
    
    // 標準入力を非同期で読み込むための設定
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut buffer = String::new();
    
    // タスクリストのロード
    let tasks = load_tasks();

    // ターミナルの初期表示
    println!("--- OSAI CLI Interface (Task System Ready) ---");
    println!("Commands: server, http_server, text, r_file, vocaloid, play, exit, task <date:time:name>, show_tasks, ai <query>");
    
    // バックグラウンドでタスクスケジューラを実行
    let osai_clone = osai.clone();
    let task_handle = tokio::spawn(async move {
        run_task_scheduler(osai_clone, tasks).await;
    });

    // 実行結果を保持する変数
    let mut output: Result<String, Box<dyn std::error::Error>> = Ok(String::new());

    loop {
        // 前回の実行結果を表示
        if output.is_ok() {
            let content = output.as_ref().unwrap();
            if !content.is_empty() {
                 println!("{}", content);
            }
        } else if let Err(e) = &output {
            eprintln!("Error: {}", e);
        }
        
        // コマンドプロンプトを表示
        print!("Command:> ");
        stdout().flush()?;

        // 入力待ち
        buffer.clear();
        reader.read_line(&mut buffer).await?;
        let full_cmd = buffer.trim();
        let mut parts = full_cmd.splitn(2, ' ');
        let cmd = parts.next().unwrap_or("").trim();
        let args = parts.next().unwrap_or("").trim();

        output = Ok(String::new());
        
        // コマンド実行と出力の取得
        match cmd {
            "server" => {
                match osai.run().await {
                    Ok(result) => output = Ok(format!("Server running...\n{}", result)),
                    Err(e) => output = Err(e),
                }
            }
            "http_server" => {
                match OSAI::http_server().await {
                    Ok(result) => output = Ok(format!("HTTP Server finished.\n{}", result)),
                    Err(e) => output = Err(e.into()),
                }
            }
            "text" => {
                 match OSAI::send_text_cli().await {
                    Ok(result) => output = Ok(format!("Text command output:\n{}", result)),
                    Err(e) => output = Err(e),
                }
            }
            "r_file" => {
                match OSAI::request_http("172.20.10.2").await {
                    Ok(result) => output = Ok(format!("File request complete.\n{}", result)),
                    Err(e) => output = Err(e),
                }
            }
            "vocaloid" => {
                match OSAI::emotion_vocaloid() {
                     Ok(_) => output = Ok("Vocaloid command sent (from lyric.txt).".to_string()),
                     Err(e) => output = Err(e.into()),
                }
            }
            "play" => {
                OSAI::play();
                output = Ok("Play command executed.".to_string());
            }
            "task" => {
                match add_new_task(args) {
                    Ok(new_task) => {
                        let mut loaded_tasks = load_tasks();
                        loaded_tasks.push(new_task.clone());
                        save_tasks(&loaded_tasks)?;
                        output = Ok(format!("Task added and saved: {}:{}", new_task.datetime, new_task.name));
                    }
                    Err(e) => output = Err(e),
                }
            }
            "show_tasks" => {
                let current_tasks = load_tasks();
                output = Ok(display_tasks(current_tasks));
            }
            "ai" => {
                if args.is_empty() {
                    output = Err("Error: 'ai' command requires a query.".into());
                } else {
                    match gemini_call(args).await {
                        Ok(response_text) => {
                            println!("[Vocaloid Output]: {}", response_text);
                            output = Ok(format!("AI Response (Text):\n{}", response_text));
                        }
                        Err(e) => output = Err(e),
                    }
                }
            }
            "exit" => {
                task_handle.abort();
                process::exit(0);
            }
            "" => continue,
            _ => output = Ok(format!("no cmd: {}", cmd)),
        }
    }
}

// --- Gemini API Call ---

async fn gemini_call(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    
    let payload = serde_json::json!({
        "contents": [{ "parts": [{ "text": query }] }],
        "tools": [{ "google_search": {} }],
        "systemInstruction": {
            "parts": [{ "text": "Act as a helpful and friendly small AI assistant. Respond concisely and clearly in Japanese." }]
        },
    });

    let client = Client::new();
    let url = format!("{}?key={}", API_URL, API_KEY);

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
        tokio::time::sleep(tokio::time::Duration::from_secs(1 << i)).await;
    }
    
    Err("Failed to get response from Gemini API after all retries.".into())
}

// --- Task and File Management ---

// Geminiからの応答とパラメータを結合し、lyric.txtに保存する
async fn generate_and_save_lyric(task_name: &str, task_time_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    
    // 1. Task Doc (Gemini呼び出し)
    let user_query = format!(
        "タスク: {} が{}にあります。このタスクの内容を要約し、実行5分前であることを含めて、私に親切に教えてください。", 
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

fn load_tasks() -> Vec<Task> {
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

fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_json::to_string_pretty(tasks)?;
    fs::write(TASK_FILE, data)?;
    Ok(())
}

fn add_new_task(args: &str) -> Result<Task, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = args.splitn(3, ':').collect();
    if parts.len() != 3 {
        return Err("Task format must be <date>:<time>:<name> (e.g., 2025-11-18:14:30:ミーティング)".into());
    }
    
    let datetime_str = format!("{} {}", parts[0], parts[1]);
    match NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M") {
        Ok(_) => {
            Ok(Task {
                datetime: datetime_str,
                name: parts[2].to_string(),
                notified: false,
            })
        }
        Err(_) => Err("Invalid date/time format. Use YYYY-MM-DD:HH:MM.".into()),
    }
}

fn display_tasks(tasks: Vec<Task>) -> String {
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

// バックグラウンドでタスクを監視し、通知を行うスケジューラ
async fn run_task_scheduler(osai: OSAI, mut initial_tasks: Vec<Task>) {
    
    let mut tasks = initial_tasks;

    loop {
        // 5秒ごとにチェック
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let now = Local::now().naive_local();
        let mut needs_save = false;
        
        // ファイルから最新のタスクをロードし直す
        tasks = load_tasks();

        for task in tasks.iter_mut() {
            if task.notified {
                continue;
            }

            if let Ok(task_dt) = NaiveDateTime::parse_from_str(&task.datetime, "%Y-%m-%d %H:%M") {
                
                let five_min_before = task_dt - Duration::minutes(5);
                
                if now >= five_min_before && now < task_dt && !task.notified {
                    
                    let task_time_str = task_dt.format("%H時%M分").to_string();

                    // 1. Geminiからの応答を生成し、lyric.txtに保存
                    let generate_result = generate_and_save_lyric(&task.name, &task_time_str).await;

                    // 2. lyric.txt を読み込んで音声合成を実行
                    let vocaloid_result = match generate_result {
                        Ok(_) => OSAI::emotion_vocaloid(),
                        Err(e) => {
                            eprintln!("Task Preparation Error (Gemini/File Write): {}", e);
                            continue;
                        }
                    };
                    
                    // 3. Aplay.sh コマンドを実行（音声再生）
                    if vocaloid_result.is_ok() {
                        OSAI::cmd("sh Aplay.sh");
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

// --- OSAIクレートのダミー実装とヘルパー関数 ---

impl OSAI {
    // ユーザーが提示した emotion_vocaloid の実装
    pub fn emotion_vocaloid() -> Result<(), io::Error>{
        
        let mut file = FileIO::new(LYRIC_FILE);
        file.read_lines()?; // ReadTask() 相当
        
        let sep_list = [","];
        let mut words = Vec::new();
        let mut params = Vec::new();

        for line in &file.contents {
            let splits = FileIO::phaser(line, &sep_list);
            if splits.is_empty() {
                continue;
            }

            let word = splits[0].clone();
            let param: Vec<f32> = splits[1..]
                .iter()
                .filter_map(|s| s.parse::<f32>().ok()) 
                .collect();

            words.push(word);
            params.push(param);
        }

        println!("[Vocaloid Parser] Words: {:?}", words);
        println!("[Vocaloid Parser] Params: {:?}", params);

        // WAVファイル生成のコア処理
        let _ = create_wav(&words, &params); 
        
        Ok(())
    }
    
    // 実行コマンドのダミー (Aplay.sh 実行に相当)
    pub fn cmd(command: &str) {
        println!("[CMD] Executing: {}", command);
    }

    // --- その他、main関数で使用されているOSAIのダミーメソッド ---
    pub fn new() -> Self { OSAI {} }
    pub async fn run(&self) -> Result<String, Box<dyn std::error::Error>> { Ok("Core service started.".to_string()) }
    pub async fn http_server() -> Result<String, io::Error> { Ok("HTTP server started.".to_string()) }
    pub async fn send_text_cli() -> Result<String, Box<dyn std::error::Error>> { Ok("Text service running.".to_string()) }
    pub async fn request_http(_addr: &str) -> Result<String, Box<dyn std::error::Error>> { Ok("HTTP request simulated.".to_string()) }
    pub fn play() { println!("Simulating audio playback."); }
}

// WAVファイル生成のダミー関数（コンパイルを通すため）
fn create_wav(words: &[String], params: &[Vec<f32>]) -> Result<(), io::Error> {
    if words.is_empty() {
         println!("[WAV] No content to synthesize.");
         return Ok(());
    }
    println!("[WAV] Successfully synthesized {} word(s) into WAV.", words.len());
    Ok(())
}
