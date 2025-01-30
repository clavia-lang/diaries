use crate::TEMPLATES;

use pulldown_cmark::Options;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tera::Context;
use serde::Serialize;
use crate::config::Dirs;

#[derive(Serialize)]
struct Diary<'a> {
    content: &'a String,
}

#[derive(Serialize)]
struct DiaryItem {
    permalink: PathBuf,
}

pub fn render_diary(input_path: &Path, output_path: &Path) -> io::Result<String> {
    println!("rendering diary {:?} {:?}", input_path, output_path);

    let markdown_input = fs::read_to_string(input_path)?;
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    let parser = pulldown_cmark::Parser::new_ext(&markdown_input, options);
    let mut html_buf = String::new();
    pulldown_cmark::html::push_html(&mut html_buf, parser);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let diary = Diary {
        content: &html_buf,
    };
    let mut context = Context::new();
    context.insert("diary", &diary);
    let output = match TEMPLATES.render("diary.html", &context) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("failed to rendering page: {}", error);
            return Err(io::Error::new(io::ErrorKind::Other, "could not rendering page"))
        }
    };
    fs::write(output_path, &output)?;
    Ok(output)
}

pub fn render_index(input_path: &Path, output_path: &Path, dirs: &Dirs) -> io::Result<String> {
    println!("rendering index {:?} {:?}", input_path, output_path);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut diaries = Vec::new();
    for entry in fs::read_dir(input_path)? {
        let entry = entry?;
        let permalink = entry.path().strip_prefix(&dirs.src).unwrap().with_extension("");

        let diary = DiaryItem {
            permalink,
        };
        diaries.push(diary);
    }
    let mut context = Context::new();
    context.insert("diaries", &diaries);
    let output = match TEMPLATES.render("index.html", &context) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("failed to rendering index: {}", error);
            return Err(io::Error::new(io::ErrorKind::Other, "could not rendering index"))
        }
    };
    fs::write(output_path, &output)?;
    Ok(output)
}
