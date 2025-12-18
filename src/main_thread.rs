use std::{
    sync::atomic::{AtomicU64, Ordering},
    thread::current,
};

use log::error;

static MAIN_THREAD_ID: AtomicU64 = AtomicU64::new(0);

pub fn current_thread_id() -> u64 {
    current().id().as_u64().into()
}

pub fn assert_main_thread() {
    let is_main = is_main_thread();

    if !is_main {
        error!("This operation can be called only from main thread");
    }

    assert!(is_main, "This operation can be called only from main thread");
}

pub fn is_main_thread() -> bool {
    current_thread_id() == supposed_main_id()
}

pub fn set_current_thread_as_main() {
    MAIN_THREAD_ID.store(current_thread_id(), Ordering::Relaxed);
}

pub(crate) fn supposed_main_id() -> u64 {
    let id = MAIN_THREAD_ID.load(Ordering::Relaxed);

    if id == 0 { 1 } else { id }
}

#[cfg(test)]
mod test {
    use std::sync::atomic::Ordering;

    use pretty_assertions::assert_eq;
    use serial_test::serial;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{main_thread::MAIN_THREAD_ID, supposed_main_id};

    #[serial]
    #[wasm_bindgen_test(unsupported = test)]
    fn test() {
        MAIN_THREAD_ID.store(0, Ordering::Relaxed);
        assert_eq!(supposed_main_id(), 1);

        MAIN_THREAD_ID.store(5, Ordering::Relaxed);
        assert_eq!(supposed_main_id(), 5);
    }
}
