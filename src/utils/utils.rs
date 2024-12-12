use crate::Ticks;

pub fn rand_string() -> String {
    use rand::distributions::{Alphanumeric, DistString};
    Alphanumeric.sample_string(&mut rand::thread_rng(), 8)
}

pub fn rand_jitter() -> Ticks {
    use rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    rng.gen_range(150..=300)
}

pub fn rand_heartbeat_inteval() -> Ticks {
    use rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    rng.gen_range(50..=90)
}
