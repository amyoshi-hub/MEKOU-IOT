## OSAI_core
osai_core is the low-level engine for the OSAI system, supporting AI interactions via WebSocket and HTTP. Designed for Raspberry Pi and local edge AI deployments.

## Exsample
```rs
use osai_core::OSAI;
use std::io::{Write};
use std::io;

#[tokio::main]
async fn main() {

    println!("input command");
    let mut in_cmd = String::new();
    io::stdin().read_line(&mut in_cmd).unwrap();
    let cmd = in_cmd.trim();

    let  osai = OSAI::new();
    if cmd == "server"{
        let _ = osai.run().await;
    }else if cmd == "http_server"{
        OSAI::http_server().await;
    }else if cmd == "text" {
        OSAI::send_text_cli().await;
    }else{
        println!("no cmd"); 
    }
}
```
## feature
- run: luanch the OSAI server
- http_server: luanch http_server serving file API
- send_text: CLI interface for sending formatted OSAI messages
