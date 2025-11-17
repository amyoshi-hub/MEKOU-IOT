//use rand::Rng;

const INPUT_SIZE: usize = 14;
const HIDDEN_SIZE: usize = 1024;
const OUTPUT_SIZE: usize = 14;
const ALPHA: f64 = 0.1;
//ここでforの回数
const _EPOCHS: usize = 10;
const SIMI_THRESHOLD: f64 = 0.95;

fn convert_u8_to_f64_array(input: [u8; 14]) -> [f64; 14] {
    let mut result = [0.0; 14];
    for (i, val) in input.iter().enumerate() {
        result[i] = *val as f64 / 255.0;
    }
    result
}

fn convert_f64_to_u8_array(input: [f64; 14]) -> [u8; 14] {
    let mut result = [0u8; 14];
    for (i, val) in input.iter().enumerate() {
        result[i] = (val * 255.0).clamp(0.0, 255.0) as u8;
    }
    result
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + f64::exp(-x))
}

fn rand_init() -> f64 {
    let mut rng = rand::rngs::ThreadRng::default();
    rand::Rng::random_range(&mut rng, -1.0..=1.0)
}


fn train_one_epoch(w1: &mut Vec<Vec<f64>>, w2: &mut Vec<Vec<f64>>, input: [f64; 14], target: [f64; 14]) {
    let mut hidden = vec![0.0; HIDDEN_SIZE];
    for i in 0..HIDDEN_SIZE {
        let mut sum = 0.0;
        for j in 0..INPUT_SIZE {
            sum += w1[i][j] * input[j];
        }
        hidden[i] = sigmoid(sum);
    }

    let mut output = vec![0.0; OUTPUT_SIZE];
    for i in 0..OUTPUT_SIZE {
        let mut sum = 0.0;
        for j in 0..HIDDEN_SIZE {
            sum += w2[i][j] * hidden[j];
        }
        output[i] = sigmoid(sum);
    }

    let mut output_errors = vec![0.0; OUTPUT_SIZE];
    for i in 0..OUTPUT_SIZE {
        output_errors[i] = target[i] - output[i];
    }

    for i in 0..OUTPUT_SIZE {
        for j in 0..HIDDEN_SIZE {
            w2[i][j] += ALPHA * output_errors[i] * hidden[j];
        }
    }

    for i in 0..HIDDEN_SIZE {
        let mut hidden_error = 0.0;
        for k in 0..OUTPUT_SIZE {
            hidden_error += output_errors[k] * w2[k][i];
        }
        for j in 0..INPUT_SIZE {
            w1[i][j] += ALPHA * hidden_error * input[j];
        }
    }
}

fn calc_and_return_output(input: [f64; 14], w1: &mut Vec<Vec<f64>>, w2: &mut Vec<Vec<f64>>) -> [f64; 14] {

    // フォワードパス (隠れ層)
    let mut hidden = vec![0.0; HIDDEN_SIZE];
    for i in 0..HIDDEN_SIZE {
        let mut sum = 0.0;
        for j in 0..INPUT_SIZE {
            sum += w1[i][j] * input[j];
        }
        hidden[i] = sigmoid(sum);
    }

    // フォワードパス (出力層)
    let mut output = [0.0; OUTPUT_SIZE];
    for i in 0..OUTPUT_SIZE {
        let mut sum = 0.0;
        for j in 0..HIDDEN_SIZE {
            sum += w2[i][j] * hidden[j];
        }
        output[i] = sigmoid(sum);
    }

    output
}

fn check_similarity(a: [u8; 14], b: [u8; 14]) -> f64 {
    let mut match_count = 0;
    for i in 0..14 {
        if a[i] == b[i] {
            match_count += 1;
        }
    }
    match_count as f64 / 14.0
}

pub fn ai(my_vec: [u8; 14], target_vec: [u8; 14], w1: &mut Vec<Vec<f64>>, w2: &mut Vec<Vec<f64>>) -> ([u8; 14], bool) {
    let my_input_f64 = convert_u8_to_f64_array(my_vec);
    let target_f64 = convert_u8_to_f64_array(target_vec);

    train_one_epoch(w1, w2, my_input_f64, target_f64);
    let output_f64 = calc_and_return_output(target_f64, w1, w2);
    let output_u8 = convert_f64_to_u8_array(output_f64);

    let similarity = check_similarity(my_vec, output_u8);
    println!("{}", similarity);

    if similarity >= SIMI_THRESHOLD {
        let mut new_vec = [0u8; 14];
        for i in 0..14 {
            new_vec[i] = ((my_vec[i] as u16 + output_u8[i] as u16) / 2) as u8;
        }
        (new_vec, true)
    } else {
        println!("enemy");
        (my_vec, false)
    }
}

