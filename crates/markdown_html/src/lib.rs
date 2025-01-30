pub mod template;
pub mod config;

use lazy_static::lazy_static;
use tera::Tera;
use std::env;
use std::path::PathBuf;

pub fn find_root() -> Option<PathBuf> {
    let mut current_dir = env::current_dir().ok()?;

    loop {
        let config_path = current_dir.join("diary.toml");
        if config_path.exists() {
            return Some(current_dir);
        }
        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        }
    }
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        // temporarily hardcoding the root directory path
        let root_dir = find_root().unwrap();
        Tera::new(root_dir.join("templates/**/*.html").to_str().unwrap()).unwrap()
    };
}
