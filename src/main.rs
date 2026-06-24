use clap::{Args, Parser, Subcommand, ValueEnum, error::Result};
use easy_storage::Storeable;
use itertools::Itertools;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::PathBuf;

const PROGRAM_NAME: &str = "wlmstr";

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
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Status {
    dir_path: PathBuf,
    paper_path: PathBuf,
    mode: Mode,
}

impl Storeable for Status {}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "current dir: {}\ncurrent WallPaper: {}\nMode: {}",
            self.dir_path.to_string_lossy(),
            self.paper_path.to_string_lossy(),
            self.mode
        )
    }
}

impl Status {
    fn new(dir_path: PathBuf, paper_path: PathBuf, mode: Mode) -> Self {
        Self {
            dir_path,
            paper_path,
            mode,
        }
    }

    fn set(self, dir: Option<PathBuf>, path: Option<PathBuf>, mode: Option<Mode>) -> Self {
        let res = match (dir, path, mode) {
            (None, None, None) => (self.dir_path, self.paper_path, self.mode),
            (None, None, Some(v)) => (self.dir_path, self.paper_path, v),
            (None, Some(v), None) => (self.dir_path, v, self.mode),
            (None, Some(v), Some(w)) => (self.dir_path, v, w),
            (Some(v), None, None) => (v, self.paper_path, self.mode),
            (Some(v), None, Some(w)) => (v, self.paper_path, w),
            (Some(v), Some(w), None) => (v, w, self.mode),
            (Some(v), Some(w), Some(x)) => (v, w, x),
        };
        Self {
            dir_path: res.0,
            paper_path: res.1,
            mode: res.2,
        }
    }

    fn update(self, derection: Derection, file_list: Vec<PathBuf>) -> Result<Status, Error> {
        let next = match file_list
            .iter()
            .sorted()
            .position(|x| *x == self.paper_path)
        {
            Some(pos) => match derection {
                Derection::Sequence => file_list.get(pos + 1),
                Derection::Previous => file_list.get(pos - 1),
                Derection::Random => {
                    let mut rng = rand::rng();
                    file_list.choose(&mut rng)
                }
            },
            _ => None,
        }
        .unwrap_or(&self.paper_path);

        if !self.paper_path.exists() {
            return Err(Error::NotFoundSpecificImage);
        }

        Ok(Self {
            dir_path: self.dir_path,
            paper_path: next.to_path_buf(),
            mode: self.mode,
        })
    }

    fn apply(&self) -> Result<(), Error> {
        let path = self.paper_path.to_string_lossy().to_string();
        let res = match self.mode {
            Mode::Images => std::process::Command::new("awww")
                .args([
                    "img",
                    &path,
                    "--transition-type",
                    "center",
                    "--transition-duration",
                    "0.5",
                ])
                .output(),
            Mode::Videos => {
                let mpv_args = format!("--mpv-args=\"{}\"", "--hwdec=auto-safe --panscan=1.0");
                std::process::Command::new("mpbpaper")
                    .args([
                        "*",
                        self.paper_path.to_string_lossy().as_ref(),
                        "-o",
                        "no-audio loop",
                        "--fork",
                        "-o",
                        "no-audio loop --panscan=1.0",
                        &mpv_args,
                    ])
                    .output()
            }
        };

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::FailedAwww(e.to_string())),
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
    derection: Derection,
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
enum Derection {
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
                let list = std::fs::read_dir(&st.dir_path)
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
            Commands::Set(init) => st.set(init.dir, init.paper_path, init.mode),
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
    println!("change to {}", new_st.paper_path.to_string_lossy());
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
