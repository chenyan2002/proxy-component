//! Runs the proxy's executor alongside the guest's `run`, as the generated `start` does.
wit_bindgen::generate!({ path: "../wit", world: "driver" });

use spike::race::{executor, guest};

struct Driver;

impl exports::spike::race::guest::Guest for Driver {
    async fn run() -> String {
        let executor = wit_bindgen::spawn_local(executor::run());
        let out = guest::run().await;
        executor::stop();
        executor.await;
        out
    }
}

export!(Driver);
