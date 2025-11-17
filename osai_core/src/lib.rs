use std::io::{Write};
use std::io;

pub mod fileIO;

pub mod server;
pub mod client;
use vocaloid;
use std::process::Command;

/*
pub mod file_copy;
pub mod websocket;
pub mod file_server;
pub mod file_read;
pub mod server_list;
use std::net::UdpSocket;
*/
use server::server::start_server;
use client::client::send_text;
use server::web::http_server::{http_server, fetch_file_list, request_file};
/*
use file_copy::{process_and_add_world};
use file_copy::{get_world_list, open_world};
use websocket::start_websocket_server;
use file_server::get_file_list;
use file_read::read_file_content;
use server_list::request_server_list;
*/
mod ai;

pub struct OSAI;

impl OSAI{
    pub fn new() -> Self {
        Self
    }

    pub async fn send_text_cli(){
        let mut dst_ip = String::new();
        let mut dst_port = String::new();
        let mut text = String::new();
        print!("sendTo:");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut dst_ip).unwrap();
        print!("sendPort:");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut dst_port).unwrap();
        print!("sendText:");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut text).unwrap();

        let dst_ip = dst_ip.trim().to_string();
        let dst_port = dst_port.trim().parse::<u16>().unwrap_or(8080);
        let text = text.trim().to_string();

        send_text(dst_ip, dst_port, text).await;
    }

    pub fn request_http(ip: &str){
        let mut ip = String::new();
        io::stdin().read_line(&mut ip);

        let url = format!("http://{}/share/files.json", ip);
        fetch_file_list(url);
    }

    pub async fn http_server() -> Result<(), warp::Error>{
        let _ = http_server().await?;
        Ok(())
    }

    pub fn vocaloid() -> Result<(), hound::Error>{
        vocaloid::emotion_vocaloid();     
        Ok(())
    }

    pub fn play() {
        let output = Command::new("aplay")
            .arg("output.wav")
            .output()
            .expect("faild to call");

        print!("status: {}", output.status);
        print!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        print!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }


    pub async fn run(&self) -> Result<(), String>{
        let _ = start_server("8080".to_string()).await?;
        
        //start_websoket_server();
        //send_text();
        //get_world_list();
        //get_file_list();
        //read_file_content();
        //request_server_list();
        //fetch_file_list();
        //request_file();
        Ok(())
    }
    
    

}

