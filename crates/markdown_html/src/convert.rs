use crate::TEMPLATES;
use pulldown_cmark::Options;
use std::fs;
use std::io;
use std::path::Path;
use tera::Context;

pub fn markdown_to_html(input_path: &Path, output_path: &Path) -> io::Result<String> {
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
    let mut context = Context::new();
    context.insert("content", &html_buf);
    let output = match TEMPLATES.render("base.html", &context) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("failed to rendering template: {}", error);
            return Err(io::Error::new(io::ErrorKind::Other, "could not rendering template"))
        }
    };
    fs::write(output_path, &output)?;
    Ok(output)
}
