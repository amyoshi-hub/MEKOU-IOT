use osai_core::OSAI;
use std::io::{self, stdout, Write};
use std::process;
use tokio::io::{AsyncBufReadExt, BufReader};

// osai_core::IOT::task から必要な関数をインポート
use osai_core::IOT::task::add_new_task;
use osai_core::IOT::task::load_tasks;
use osai_core::IOT::task::gemini_call;
use osai_core::IOT::task::display_tasks;
use osai_core::IOT::task::save_tasks;
use osai_core::IOT::task::run_task_scheduler;


// 戻り値の型を、エラー時に Box<dyn std::error::Error> を返すように修正します。
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // OSAI インスタンスの初期化
    let osai = OSAI::new();

    let initial_tasks = load_tasks();
    println!("Loaded {} tasks.", initial_tasks.len());

    let scheduler_osai = osai.clone(); // OSAIをクローンしてタスクに所有権を渡す
    tokio::spawn(async move {
        run_task_scheduler(scheduler_osai, initial_tasks).await;
    });
    println!("\nOSAI-Core Task Manager is RUNNING.");


    
    // 標準入力を非同期で読み込むための設定
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut buffer = String::new();

    // ターミナルの初期表示
    println!("--- OSAI CLI Interface ---");
    println!("Commands: server, http_server, text, r_file, vocaloid, play, task <date:time:name>, show_tasks, ai <query>, exit");
    
    // 実行結果を保持する変数。ループ内で使用
    let mut output: Result<String, Box<dyn std::error::Error>> = Ok(String::new());

    loop {
        // 前回の実行結果を表示
        if output.is_ok() {
            // 前回の結果がOkの場合、その内容を表示（ただし初回は空）
            let content = output.as_ref().unwrap();
            if !content.is_empty() {
                println!("{}", content);
            }
        } else if let Err(e) = &output {
            // エラーが発生した場合は、エラーメッセージを表示
            eprintln!("Error: {}", e);
        }
        
        // --- コマンドプロンプトを表示 ---
        print!("Command:> ");
        stdout().flush()?;

        // --- 入力待ち（Enterが押されるまでブロック）---
        buffer.clear();
        reader.read_line(&mut buffer).await?;
        let full_cmd = buffer.trim();

        // ユーザー入力をメインコマンドと引数文字列に分割
        let mut parts = full_cmd.splitn(2, ' ');
        let main_command = parts.next().unwrap_or("");
        let args_str = parts.next().unwrap_or(""); // 引数部分
        
        // 次のループのために出力結果をリセット
        output = Ok(String::new());
        
        // マッチする対象を main_command に変更
        match main_command {
            "server" => { let _ = osai.run().await; }
            "http_server" => OSAI::http_server().await?,
            "text" => OSAI::send_text_cli().await,
            "r_file" => OSAI::request_http("172.20.10.2"),
            
            // vocaloid コマンドの呼び出し
            "vocaloid" => { 
                // FIX: map_err の戻り値の型を明示
                OSAI::vocaloid(args_str).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
                output = Ok(format!("Vocaloid processing complete for: '{}'", args_str));
            }
            "play" => OSAI::play(),
            "task" => {
                // ... task ロジックは省略せずにそのまま
                match add_new_task(args_str) {
                    Ok(new_task) => {
                        let mut loaded_tasks = load_tasks();
                        loaded_tasks.push(new_task.clone());
                        save_tasks(&loaded_tasks)?;
                        output = Ok(format!("Task added and saved: {}:{}", new_task.datetime, new_task.name));
                    }
                    Err(e) => output = Err(e.into()), // エラー型を Box<dyn Error> に変換
                }
            }
            "show_tasks" => {
                let current_tasks = load_tasks();
                output = Ok(display_tasks(current_tasks));
            }
            
            "ai" => {
                if args_str.is_empty() {
                    output = Err("Error: 'ai' command requires a query.".into());
                } else {
                    // Gemini APIを呼び出し、応答を得る
                    match gemini_call(&(args_str.to_owned() + "ひらがなで返して")).await {
                        Ok(response_text) => {
                            println!("[AI Text Generated]: {}", response_text);
                            
                            // 1. AI応答テキストを vocaloid に渡し、エラーを Box<dyn Error>に変換して伝播
                            // FIX: map_err の戻り値の型を明示
                            OSAI::vocaloid(&response_text).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
                            
                            // 2. vocaloid 成功後、play を呼び出す
                            OSAI::play();
                            
                            output = Ok(format!("AI Response (Text):\n{}\n[Vocalization complete. Playing audio...]", response_text));
                        }
                        Err(e) => output = Err(e.into()), // エラー型を Box<dyn Error> に変換
                    }
                }
            }
            "help" => {
                output = Ok(format!(
                    "Available commands:
  ai <query>         : Ask the Gemini AI a question and get a vocal response.
  task <date:time:name> : Add a new task (e.g., task 2025-12-25:08:30:Wake up call)
  show_tasks         : Display all scheduled tasks.
  vocaloid <text>    : Speak custom text.
  exit | quit        : Stop the application."
                ));
            }
            "exit" | "quit" => process::exit(0),
            "" => continue,
            _ => output = Err(format!("Unknown command: {}", full_cmd).into()), // 未知のコマンドもエラーとして扱う
        }
    }
}
