use rand::{distributions::Alphanumeric, Rng};

pub fn generate_short_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}
