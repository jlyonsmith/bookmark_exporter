mod log_macros;

use anyhow::{bail, Context, Result};
use clap::Parser;
use core::fmt::Arguments;
use glob::glob;
use std::{
    env,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub trait BookmarkExporterLog {
    fn output(self: &Self, args: Arguments);
    fn warning(self: &Self, args: Arguments);
    fn error(self: &Self, args: Arguments);
}

pub struct BookmarkExporterTool<'a> {
    log: &'a dyn BookmarkExporterLog,
}

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Disable colors in output
    #[arg(long = "no-color", short = 'n', env = "NO_CLI_COLOR")]
    no_color: bool,

    /// The output file
    #[arg(value_name = "OUTPUT_FILE")]
    output_file: Option<PathBuf>,
}

impl Cli {
    fn get_output(&self) -> anyhow::Result<Box<dyn Write>> {
        match self.output_file {
            Some(ref path) => File::create(path)
                .context(format!(
                    "Unable to create file '{}'",
                    path.to_string_lossy()
                ))
                .map(|f| Box::new(f) as Box<dyn Write>),
            None => Ok(Box::new(io::stdout())),
        }
    }
}

impl<'a> BookmarkExporterTool<'a> {
    pub fn new(log: &'a dyn BookmarkExporterLog) -> BookmarkExporterTool {
        BookmarkExporterTool { log }
    }

    pub fn run(self: &mut Self, args: impl IntoIterator<Item = std::ffi::OsString>) -> Result<()> {
        let cli = match Cli::try_parse_from(args) {
            Ok(m) => m,
            Err(err) => {
                output!(self.log, "{}", err.to_string());
                return Ok(());
            }
        };

        let content = self.export_firefox_bookmarks()?;

        write!(cli.get_output()?, "{}", content)?;

        Ok(())
    }

    pub fn export_firefox_bookmarks(&self) -> Result<String> {
        let path: PathBuf = [
            env::var("HOME")?.as_ref(),
            Path::new("Library/Application Support/Firefox/Profiles/*.default-release"),
        ]
        .iter()
        .collect();

        let profile_dir_path;

        if let Some(entry) = glob(&path.to_string_lossy())?.next() {
            profile_dir_path = entry?;
        } else {
            bail!("A Firefox profile directory was not found");
        }

        let places_path: PathBuf = [Path::new(&profile_dir_path), Path::new("places.sqlite")]
            .iter()
            .collect();
        let connection = sqlite::open(places_path)?;
        let query = "select '[' || moz_bookmarks.title || '](' || url || ')' from moz_bookmarks left join moz_places on fk = moz_places.id where url <> '' and moz_bookmarks.title <> '';";

        for row in connection
            .prepare(query)?
            .into_iter()
            .map(|row| row.unwrap())
        {
            println!("{}", row.read::<&str, _>(0));
        }

        Ok("".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        struct TestLogger;

        impl TestLogger {
            fn new() -> TestLogger {
                TestLogger {}
            }
        }

        impl BookmarkExporterLog for TestLogger {
            fn output(self: &Self, _args: Arguments) {}
            fn warning(self: &Self, _args: Arguments) {}
            fn error(self: &Self, _args: Arguments) {}
        }

        let logger = TestLogger::new();
        let mut tool = BookmarkExporterTool::new(&logger);
        let args: Vec<std::ffi::OsString> = vec!["".into(), "--help".into()];

        tool.run(args).unwrap();
    }
}
