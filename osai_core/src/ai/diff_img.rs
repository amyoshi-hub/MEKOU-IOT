use image::{open, DynamicImage, GenericImageView, GrayImage};
use std::error::Error;

// 画像をグレースケールに変換してピクセル値を取得
fn load_image_as_grayscale(image_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let img = open(image_path)?.to_luma8();
    Ok(img.into_raw())
}

// ユークリッド距離で画像類似度を計算
fn calculate_similarity(img1: &[u8], img2: &[u8]) -> f64 {
    let mut diff_sum = 0.0;
    for (p1, p2) in img1.iter().zip(img2.iter()) {
        let diff = (*p1 as f64 - *p2 as f64).abs();
        diff_sum += diff * diff;
    }
    diff_sum.sqrt()
}

fn main() -> Result<(), Box<dyn Error>> {
    // 画像パスを指定
    let img1_path = "path/to/image1.png";
    let img2_path = "path/to/image2.png";

    // グレースケール画像を取得
    let img1 = load_image_as_grayscale(img1_path)?;
    let img2 = load_image_as_grayscale(img2_path)?;

    // 比較
    let similarity = calculate_similarity(&img1, &img2);
    println!("Image similarity (lower is better): {:.2}", similarity);

    Ok(())
}

