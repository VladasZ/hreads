use std::sync::mpsc::channel;

use anyhow::Result;
use parking_lot::Mutex;

use crate::main_thread::is_main_thread;

type Callback = Box<dyn FnOnce() + Send>;
type Callbacks = Mutex<Vec<Callback>>;

static CALLBACKS: Callbacks = Callbacks::new(vec![]);

pub fn from_main<T, A>(action: A) -> T
where
    A: FnOnce() -> T + Send + 'static,
    T: Send + 'static, {
    if is_main_thread() {
        return action();
    }

    let (se, re) = channel::<T>();

    on_main(move || {
        se.send(action()).expect("Failed to send result of from_main");
    });

    re.recv().expect("Failed to receive result in from_main")
}

pub fn wait_async<T, A>(action: A) -> T
where
    A: Future<Output = T> + Send + 'static,
    T: Send + 'static, {
    let (se, re) = channel::<T>();

    crate::spawn(async move {
        se.send(action.await).expect("Failed to send result of wait_async");
    });

    re.recv().expect("Failed to receive result in wait_async")
}

pub fn wait_for_next_frame() {
    assert!(
        !is_main_thread(),
        "Waiting for next frame on main thread does nothing"
    );
    from_main(|| {});
}

pub fn on_main(action: impl FnOnce() + Send + 'static) {
    if is_main_thread() {
        action();
    } else {
        CALLBACKS.lock().push(Box::new(action));
    }
}

pub fn ok_main(action: impl FnOnce() + Send + 'static) -> Result<()> {
    on_main(action);
    Ok(())
}

pub fn after(delay: f32, action: impl FnOnce() + Send + 'static) {
    crate::spawn(async move {
        crate::sleep(delay).await;
        CALLBACKS.lock().push(Box::new(action));
    });
}

pub fn invoke_dispatched() {
    for action in CALLBACKS.lock().drain(..) {
        action();
    }
}
