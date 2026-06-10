use std::{
    mem::take,
    sync::mpsc::{Receiver, channel},
};

use anyhow::Result;
use parking_lot::Mutex;
#[cfg(not_wasm)]
use tokio::{
    runtime::{Handle, RuntimeFlavor},
    task::block_in_place,
};

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

    recv_dispatched(&re).expect("Failed to receive result in from_main")
}

/// Receive without starving tokio. A blocked multithread worker hands its
/// queued tasks to other workers before parking.
fn recv_dispatched<T>(re: &Receiver<T>) -> Option<T> {
    #[cfg(not_wasm)]
    {
        let on_multithread_worker =
            Handle::try_current().is_ok_and(|handle| handle.runtime_flavor() == RuntimeFlavor::MultiThread);

        if on_multithread_worker {
            return block_in_place(|| re.recv()).ok();
        }
    }

    re.recv().ok()
}

pub fn wait_async<T, A>(action: A) -> T
where
    A: Future<Output = T> + Send + 'static,
    T: Send + 'static, {
    assert!(!is_main_thread(), "wait_async on the main thread can deadlock");

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
    let actions = take(&mut *CALLBACKS.lock());

    for action in actions {
        action();
    }
}

#[cfg(all(test, not_wasm))]
mod test {
    use serial_test::serial;
    use tokio::runtime::Runtime;

    use crate::{from_main, invoke_dispatched, set_current_thread_as_main};

    #[test]
    #[serial]
    fn from_main_on_tokio_worker() {
        set_current_thread_as_main();

        let rt = Runtime::new().unwrap();
        let handle = rt.spawn(async { from_main(|| 5) });

        while !handle.is_finished() {
            invoke_dispatched();
        }

        assert_eq!(rt.block_on(handle).unwrap(), 5);
    }
}
