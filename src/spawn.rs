#[cfg(wasm)]
pub fn spawn<F>(future: F)
where F: Future<Output = ()> + 'static {
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not_wasm)]
pub fn spawn<F, O>(future: F)
where
    F: Future<Output = O> + Send + 'static,
    O: Send + 'static, {
    tokio::spawn(future);
}

pub fn block_on<F>(future: F)
where F: Future<Output = ()> + 'static {
    #[cfg(wasm)]
    wasm_bindgen_futures::spawn_local(future);
    #[cfg(not_wasm)]
    async_std::task::block_on(future);
}

#[cfg(not_wasm)]
pub fn unasync<F, Out>(future: F) -> Out
where F: Future<Output = Out> {
    async_std::task::block_on(future)
}

pub async fn sleep(duration: f32) {
    #[cfg(not_wasm)]
    async_std::task::sleep(std::time::Duration::from_secs_f32(duration)).await;
    #[cfg(wasm)]
    gloo_timers::future::TimeoutFuture::new((duration * 1000.0) as _).await;
}

pub fn now() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .expect("should have a window")
            .performance()
            .expect("should have performance")
            .now()
            / 1000.0
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs_f64()
    }
}

pub fn busy_sleep(seconds: f32) {
    let start = now();
    let target = start + f64::from(seconds);

    while now() < target {
        std::hint::spin_loop();
    }
}

#[cfg(test)]
mod test {

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{busy_sleep, now};

    #[wasm_bindgen_test(unsupported = test)]
    fn test_busy_sleep() {
        let start = now();
        busy_sleep(0.2);
        let elapsed = now() - start;

        assert!(elapsed >= 0.2);
        assert!(elapsed < 0.25);
    }
}
