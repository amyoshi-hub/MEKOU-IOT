use osai_core::OSAI;
use std::io::{stdout, Write};
use std::process;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    let osai = OSAI::new();

    loop {
        // ループごとに新しい入力バッファ
        let mut input = String::new();

        // コマンドプロンプトを同じ行に表示
        print!("Command:> ");
        stdout.flush()?;

        // ===== 入力ループ =====
        loop {
            if let Event::Key(event) = event::read()? {
                match event.code {
                    KeyCode::Enter => {
                        print!("\n"); // 改行
                        break;       // 入力完了
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                        print!("{}", c);
                        stdout.flush()?;
                    }
                    KeyCode::Backspace => {
                        if input.pop().is_some() {
                            execute!(
                                stdout,
                                cursor::MoveLeft(1),
                                terminal::Clear(ClearType::UntilNewLine)
                            )?;
                        }
                    }
                    _ => {}
                }
            }
        }

        let cmd = input.trim();

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

