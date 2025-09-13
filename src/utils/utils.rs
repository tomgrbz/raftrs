use rand::rngs::ThreadRng;

use crate::Ticks;

pub fn rand_string() -> String {
    use rand::distributions::{Alphanumeric, DistString};
    Alphanumeric.sample_string(&mut rand::thread_rng(), 8)
}

pub fn rand_jitter(seed: Option<ThreadRng>) -> Ticks {
    use rand::{thread_rng, Rng};
    if let Some(mut seed) = seed {
        return seed.gen_range(150..=300);
    }
    let mut rng = thread_rng();
    rng.gen_range(300..=1200)
}

pub fn rand_heartbeat_inteval() -> Ticks {
    use rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    rng.gen_range(50..=70)
}

// #[cfg(test)]
// mod tests {
//     use rand::rngs::ThreadRng;

//     use crate::rand_jitter;

//     #[test]
//     fn test_rand_jitter() {
//         ThreadRng::
//     }
// }
