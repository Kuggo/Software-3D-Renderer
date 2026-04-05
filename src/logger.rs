use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;


// ==========================
// Public API
// ==========================

pub struct Logger;
impl Logger {
    pub fn info(msg: &str) {
        let _ = LOGGER.send(LogMsg::Entry(LogLevel::Info, msg.to_string()));
    }

    pub fn warn(msg: &str) {
        let _ = LOGGER.send(LogMsg::Entry(LogLevel::Warn, msg.to_string()));
    }

    pub fn error(msg: &str) {
        let _ = LOGGER.send(LogMsg::Entry(LogLevel::Error, msg.to_string()));
    }

    pub fn info_now(msg: &str) {
        print(LogLevel::Info, &msg.to_string(), 1);
    }

    pub fn warn_now(msg: &str) {
        print(LogLevel::Warn, &msg.to_string(), 1);
    }

    pub fn error_now(msg: &str) {
        print(LogLevel::Error, &msg.to_string(), 1);
    }

    pub fn init_with_interval(interval: Duration) {
        Lazy::force(&LOGGER); // ensure init
        let _ = LOGGER.send(LogMsg::SetInterval(interval));
    }
}



// ==========================
// Internal Types
// ==========================

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum LogLevel {
    Info,
    Warn,
    Error,
}

enum LogMsg {
    Entry(LogLevel, String),
    SetInterval(Duration),
}

struct LogAggregator {
    order: Vec<(LogLevel, String)>,
    counts: HashMap<(LogLevel, String), usize>,
}

impl LogAggregator {
    fn new() -> Self {
        Self {
            order: Vec::new(),
            counts: HashMap::new(),
        }
    }

    fn log(&mut self, level: LogLevel, msg: String) {
        let key = (level, msg.clone());

        if let Some(count) = self.counts.get_mut(&key) {
            *count += 1;
        } else {
            self.order.push((level, msg.clone()));
            self.counts.insert(key, 1);
        }
    }

    fn flush(&mut self) {
        for (level, msg) in &self.order {
            let key = (*level, msg.clone());
            let count = self.counts.get(&key).unwrap();

            print(*level, msg, *count);
        }

        self.order.clear();
        self.counts.clear();
    }
}



// ==========================
// Global Logger (Channel)
// ==========================

static LOGGER: Lazy<Sender<LogMsg>> = Lazy::new(|| {
    let (tx, rx) = channel::<LogMsg>();

    thread::spawn(move || {
        let mut logger = LogAggregator::new();
        let mut interval = Duration::from_secs(1);
        let mut last_flush = Instant::now();

        loop {
            // Drain messages
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    LogMsg::Entry(level, text) => {
                        logger.log(level, text);
                    }
                    LogMsg::SetInterval(new_interval) => {
                        interval = new_interval;
                    }
                }
            }

            // Flush if time elapsed
            if last_flush.elapsed() >= interval {
                logger.flush();
                last_flush = Instant::now();
            }

            // Prevent busy loop
            thread::sleep(Duration::from_millis(10));
        }
    });

    tx
});

// ==========================
// Helpers
// ==========================

fn print(level: LogLevel, msg: &String, count: usize) {
    let colored = colorize(level, msg);

    if count > 1 {
        println!("[x{}] {}", count, colored);
    } else {
        println!("{}", colored);
    }
}

// ANSI color codes
fn colorize(level: LogLevel, msg: &str) -> String {
    match level {
        LogLevel::Info => format!("\x1b[34m{}\x1b[0m", msg),  // Blue
        LogLevel::Warn => format!("\x1b[33m{}\x1b[0m", msg),  // Yellow
        LogLevel::Error => format!("\x1b[31m{}\x1b[0m", msg), // Red
    }
}