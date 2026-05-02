use std::backtrace::Backtrace;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOG_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static PANIC_HOOK_INSTALLED: OnceLock<()> = OnceLock::new();

pub fn install_panic_logging() {
    PANIC_HOOK_INSTALLED.get_or_init(|| {
        std::panic::set_hook(Box::new(|panic_info| {
            let location = panic_info
                .location()
                .map(|location| {
                    format!(
                        "{}:{}:{}",
                        location.file(),
                        location.line(),
                        location.column()
                    )
                })
                .unwrap_or_else(|| "<unknown>".to_string());
            let payload = if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
                (*message).to_string()
            } else if let Some(message) = panic_info.payload().downcast_ref::<String>() {
                message.clone()
            } else {
                "non-string panic payload".to_string()
            };
            let backtrace = Backtrace::force_capture();
            log_line(
                "panic",
                &format!("panic at {location}: {payload}\nbacktrace:\n{backtrace}"),
            );
            eprintln!("panic at {location}: {payload}");
            eprintln!("{backtrace}");
        }));
    });
}

pub fn log_info(scope: &str, message: impl AsRef<str>) {
    log_line(scope, message.as_ref());
}

pub fn log_error(scope: &str, message: impl AsRef<str>) {
    log_line(scope, &format!("ERROR {}", message.as_ref()));
}

pub fn log_result_error<T, E: std::fmt::Display>(
    scope: &str,
    result: Result<T, E>,
) -> Result<T, E> {
    if let Err(error) = &result {
        log_error(scope, error.to_string());
    }
    result
}

pub fn log_path() -> PathBuf {
    LOG_PATH
        .get_or_init(|| default_log_path("artifacts/logs/trekr.log"))
        .clone()
}

fn default_log_path(relative: &str) -> PathBuf {
    PathBuf::from(relative)
}

fn log_line(scope: &str, message: &str) {
    let path = LOG_PATH
        .get_or_init(|| default_log_path("artifacts/logs/trekr.log"))
        .clone();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let timestamp = unix_timestamp_seconds();
    let line = format!("[{timestamp}] [{scope}] {message}\n");
    let lock = LOG_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().ok();
    if let Ok(mut file) = open_append(&path) {
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}

fn open_append(path: &Path) -> std::io::Result<std::fs::File> {
    OpenOptions::new().create(true).append(true).open(path)
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
