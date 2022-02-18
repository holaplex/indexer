use std::{
    future::Future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use smol::{channel, Executor};

#[derive(Debug)]
pub struct Sender<T> {
    payload: Arc<T>,
    executor: Arc<Executor<'static>>,
    _stop: channel::Sender<()>,
    background_count: AtomicUsize,
    limit: usize,
}

impl<T: 'static> Sender<T> {
    pub fn new(payload: T, limit: usize) -> Self {
        let executor = Arc::new(Executor::new());
        let (stop, stop_rx) = channel::bounded(1);

        std::thread::spawn({
            let executor = executor.clone();

            move || smol::block_on(executor.run(stop_rx.recv()))
        });

        Self {
            payload: Arc::new(payload),
            executor,
            _stop: stop,
            background_count: AtomicUsize::new(0),
            limit,
        }
    }

    async fn wrap_future(f: impl Future<Output = Result<(), anyhow::Error>> + Send + 'static) {
        match f.await {
            Ok(()) => (),
            Err(e) => log::error!("{:?}", e),
        }
    }

    pub async fn run<F: Future<Output = Result<(), anyhow::Error>> + Send + 'static>(
        &self,
        f: impl FnOnce(Arc<T>) -> F,
    ) {
        let new_count =
            self.background_count
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |c| {
                    if c < self.limit { Some(c + 1) } else { None }
                });

        if new_count.is_ok() {
            self.executor
                .spawn(Self::wrap_future(f(Arc::clone(&self.payload))))
                .detach();

            assert!(self.background_count.fetch_sub(1, Ordering::SeqCst) > 0);
        } else {
            Self::wrap_future(f(Arc::clone(&self.payload))).await;
        }
    }
}
