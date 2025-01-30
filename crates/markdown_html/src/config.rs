use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub diary: Diary,
    pub build: Build,
}

#[derive(Serialize, Deserialize)]
pub struct Diary {
    pub src: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct Build {
    #[serde(rename = "build-dir")]
    pub build_dir: PathBuf,
}

// Runtime type for avoiding excessive use of unwraps
#[derive(Clone)]
pub struct Dirs {
    pub src: PathBuf,
    pub build_dir: PathBuf,
}
