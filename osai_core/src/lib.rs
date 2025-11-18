use std::io::{Write};
use std::io;

pub mod fileIO;

pub mod server;
pub mod client;
pub mod IOT;
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
// Fixed: Removed `request_file` from imports to resolve unused import warning.
use server::web::http_server::{http_server, fetch_file_list}; 

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

    // Fixed: Changed `ip` to `_ip` to resolve the unused variable warning.
    pub fn request_http(_ip: &str){
        let mut ip = String::new();
        // Note: The argument `_ip` is unused as the value is read from stdin.
        io::stdin().read_line(&mut ip).expect("Failed to read line for IP");

        let ip_trimmed = ip.trim();
        let url = format!("http://{}/share/files.json", ip_trimmed);
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
            .expect("failed to call aplay");

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

    // --- Methods used by task.rs ---

    /// Executes Vocaloid emotion processing, resolving the call on the OSAI instance in task.rs.
    // FIX: Now returns a Result type so task.rs can use .is_ok()
    pub fn emotion_vocaloid(&self) -> Result<(), std::io::Error> {
        // Assumes it wraps the existing static function.
        match vocaloid::emotion_vocaloid() {
            Ok(_) => Ok(()),
            // We use a generic IO Error here, as the source of the `hound::Error` is internal.
            // For simplicity in this context, we map it to a basic IO Error.
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Hound Error: {}", e))),
        }
    }

    /// Executes a shell command, resolving the call on the OSAI instance in task.rs.
    pub fn cmd(&self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            eprintln!("Error: Command string is empty.");
            return;
        }

        let program = parts[0];
        let args = &parts[1..];
        
        let output = match Command::new(program)
            .args(args)
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                eprintln!("Failed to execute command '{}': {}", command, e);
                return;
            }
        };

        println!("Command Status: {}", output.status);
        if !output.stdout.is_empty() {
            println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
}
