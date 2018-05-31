extern crate log;
extern crate thread_id;
use log::{Record, Level, Metadata, RecordBuilder};
use log::{SetLoggerError};

use config::CONFIG;

use template::logger::Log;

use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::io;
use std::io::Write;
use std::sync::Mutex;
use std::time::Instant;
use std::time::Duration;
use std::fs::create_dir_all;
use std::fs::File;

lazy_static!(
    static ref LOGGER: Logger = Logger {
        start: Instant::now(),
        stdout: Mutex::new(Box::new(io::stdout())),
        stderr: Mutex::new(Box::new(io::stderr())),
        file: Mutex::new(None),
    };
);

pub fn init() -> Result<Sender<(String, Log)>, SetLoggerError> {
    macro_rules! plugin_log {
        ($plugin_name:expr, target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
            let lvl = $lvl;
            PluginLog::log_plugin(
                &LOGGER,
                &RecordBuilder::new()
                    .args(format_args!($($arg)+))
                    .level(lvl)
                    .target($target)
                    .module_path(Some(module_path!()))
                    .file(Some(file!()))
                    .line(Some(line!()))
                    .build(),
                &$plugin_name
            )
        });
        ($plugin_name:expr, $lvl:expr, $($arg:tt)+) => (plugin_log!($plugin_name, target: module_path!(), $lvl, $($arg)+))
    }

    let (sender, receiver) = channel::<(String, Log)>();

    let log = CONFIG.log();
    log::set_max_level(log.level());

    let result = log::set_logger(&LOGGER);
    match result {
        Ok(_) => {
            if CONFIG.log().to_file() {
                let mut path = log.path();
                create_dir_all(path.as_path()).expect(&format!("Failed to create the directory '{}'", path.to_str().unwrap()));
                path.push("BEST-Bot.log");
                match File::create(path) {
                    Ok(f) => *LOGGER.file.lock().unwrap() = Some(Box::new(f)),
                    Err(e) => error!("could not create the log file ({:?})", e),
                }
            }

            thread::spawn(move || {
                let receiver = receiver;
                loop {
                    match receiver.recv() {
                        Ok(l) => match l {
                            (plugin_name, Log::Error(msg)) => plugin_log!(plugin_name, Level::Error, "{}", msg),
                            (plugin_name, Log::Warn(msg))  => plugin_log!(plugin_name, Level::Warn,  "{}", msg),
                            (plugin_name, Log::Info(msg))  => plugin_log!(plugin_name, Level::Info,  "{}", msg),
                            (plugin_name, Log::Debug(msg)) => plugin_log!(plugin_name, Level::Debug, "{}", msg),
                            (plugin_name, Log::Trace(msg)) => plugin_log!(plugin_name, Level::Trace, "{}", msg),
                        }
                        Err(_) => break,
                    }
                }
            });

            Ok(sender)
        },
        Err(e) => Err(e),
    }
}

fn write<S: Write>(sink: &mut S, now: Duration, record: &Record, plugin_name: &str) {
    let seconds = now.as_secs();
    let hours = seconds / 3600;
    let minutes = (seconds / 60) % 60;
    let seconds = seconds % 60;
    let miliseconds = now.subsec_nanos() / 1_000_000;

    let _ = write!(
        sink,
        "[{:02}:{:02}:{:02}.{:03}] ({:x}) [{}] {:6} {}\n",
        hours,
        minutes,
        seconds,
        miliseconds,
        thread_id::get(),
        plugin_name,
        record.level(),
        record.args()
    );
}

struct Logger {
    start: Instant,
    stdout: Mutex<Box<Write + Send>>,
    stderr: Mutex<Box<Write + Send>>,
    file: Mutex<Option<Box<Write + Send>>>,
}

trait PluginLog: log::Log {
    fn log_plugin(&self, record: &Record, plugin_name: &str);
}

impl PluginLog for LOGGER {
    fn log_plugin(&self, record: &Record, plugin_name: &str) {
        use log::Log;

        if self.enabled(record.metadata()) {
            if CONFIG.log().to_terminal() {
                match record.level() {
                    Level::Error => write(&mut *self.stderr.lock().unwrap(), self.start.elapsed(), record, plugin_name),
                    Level::Warn => write(&mut *self.stderr.lock().unwrap(), self.start.elapsed(), record, plugin_name),
                    Level::Info => write(&mut *self.stdout.lock().unwrap(), self.start.elapsed(), record, plugin_name),
                    Level::Debug => write(&mut *self.stdout.lock().unwrap(), self.start.elapsed(), record, plugin_name),
                    Level::Trace => write(&mut *self.stdout.lock().unwrap(), self.start.elapsed(), record, plugin_name),
                }
            }

            if CONFIG.log().to_file() {
                let ref mut sink: Option<Box<Write + Send>> = *self.file.lock().unwrap();
                if sink.is_some() {
                    let sink = sink.as_mut().unwrap();
                    match record.level() {
                        Level::Error => write(sink, self.start.elapsed(), record, plugin_name),
                        Level::Warn => write(sink, self.start.elapsed(), record, plugin_name),
                        Level::Info => write(sink, self.start.elapsed(), record, plugin_name),
                        Level::Debug => write(sink, self.start.elapsed(), record, plugin_name),
                        Level::Trace => write(sink, self.start.elapsed(), record, plugin_name),
                    }
                }
            }
        }
    }
}

impl log::Log for LOGGER {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        self.log_plugin(record, "BEST-Bot")
    }

    fn flush(&self) {
        if CONFIG.log().to_file() {
            let ref mut sink: Option<Box<Write + Send>> = *self.file.lock().unwrap();
            if sink.is_some() {
                let sink = sink.as_mut().unwrap();
                sink.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;

    #[test]
    fn test() {
        let s = init().unwrap();

        log::set_max_level(LevelFilter::Trace);
        error!("error");
        warn!("warn");
        info!("info");
        debug!("debug");
        trace!("trace");

        log::set_max_level(LevelFilter::Trace);
        s.send(("test".to_string(), Log::Error("error".to_string())));
        s.send(("test".to_string(), Log::Warn("warn".to_string())));
        s.send(("test".to_string(), Log::Info("info".to_string())));
        s.send(("test".to_string(), Log::Debug("debug".to_string())));
        s.send(("test".to_string(), Log::Trace("trace".to_string())));

        drop(s);

        thread::sleep(Duration::new(0,10000));
    }
}