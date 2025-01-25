use clap::Parser;

mod cli;
#[cfg(feature = "serve")]
mod serve;

use cli::{Args, Command};
use markdown_html::find_root;
use markdown_html::convert::markdown_to_html;
use markdown_html::config::{Config, Dirs};
use serve::serve;
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

fn visit_dirs<F, E>(dir: &Path, cb: &F) -> io::Result<()>
where 
    F: Fn(&DirEntry) -> Result<(), E>,
    E: Into<io::Error>,
{
    if !dir.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            format!("directory expected, got file {}", dir.display())));
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dirs(&path, cb)?;
        } else {
            cb(&entry).map_err(Into::into)?;
        }
    }
    Ok(())
}

fn build(dirs: &Dirs) -> io::Result<()> {
    visit_dirs(&dirs.src, &|entry: &DirEntry| {
        let path = entry.path();
        if path.extension() != Some("md".as_ref()) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid file extension: {:?}", path)));
        }

        // should not panic because entry path is always in input_dir
        let mut output_path = dirs.build_dir.join(path.strip_prefix(&dirs.src).unwrap());
        output_path.set_extension("html");

        if let Err(error) = markdown_to_html(&path, &output_path) {
            eprintln!("Could not render {:?}: {}", path, error)
        }
        Ok(())
    })
    .unwrap();
    Ok(())
}

#[tokio::main]
async fn main() {
    let root_dir = find_root().expect("could not find the root directory of the diary");
    let config_dir = root_dir.join("diary.toml");
    if !config_dir.exists() {
        unimplemented!("create default configuration file")
    }
    let config = fs::read_to_string(config_dir).unwrap();
    let config: Config = toml::from_str(&config).unwrap();

    let args = Args::parse();
    println!("{:?} {:?} {:?}", args.dir, root_dir, config.diary.src);
    let dirs = Dirs {
        src: args.dir.unwrap_or(root_dir.join(config.diary.src)),
        build_dir: args.dest_dir.unwrap_or(root_dir.join(config.build.build_dir)),
    };

    println!("{:?} {:?}", dirs.src, dirs.build_dir);

    match args.command {
        Command::Build => build(&dirs).unwrap(),
        Command::Serve => serve(&dirs).await.unwrap(),
    }
}
