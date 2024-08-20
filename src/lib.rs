mod log_macros;

use anyhow::{bail, Context, Result};
use clap::Parser;
use core::fmt::Arguments;
use glob::glob;
use json::JsonValue;
use std::{
    env,
    fs::{self, File},
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

        write!(cli.get_output()?, "{}", self.export_firefox_bookmarks()?)?;

        write!(cli.get_output()?, "{}", self.export_chrome_bookmarks()?)?;

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

        let places_path: PathBuf = [&profile_dir_path, Path::new("places.sqlite")]
            .iter()
            .collect();
        let connection = sqlite::open(places_path)?;
        let query = "select '[' || moz_bookmarks.title || '](' || url || ')' from moz_bookmarks left join moz_places on fk = moz_places.id where url <> '' and moz_bookmarks.title <> '';";
        let mut output = String::new();

        for row in connection
            .prepare(query)?
            .into_iter()
            .map(|row| row.unwrap())
        {
            output.push_str(&format!("{}\n", row.read::<&str, _>(0)));
        }

        Ok(output)
    }

    pub fn export_chrome_bookmarks(&self) -> Result<String> {
        let mut bookmarks_path = PathBuf::from(env::var("HOME")?);

        bookmarks_path.push("Library/Application Support/Google/Chrome/Default/Bookmarks");

        let json = json::parse(&fs::read_to_string(bookmarks_path)?)?;

        fn extract_children<'a>(value: &'a JsonValue) -> Option<Vec<&'a JsonValue>> {
            if !value.is_null() && value.is_object() {
                let children = &value["children"];

                if children.is_array() {
                    let mut entries: Vec<&JsonValue> = vec![];

                    for child in children.members() {
                        if let Some(mut child_values) = extract_children(child) {
                            entries.append(&mut child_values);
                        }
                    }

                    return Some(entries);
                }
            }

            return None;
        }

        let mut entries: Vec<&JsonValue> = vec![];

        if !json["root"].is_null() {
            let bookmarks_bar = &json["bookmarks_bar"];

            if bookmarks_bar.is_object() {
                if let Some(mut child_values) = extract_children(&bookmarks_bar["children"]) {
                    entries.append(&mut child_values);
                }
            }
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
