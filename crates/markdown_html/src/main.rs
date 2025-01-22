use clap::Parser;

mod cli;
#[cfg(feature = "serve")]
mod serve;

use cli::{Args, Command};
use markdown_html::convert::markdown_to_html;
use serve::serve;
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

fn visit_dirs<F, E>(dir: &Path, cb: &F) -> io::Result<()>
where 
    F: Fn(&DirEntry) -> Result<(), E>,
    E: Into<io::Error>,
{
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry).map_err(Into::into)?;
            }
        }
    }
    Ok(())
}

fn build(input_dir: &Path, output_dir: &Path) -> io::Result<()> {
    visit_dirs(input_dir, &|entry: &DirEntry| {
        if entry.path().extension() != Some("md".as_ref()) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid file extension"));
        }

        // should not panic because entry path is always in input_dir
        let mut output_path = output_dir.join(entry.path().strip_prefix(input_dir).unwrap());
        output_path.set_extension("html");

        if let Err(error) = markdown_to_html(&entry.path(), &output_path) {
            eprintln!("Could not render {:?}: {}", entry.path(), error)
        }
        Ok(())
    })
    .unwrap();
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.command {
        Command::Build => build(&args.input_dir, &args.output_dir).unwrap(),
        Command::Serve => serve(&args.input_dir, &args.output_dir).await.unwrap(),
    }
}
