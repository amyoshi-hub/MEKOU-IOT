fn main() {
    read(output.wav)

    let data_vec: [uu8; 14] = features
    .iter()
    .map(|f| (f * 255.0).min(255.0).max(0.0) as u8)
    .collect::<Vec<u8>>()
    .try_into()
    .unwrap();
}
