use crate::{Direction, Error};
use easy_storage::Storeable;
use itertools::Itertools;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

const SUPPURTED_VIDEO_TYPES: [&str; 12] = [
    "mp4", "mkv", "webp", "avi", "mov", "flv", "m4v", "ts", "m2ts", "git", "apng", "webp",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    dir_path: PathBuf,
    paper_path: PathBuf,
    // mode: Mode,
}

impl Storeable for Status {}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "current dir: {}\ncurrent WallPaper: {}\n",
            self.dir_path.to_string_lossy(),
            self.paper_path.to_string_lossy(),
            // self.mode
        )
    }
}

impl Status {
    pub fn new(dir_path: PathBuf, paper_path: PathBuf) -> Self {
        Self {
            dir_path,
            paper_path,
            // mode,
        }
    }

    pub fn get_current_p_path(&self) -> PathBuf {
        self.paper_path.to_path_buf()
    }

    pub fn get_dir_path(&self) -> PathBuf {
        self.dir_path.to_path_buf()
    }

    pub fn set(
        self,
        dir: Option<PathBuf>,
        path: Option<PathBuf>,
        // mode: Option<Mode>,
    ) -> Result<Self, Error> {
        let res = match (dir, path) {
            (None, None) => (self.dir_path, self.paper_path),
            (None, Some(path)) => (self.dir_path, path),
            (Some(dir), None) => {
                let pp = Self::get_first_paper_in_dir(&dir)?;
                (dir, pp)
            }
            (Some(dir), Some(path)) => (dir, path),
        };
        Ok(Self {
            dir_path: res.0,
            paper_path: res.1,
            // mode: res.2,
        })
    }

    // fn paper_exists(&self) -> Result<bool, std::io::Error> {
    //     Ok(std::fs::read_dir(&self.dir_path)?
    //         .filter_map(|f| f.ok())
    //         .map(|f| f.path())
    //         .any(|f| f == self.paper_path))
    // }

    pub fn get_first_paper_in_dir<P: AsRef<Path>>(dir: P) -> Result<PathBuf, Error> {
        let res = std::fs::read_dir(&dir)
            .map_err(Error::Io)?
            // .unwrap()
            .next()
            .ok_or(Error::NoWallpaperFound)?
            .map(|f| f.path())
            // .unwrap()
            .map_err(Error::Io)
            .unwrap();
        Ok(res)
    }

    pub fn update(self, derection: Direction, file_list: Vec<PathBuf>) -> Result<Status, Error> {
        let sorted = file_list
            .iter()
            .sorted_by_key(|p| p.file_name().unwrap().to_os_string())
            .collect::<Vec<_>>();

        if sorted.is_empty() {
            return Err(Error::NoWallpaperFound);
        }

        let next = match sorted.iter().position(|x| **x == self.paper_path) {
            Some(pos) => match derection {
                Direction::Sequence => {
                    let next = (pos + 1) % sorted.len();
                    sorted.get(next)
                }
                Direction::Previous => {
                    let prev = if pos == 0 { sorted.len() - 1 } else { pos - 1 };
                    sorted.get(prev)
                }
                // pos.checked_sub(1).and_then(|p| sorted.get(p))},
                Direction::Random => {
                    let mut rng = rand::rng();
                    sorted.choose(&mut rng)
                }
            },
            None => None,
        }
        .unwrap_or(&&self.paper_path)
        .to_path_buf();

        if !self.paper_path.exists() {
            return Err(Error::NotFoundSpecificImage);
        }

        Ok(Self {
            dir_path: self.dir_path,
            paper_path: next,
            // mode: self.mode,
        })
    }

    pub fn apply(&self) -> Result<(), Error> {
        let path = self.paper_path.to_string_lossy().to_string();
        let is_image = {
            PathBuf::from(&path)
                .extension()
                .and_then(|f| f.to_str())
                .map(|f| SUPPURTED_VIDEO_TYPES.iter().any(|t| t == &f))
        }
        .unwrap_or(false);

        let res = if is_image {
            std::process::Command::new("awww")
                .args([
                    "img",
                    &path,
                    "--transition-type",
                    "center",
                    "--transition-duration",
                    "0.5",
                ])
                .output()
        } else {
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
        };

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::FailedAwww(e.to_string(), path.to_string())),
        }
    }
}
