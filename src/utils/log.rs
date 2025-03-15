use std::path::Path;
use std::sync::OnceLock;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry, fmt};

pub const LOG_ENV_VAR: &str = "RUST_LOG";
pub const LOG_OUTPUT_DIR: &str = "log";

/// Initialize the logger.
///
/// If `test_mode` is `true`, it will always set the log level to "trace".
/// Otherwise, it will read the log level from the environment variable
/// specified by [`LOG_ENV_VAR`] and set it to "info" if not present.
/// The log will be written to a file in the directory specified by
/// [`LOG_OUTPUT_DIR`], and the file name will be "test" if `test_mode` is
/// `true` and "ourchat" otherwise.
/// If `debug_cfg` is `Some` and `debug_console` is `true`, it will also
/// write the log to the console at the address specified by
/// `debug_cfg.debug_console_port`.
///
/// # Warning
/// This function should be called only once.
/// The second one will be ignored
pub fn logger_init(output_file: impl AsRef<Path>) {
    static INIT: OnceLock<Option<WorkerGuard>> = OnceLock::new();
    INIT.get_or_init(|| {
        let env = || EnvFilter::try_from_env(LOG_ENV_VAR).unwrap_or("info".into());
        let file_appender = tracing_appender::rolling::daily(LOG_OUTPUT_DIR, output_file);
        let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);
        Registry::default()
            .with(env())
            .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
            .init();
        Some(file_guard)
    });
}
