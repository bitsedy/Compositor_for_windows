use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

/// Tells its owner when a `.comp` project package changes on disk.
/// Listens to kernel file system events (ReadDirectoryChangesW on Windows via `notify`).
/// Events are coalesced by 300 milliseconds so atomic multi-file saves report once.
pub struct ProjectWatcher {
    _watcher: RecommendedWatcher,
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl ProjectWatcher {
    /// Creates and arms a watcher on the specified `.comp` folder URL/path.
    /// Whenever a change settles after 300ms of quiet, `on_change` is invoked.
    pub fn new<F>(path: &Path, on_change: F) -> Result<Self, notify::Error>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let last_event_time = Arc::new(Mutex::new(None::<Instant>));
        let running = Arc::new(AtomicBool::new(true));

        let last_time_clone = Arc::clone(&last_event_time);
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    // Check if relevant event (modify, create, remove)
                    if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                        if let Ok(mut t) = last_time_clone.lock() {
                            *t = Some(Instant::now());
                        }
                    }
                }
            },
            Config::default(),
        )?;

        watcher.watch(path, RecursiveMode::Recursive)?;

        let callback = Arc::new(on_change);
        let run_flag = Arc::clone(&running);
        let time_tracker = Arc::clone(&last_event_time);

        let thread_handle = thread::spawn(move || {
            const COALESCING: Duration = Duration::from_millis(300);
            while run_flag.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(50));

                let should_fire = {
                    if let Ok(mut opt_time) = time_tracker.lock() {
                        if let Some(time) = *opt_time {
                            if time.elapsed() >= COALESCING {
                                *opt_time = None;
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };

                if should_fire {
                    callback();
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            running,
            thread_handle: Some(thread_handle),
        })
    }

    /// Stops the watcher and cancels pending coalesced deliveries.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for ProjectWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}
