use std::fs::OpenOptions;
use std::io::Write;

pub fn create_lyric(text: &str, vec14: [u8; 14]){
    let vec_str: Vec<String> = vec14.iter().map(|v| v.to_string()).collect();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("lyric.txt")
        .expect("Failed to open lyric.txt");

    // ここでは簡単にカンマ区切りに変換
    let line = format!("{},{}\n", text, vec_str.join(","));

    file.write_all(line.as_bytes())
        .expect("Failed to write to lyric.txt");

    println!("Created lyric for '{}'", text);
}
