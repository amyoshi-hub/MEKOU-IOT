use osai_core::OSAI;
use std::io::{self, stdout, Write};
use std::process;
use tokio::io::{AsyncBufReadExt, BufReader};

use osai_core::IOT::task::add_new_task;
// FIX 1: Removed `use std::env::args;` to prevent conflict with the argument variable name.
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

        // FIX 2: ユーザー入力をメインコマンドと引数文字列に分割
        let mut parts = full_cmd.splitn(2, ' ');
        let main_command = parts.next().unwrap_or("");
        let args_str = parts.next().unwrap_or(""); // 引数部分
        
        // 次のループのために出力結果をリセット
        output = Ok(String::new());
        
        // FIX 3: マッチする対象を main_command に変更
        match main_command {
            "server" => { let _ = osai.run().await; }
            "http_server" => OSAI::http_server().await?,
            "text" => OSAI::send_text_cli().await,
            "r_file" => OSAI::request_http("172.20.10.2"),
            "vocaloid" => { OSAI::vocaloid()?; }
            "play" => OSAI::play(),
            "task" => {
                // FIX: args_str を渡す
                match add_new_task(args_str) {
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
                // FIX: args_str の空チェックと渡し
                if args_str.is_empty() {
                    output = Err("Error: 'ai' command requires a query.".into());
                } else {
                    match gemini_call(args_str).await {
                        Ok(response_text) => {
                            println!("[Vocaloid Output]: {}", response_text);
                            output = Ok(format!("AI Response (Text):\n{}", response_text));
                        }
                        Err(e) => output = Err(e),
                    }
                }
            }
            "help" => {
                println!("Available commands:");
                println!("  ai <query>         : Ask the Gemini AI a question and get a vocal response.");
                println!("  task add <YYYY-MM-DD:HH:MM:Name> : Add a new task (e.g., task add 2025-12-25:08:30:Wake up call)");
                println!("  task list          : Display all scheduled tasks.");
                println!("  exit | quit        : Stop the application.");
                println!("  vocaloid <text>    : Speak custom text (emotion parameters will be used).");
                println!("  cmd <command>      : Execute a shell command (e.g., cmd ls -l).");
            }
            "exit" => process::exit(0),
            "" => continue,
            _ => println!("Unknown command: {}", full_cmd),
        } 
    }
}
