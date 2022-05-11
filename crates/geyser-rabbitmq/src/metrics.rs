use std::sync::Arc;

use log::Level;
use parking_lot::Mutex;
use solana_metrics::counter::Counter as CounterInner;

// Despite being entirely atomic, Solana's counter still requires a mutable
// borrow for the inc() method.  So we have to do this awful Mutex<Atomic>
// pattern unless they change that.
pub struct Counter(Mutex<CounterInner>, Level);

// Solana's counter also doesn't implement Debug.
impl std::fmt::Debug for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("Counter")
            .field(&self.0.try_lock().map_or("<locked>", |c| c.name))
            .field(&self.1)
            .finish()
    }
}

impl Counter {
    #[inline]
    fn new(name: &'static str, lvl: Level) -> Self {
        let mut inner = CounterInner {
            name,
            counts: 0.into(),
            times: 0.into(),
            lastlog: 0.into(),
            lograte: 0.into(),
            metricsrate: 0.into(),
        };

        inner.init();
        Self(Mutex::new(inner), lvl)
    }

    pub fn log(&self, n: usize) {
        self.0.lock().inc(self.1, n);
    }
}

#[derive(Debug)]
pub struct Metrics {
    pub sends: Counter,
    pub recvs: Counter,
    pub errs: Counter,
    pub reconnects: Counter,
}

impl Metrics {
    pub fn new_rc() -> Arc<Self> {
        Arc::new(Self {
            sends: Counter::new("geyser_sends", Level::Info),
            recvs: Counter::new("geyser_recvs", Level::Info),
            errs: Counter::new("geyser_errs", Level::Error),
            reconnects: Counter::new("geyser_reconnects", Level::Error),
        })
    }
}
