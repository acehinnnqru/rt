use log::{Level, Metadata, Record, debug};

struct AgrmLogger;

impl log::Log for AgrmLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init(level: Level) {
    log::set_boxed_logger(Box::new(AgrmLogger))
        .map(|()| log::set_max_level(level.to_level_filter()))
        .unwrap();
    debug!("Logger initialized: {}", level);
}
