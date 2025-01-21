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

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn build(input_dir: &Path, output_dir: &Path) {
    visit_dirs(input_dir, &|entry: &DirEntry| {
        if let Some(extension) = entry.path().extension() {
            if extension != "md" {
                return;
            }

            let mut output_path = output_dir.join(entry.path().strip_prefix(input_dir).unwrap());
            output_path.set_extension("html");

            println!("{:?} {:?}", entry.path(), output_path);
            markdown_to_html(&entry.path(), &output_path).unwrap();
        }
    })
    .unwrap();
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.command {
        Command::Build => build(&args.input_dir, &args.output_dir),
        Command::Serve => serve(&args.input_dir, &args.output_dir).await.unwrap(),
    }
}
