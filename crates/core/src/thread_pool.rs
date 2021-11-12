use std::{
    panic::{AssertUnwindSafe, RefUnwindSafe, UnwindSafe},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
};

use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use parking_lot::{Condvar, Mutex};

/// Crash if a panic escapes confinement
#[must_use = "Pass the returned value to mem::forget to prevent aborting if no panic occurred"]
fn defer_abort() -> dispose::Disposable<impl FnOnce()> {
    dispose::defer(|| {
        std::process::abort();
    })
}

struct ThreadPoolCore<J> {
    inj: Injector<J>,
    steal: Box<[Stealer<J>]>,
    live: Mutex<usize>,
    unpark_var: Condvar,
    join_var: Condvar,
    stop: AtomicBool,
}

pub struct ThreadPoolHandle<'a, J>(AssertUnwindSafe<&'a Arc<ThreadPoolCore<J>>>);
pub struct ThreadPool<J>(Arc<ThreadPoolCore<J>>, Vec<JoinHandle<()>>);

impl<J> ThreadPoolCore<J> {
    /// **NOTE:** Use with care!  This is not atomic.
    fn is_empty(&self) -> bool {
        self.inj.is_empty() && self.steal.iter().all(Stealer::is_empty)
    }

    fn park(&self) {
        if self.stop.load(Ordering::SeqCst) {
            return;
        }

        let mut live = self.live.lock();
        *live -= 1;

        if *live == 0 {
            self.join_var.notify_all();
        }

        self.unpark_var.wait(&mut live);
        *live += 1;
    }

    /// # A note on soundness
    /// This only works because the exposed function consumes the thread pool,
    /// revoking outside access to the push() function.  This makes `is_empty` a
    /// sound approximation as no items can be added if no threads are live.
    fn join(&self) {
        let mut live = self.live.lock();

        while !(*live == 0 && self.is_empty()) {
            self.join_var.wait(&mut live);
        }

        self.abort();
    }

    fn abort(&self) {
        self.stop.store(true, Ordering::SeqCst);
        self.unpark_var.notify_all();
    }

    fn push(&self, job: J) {
        self.inj.push(job);
        self.unpark_var.notify_one();
    }
}

impl<'a, J> ThreadPoolHandle<'a, J> {
    pub fn push(&self, job: J) {
        self.0.push(job);
    }
}

impl<J: Send + UnwindSafe + 'static> ThreadPool<J> {
    /// # Panics
    /// This function panics and aborts the process if a thread cannot be created successfully.
    pub fn new(
        num_threads: impl Into<Option<usize>>,
        f: impl Fn(J, ThreadPoolHandle<J>) + Send + Clone + RefUnwindSafe + 'static,
    ) -> Self {
        let num_threads = num_threads.into().unwrap_or_else(num_cpus::get);

        let work = (0..num_threads)
            .map(|i| (i, Worker::new_fifo()))
            .collect::<Vec<_>>();

        let steal = work
            .iter()
            .map(|(_, w)| w.stealer())
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let core = Arc::new(ThreadPoolCore {
            inj: Injector::new(),
            steal,
            stop: AtomicBool::new(false),
            live: Mutex::new(num_threads),
            unpark_var: Condvar::new(),
            join_var: Condvar::new(),
        });

        let abort = defer_abort();

        let threads = work
            .into_iter()
            .map(|(index, work)| {
                std::thread::Builder::new()
                    .name(format!("Worker thread {}", index))
                    .spawn({
                        let core = core.clone();
                        let f = f.clone();

                        move || WorkerThread { index, work, core }.run(f)
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>();

        std::mem::forget(abort);

        Self(core, threads)
    }
}

impl<J> ThreadPool<J> {
    #[inline]
    pub fn push(&self, job: J) {
        self.0.push(job);
    }

    /// # Panics
    /// This function panics if any of the threads in this pool have panicked
    /// without already aborting the process.
    pub fn join(mut self) {
        self.0.join();

        for handle in self.1.drain(..) {
            handle.join().unwrap();
        }

        // Final sanity check
        assert!(self.0.is_empty(), "Thread pool starved!");
    }

    #[inline]
    pub fn abort(&self) {
        self.0.abort();
    }
}

impl<J> Drop for ThreadPool<J> {
    fn drop(&mut self) {
        self.abort();
    }
}

struct WorkerThread<J> {
    index: usize,
    work: Worker<J>,
    core: Arc<ThreadPoolCore<J>>,
}

impl<J: UnwindSafe> WorkerThread<J> {
    fn get_job(&self) -> Option<J> {
        self.work.pop().or_else(|| {
            let WorkerThread { work, .. } = self;
            let ThreadPoolCore {
                ref stop,
                ref inj,
                ref steal,
                ..
            } = *self.core;

            loop {
                if stop.load(Ordering::Acquire) {
                    break None;
                }

                match inj
                    .steal_batch_and_pop(work)
                    .or_else(|| steal.iter().map(Stealer::steal).collect())
                {
                    Steal::Empty => break None,
                    Steal::Success(job) => break Some(job),
                    Steal::Retry => (),
                }
            }
        })
    }

    fn run(self, f: impl Fn(J, ThreadPoolHandle<J>) + RefUnwindSafe) {
        let abort = defer_abort();

        while !self.core.stop.load(Ordering::Acquire) {
            if let Some(job) = self.get_job() {
                let core_ref = AssertUnwindSafe(&self.core);
                match std::panic::catch_unwind(|| f(job, ThreadPoolHandle(core_ref))) {
                    Ok(()) => (),
                    Err(e) => log::error!("Job panicked: {:?}", e),
                }
            } else {
                self.core.park();
            }
        }

        std::mem::forget(abort);
    }
}
