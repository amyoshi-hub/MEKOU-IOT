use osai_core::OSAI;
use std::io::{self, stdout, Write};
use std::process;
use tokio::io::{AsyncBufReadExt, BufReader}; // tokioの非同期I/Oを使用

// 戻り値の型を、エラー時に Box<dyn std::error::Error> を返すように修正します。
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // OSAI インスタンスの初期化
    let osai = OSAI::new();
    
    // 標準入力を非同期で読み込むための設定
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut buffer = String::new();

    // ターミナルの初期表示
    println!("--- OSAI CLI Interface ---");
    println!("Commands: server, http_server, text, r_file, vocaloid, play, exit");
    
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
        let cmd = buffer.trim();

        // 次のループのために出力結果をリセット
        output = Ok(String::new());
        
        match cmd {
            "server" => { let _ = osai.run().await; }
            "http_server" => OSAI::http_server().await?,
            "text" => OSAI::send_text_cli().await,
            "r_file" => OSAI::request_http("172.20.10.2"),
            "vocaloid" => { OSAI::vocaloid()?; }
            "play" => OSAI::play(),
            "exit" => process::exit(0),
            "" => continue,
            _ => println!("no cmd"),
        } 
    }
}
