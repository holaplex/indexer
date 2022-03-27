use std::{
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::Duration,
};

use smol::{channel, Executor, Timer};

#[derive(Debug)]
pub struct Counter(AtomicI64);

impl Counter {
    #[inline]
    fn new() -> Self {
        Self(AtomicI64::new(0))
    }

    pub fn log(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }

    fn get(&self) -> i64 {
        self.0.swap(0, Ordering::SeqCst)
    }
}

#[derive(Debug)]
pub struct Metrics {
    _executor: Arc<Executor<'static>>,
    _stop: channel::Sender<()>,
    pub sends: Counter,
    pub fg_sends: Counter,
    pub recvs: Counter,
    pub errs: Counter,
    pub reconnects: Counter,
}

impl Metrics {
    pub fn new_rc() -> Arc<Self> {
        let executor = Arc::new(Executor::new());
        let (stop_tx, stop_rx) = channel::bounded(1);

        std::thread::spawn({
            let executor = executor.clone();

            move || smol::block_on(executor.run(stop_rx.recv()))
        });

        let this = Arc::new(Self {
            _executor: executor.clone(),
            _stop: stop_tx,
            sends: Counter::new(),
            fg_sends: Counter::new(),
            recvs: Counter::new(),
            errs: Counter::new(),
            reconnects: Counter::new(),
        });

        executor
            .spawn({
                let this = Arc::clone(&this);

                async move {
                    loop {
                        Timer::after(Duration::from_secs(30)).await;

                        this.submit();
                    }
                }
            })
            .detach();

        this
    }

    fn submit(&self) {
        solana_metrics::datapoint_info!(
            "accountsdb_rabbitmq",
            ("msgs_sent", self.sends.get(), i64),
            ("blocking_sends", self.fg_sends.get(), i64),
            ("evts_recvd", self.recvs.get(), i64),
        );

        solana_metrics::datapoint_error!(
            "accountsdb_rabbitmq",
            ("errors", self.errs.get(), i64),
            ("reconnects", self.reconnects.get(), i64),
        );
    }
}
