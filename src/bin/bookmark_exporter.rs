use core::fmt::Arguments;
use bookmark_exporter::{error, BookmarkExporterLog, BookmarkExporterTool};
use yansi::Paint;

struct BookmarkExporterLogger;

impl BookmarkExporterLogger {
    fn new() -> BookmarkExporterLogger {
        BookmarkExporterLogger {}
    }
}

impl BookmarkExporterLog for BookmarkExporterLogger {
    fn output(self: &Self, args: Arguments) {
        println!("{}", args);
    }
    fn warning(self: &Self, args: Arguments) {
        eprintln!("{}", Paint::yellow(&format!("warning: {}", args)));
    }
    fn error(self: &Self, args: Arguments) {
        eprintln!("{}", Paint::red(&format!("error: {}", args)));
    }
}

fn main() {
    let logger = BookmarkExporterLogger::new();

    if let Err(error) = BookmarkExporterTool::new(&logger).run(std::env::args_os()) {
        error!(logger, "{}", error);
        std::process::exit(1);
    }
}
