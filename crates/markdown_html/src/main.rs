use clap::Parser;

mod cli;
#[cfg(feature = "serve")]
mod serve;

use cli::{Args, Command};
use markdown_html::find_root;
use markdown_html::template::{render_diary, render_index};
use markdown_html::config::{Config, Dirs};
use serve::serve;
use std::fs;
use std::io;
use std::path::Path;

fn visit_dirs<F, E>(dir: &Path, cb: &F) -> io::Result<()>
where 
    F: Fn(&Path) -> Result<(), E>,
    E: Into<io::Error>,
{
    if !dir.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            format!("directory expected, got file {}", dir.display())));
    }
    cb(dir).map_err(Into::into)?;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        cb(&path).map_err(Into::into)?;
        if path.is_dir() {
            visit_dirs(&path, cb)?;
        }
    }
    Ok(())
}

fn build(dirs: &Dirs) -> io::Result<()> {
    visit_dirs(&dirs.src, &|input_path: &Path| {
        // should not panic because entry path is always in input_dir
        let path = input_path.strip_prefix(&dirs.src).unwrap();

        if input_path.is_file() {
            if input_path.extension() != Some("md".as_ref()) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid file extension: {:?}", path)));
            }

            let mut output_path = dirs.build_dir.join(path);
            output_path.set_extension("html");

            if let Err(error) = render_diary(input_path, &output_path) {
                eprintln!("Could not render diary {:?}: {}", input_path, error);
            }
        } else if input_path.is_dir() {
            let output_path = dirs.build_dir.join(path).join("index.html");
            if let Err(error) = render_index(input_path, &output_path, dirs) {
                eprintln!("Could not render index {:?}: {}", input_path, error);
            }
        } else {
            unreachable!("{:?} is nether a file nor a directory", path);
        }
        Ok(())
    })?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let root_dir = find_root().expect("could not find the root directory of the diary");
    let config_dir = root_dir.join("diary.toml");
    // In the future, this will allow specifying the configuration file path via
    // command line arguments. Currently, the find_root function is used to
    // locate the directory that contains the configuration files, so the
    // condition will never be reached.
    if !config_dir.exists() {
        unimplemented!("create default configuration file")
    }
    let config = fs::read_to_string(config_dir).unwrap();
    let config: Config = toml::from_str(&config).unwrap();

    let args = Args::parse();
    let dirs = Dirs {
        src: args.dir.unwrap_or(root_dir.join(config.diary.src)),
        build_dir: args.dest_dir.unwrap_or(root_dir.join(config.build.build_dir)),
    };

    match args.command {
        Command::Build => build(&dirs).unwrap(),
        Command::Serve => serve(dirs).await.unwrap(),
    }
}
