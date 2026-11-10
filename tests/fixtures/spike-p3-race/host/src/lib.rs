//! `fetch(key)` waits `DELAY_<KEY>` milliseconds, or a random 0-19 if unset, then returns `key` in
//! uppercase. A delay of 0 returns at once, without awaiting anything.
wit_bindgen::generate!({ path: "../wit", world: "race-host" });

struct Host;

impl exports::spike::race::host::Guest for Host {
    async fn fetch(key: String) -> String {
        let delay = std::env::var(format!("DELAY_{}", key.to_uppercase()))
            .ok()
            .and_then(|delay| delay.parse::<u64>().ok())
            .unwrap_or_else(|| u64::from(wasip3::random::random::get_random_bytes(1)[0] % 20));
        if delay > 0 {
            wasip3::clocks::monotonic_clock::wait_for(delay * 1_000_000).await;
        }
        key.to_uppercase()
    }
}

export!(Host);
