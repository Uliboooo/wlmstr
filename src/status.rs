use easy_storage::Storeable;
use itertools::Itertools;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

use crate::{Direction, Error, Mode};

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
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
    pub fn new(dir_path: PathBuf, paper_path: PathBuf, mode: Mode) -> Self {
        Self {
            dir_path,
            paper_path,
            mode,
        }
    }

    pub fn get_current_p_path(&self) -> PathBuf {
        self.paper_path.to_path_buf()
    }

    pub fn set(self, dir: Option<PathBuf>, path: Option<PathBuf>, mode: Option<Mode>) -> Self {
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
            mode: self.mode,
        })
    }

    pub fn apply(&self) -> Result<(), Error> {
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
