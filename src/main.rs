use crate::status::Status;
use clap::{Args, Parser, Subcommand, ValueEnum, error::Result};
use easy_storage::Storeable;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::PathBuf;

const PROGRAM_NAME: &str = "wlmstr";

mod status;

#[derive(Debug)]
enum Error {
    FailedAwww(String),
    Io(std::io::Error),
    NotFoundXDGDATAPATH,
    NotFoundSpecificImage,
    ValueNotSet,
    InsufficientValues,
    SerializeErr(serde_json::Error),
    FileIo(easy_storage::Error),
    NoWallpaperFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO Error: {}", e),
            Error::NotFoundXDGDATAPATH => write!(
                f,
                "XDG_DATA_HOME is not set.\n\nThis program uses XDG_DATA_HOME/{}/ to store its data.\nIf you are not using Linux or a standard XDG-compatible environment,please set the XDG_DATA_HOME environment variable manually.Not dound XDG_DATA_HOME.\nThis Program use XDG_DATA_HOME/{}/ to keep path data.\nIf you don't use Linux or standard envrinment",
                PROGRAM_NAME, PROGRAM_NAME
            ),
            Error::NotFoundSpecificImage => todo!(),
            Error::ValueNotSet => write!(
                f,
                "The wallpaper directory path has not been set.\nPlease run `set` subcommand\nwlmstr set -d <dir-path> -p <start-img-path>)"
            ),
            Error::InsufficientValues => write!(
                f,
                "Insufficient values; values for dir and start img path are all required for initialization."
            ),
            Error::SerializeErr(e) => write!(f, "Serialize Error: {}", e),
            Error::FailedAwww(v) => write!(f, "failed to process awww: {}", v),
            Error::FileIo(e) => write!(f, "File IO Error: {}", e),
            Error::NoWallpaperFound => todo!(),
        }
    }
}

///  Stateful wallpaper slideshow CLI for awww
#[derive(Debug, Parser)]
#[command(version, about, name = PROGRAM_NAME)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Next(Update),
    Status(StatusCmd),
    Set(SetCmd),
}

#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
enum Mode {
    Images,
    Videos,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Mode::Images => "Image",
            Mode::Videos => "Video",
        };
        write!(f, "{}", res)
    }
}

/// go to next slide
#[derive(Debug, Args)]
struct Update {
    derection: Direction,
}

/// Get status. supports to output in JSON or debug format
#[derive(Debug, Args)]
struct StatusCmd {
    /// Default is Debug
    format: Option<StatusFmt>,
}

#[derive(Debug, Clone, ValueEnum)]
enum StatusFmt {
    Json,
    Debug,
}

#[derive(Debug, Clone, ValueEnum)]
enum Direction {
    #[value(name = "seq")]
    Sequence,

    #[value(name = "pre")]
    Previous,

    #[value(name = "rnd")]
    Random,
}

/// set config data
#[derive(Debug, Args)]
struct SetCmd {
    #[arg(short = 'd', long = "dir", help = "specify wallpapers dir")]
    dir: Option<PathBuf>,

    #[arg(short = 'p', long = "path", help = "start iamge path of slides")]
    paper_path: Option<PathBuf>,

    #[arg(short = 'm', long = "mode", help = "mode")]
    mode: Option<Mode>,
}

fn resolve_data_path() -> Result<PathBuf, Error> {
    std::env::var_os("XDG_DATA_HOME")
        .map(|f| {
            PathBuf::from(f.to_string_lossy().to_string())
                .join("wlmstr")
                .join("data.json")
        })
        .ok_or(Error::NotFoundXDGDATAPATH)
}

fn run(cli_cmd: Commands) -> Result<(), Error> {
    let data_path = resolve_data_path()?;

    let new_st = match Status::load_by_extension(&data_path) {
        Ok(st) => match cli_cmd {
            Commands::Next(update) => {
                let list = std::fs::read_dir(st.get_dir_path())
                    .map_err(Error::Io)?
                    .filter_map(|f| f.ok())
                    .map(|f| f.path())
                    .collect::<Vec<_>>();
                st.update(update.derection, list)?
            }
            Commands::Status(sc) => {
                let res = match sc.format.unwrap_or(StatusFmt::Debug) {
                    StatusFmt::Json => {
                        serde_json::to_string_pretty(&st).map_err(Error::SerializeErr)?
                    }
                    StatusFmt::Debug => st.to_string(),
                };
                println!("{}", res);
                return Ok(());
            }
            Commands::Set(input_set_data) => st.set(
                input_set_data.dir,
                input_set_data.paper_path,
                input_set_data.mode,
            )?,
        },
        Err(_) => {
            // when not found XDG_DATA_HOME/wlmstr/data.json
            match cli_cmd {
                Commands::Set(set) => match (set.dir, set.paper_path, set.mode) {
                    (Some(v), Some(w), Some(x)) => Status::new(v, w, x),
                    (Some(v), Some(w), None) => Status::new(v, w, Mode::Images),
                    _ => return Err(Error::InsufficientValues),
                },
                _ => return Err(Error::ValueNotSet),
            }
        }
    };
    new_st
        .save_by_extension(data_path, true)
        .map_err(Error::FileIo)?;

    new_st.apply()?;
    println!(
        "change to {}",
        new_st.get_current_p_path().to_string_lossy()
    );
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    match run(cli.command) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprint!("{}", e);
            std::process::exit(1)
        }
    }
}
