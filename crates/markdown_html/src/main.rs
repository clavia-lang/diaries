use clap::Parser;

mod cli;

use cli::Args;
use lazy_static::lazy_static;
use pulldown_cmark::Options;
use tera::{Tera, Context,};
use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        Tera::new("templates/**/*.html").unwrap()
    };
}

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

fn markdown_to_html(path: &Path, output_path: &Path) -> io::Result<String> {
    let markdown_input = fs::read_to_string(path)?;
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    let parser = pulldown_cmark::Parser::new(&markdown_input);
    let mut html_buf = String::new();
    pulldown_cmark::html::push_html(&mut html_buf, parser);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut context = Context::new();
    context.insert("content", &html_buf);
    let output = TEMPLATES.render("base.html", &context).unwrap();
    fs::write(output_path, &output)?;
    Ok(output)
}

fn main() {
    let args = Args::parse();
    visit_dirs(Path::new(&args.input), &|entry: &DirEntry| {
        if let Some(extension) = entry.path().extension() {
            if extension != "md" {
                return;
            }

            let mut output_path = args.output.join(entry.path().strip_prefix(&args.input).unwrap());
            output_path.set_extension("html");

            println!("{:?} {:?}", entry.path(), output_path);
            markdown_to_html(&entry.path(), &output_path).unwrap();
        }
    }).unwrap();
}
