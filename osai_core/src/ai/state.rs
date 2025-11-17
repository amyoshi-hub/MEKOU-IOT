use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

//serverList
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub addr: String,
    pub port: u16,
}

impl PartialEq for ServerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl Eq for ServerInfo {}
impl Hash for ServerInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
    }
}

pub static SERVER_LIST: Lazy<Mutex<HashSet<ServerInfo>>> = Lazy::new(|| {
    Mutex::new(HashSet::new())
});



//AI state
pub static MY_VEC: Lazy<Mutex<[u8; 14]>> = Lazy::new(|| Mutex::new([0u8; 14]));

pub static W1: Lazy<Mutex<Vec<Vec<f64>>>> = Lazy::new(|| {
    let w1 = (0..1024)
        .map(|_| (0..14).map(|_| rand_init()).collect())
        .collect();
    Mutex::new(w1)
});

pub static W2: Lazy<Mutex<Vec<Vec<f64>>>> = Lazy::new(|| {
    let w2 = (0..14)
        .map(|_| (0..1024).map(|_| rand_init()).collect())
        .collect();
    Mutex::new(w2)
});

fn rand_init() -> f64 {
    let mut rng = rand::rngs::ThreadRng::default();
    rand::Rng::random_range(&mut rng, -1.0..=1.0)
}

