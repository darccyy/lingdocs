use std::{error::Error, fs, path::Path};

use regex::Regex;

use crate::{case, convert, utils::separate_filename_ext, Config, MyError};

pub fn compile(config: Config) -> Result<(), Box<dyn Error>> {
    // Remove build directory recursively if exists
    if Path::new(&config.files.build).exists() {
        fs::remove_dir_all(&config.files.build).expect("Could not delete build directory");
    }
    // Create new build directory
    fs::create_dir(&config.files.build).expect("Could not create build directory");

    // Template file
    let path = format!("{}/{}", config.files.source, config.files.template);
    let template_html = if Path::new(&path).exists() {
        Some(fs::read_to_string(&path).expect("Could not read template file"))
    } else {
        None
    };

    // Convert scss to css
    if let Some(filepath) = &config.files.style {
        let scss = fs::read_to_string(format!("{}/{}", config.files.source, filepath))
            .expect("Could not read scss file");
        let css = grass::from_string(scss, &grass::Options::default())?;

        // Minify
        let css = if config.options.minify {
            minify_css(&css)?
        } else {
            css
        };

        // Minify css
        let (filepath_no_ext, _) = separate_filename_ext(filepath);
        fs::write(
            format!("{}/{}.css", config.files.build, filepath_no_ext),
            css,
        )
        .expect("Could not write css file");
    }

    let mut files: Vec<(String, String)> = Vec::new();
    for entry in fs::read_dir(&config.files.source).expect("Could not read source directory").flatten() {
        // Throw if not file
        if !entry.path().is_file() {
            return Err(Box::new(MyError(
                "Should not be folder in source directory".to_string(),
            )));
        }

        // Get file name
        //TODO Fix this
        let filename = match entry.path().file_name() {
            Some(x) => x.to_string_lossy().to_string(),
            None => continue,
        };

        // Add file to list
        files.push((filename, fs::read_to_string(entry.path()).expect("Could not read source file")));
    }

    for (filepath, file) in &mut files {
        let (filepath_no_ext, ext) = separate_filename_ext(filepath);

        match ext {
            "ling" => {
                *filepath = filepath_no_ext + ".html";

                *file = use_template_html(convert::ling_to_html(file), &template_html, &config);
                if config.options.minify {
                    *file = minify_html(file);
                }
            }

            //TODO
            "ldct" => continue,
            "llst" => continue,

            "html" | "css" | "scss" => continue,

            _ => return Err(Box::new(MyError("Unknown file type".to_string()))),
        }
        
        fs::write(format!("{}/{}", config.files.build, filepath), file).expect("Could not write build file");
    }

    Ok(())
}

fn minify_css(file: &str) -> Result<String, Box<dyn Error>> {
    use css_minify::optimizations::{Level, Minifier};

    match Minifier::default().minify(&file, Level::Two) {
        Ok(x) => Ok(x),
        Err(err) => Err(Box::new(MyError(err.to_string()))),
    }
}

fn minify_html(file: &str) -> String {
    use minify_html::{minify, Cfg};

    String::from_utf8_lossy(&minify(
        &file.as_bytes(),
        &Cfg {
            do_not_minify_doctype: true,
            ..Cfg::default()
        },
    ))
    .to_string()
}

fn use_template_html(file: String, template: &Option<String>, config: &Config) -> String {
    if let Some(template) = template {
        //TODO Use lazy_static for regex
        let html = Regex::new(r"\{\$\s*BODY\s*\}")
            .unwrap()
            .replace_all(template, &file)
            .to_string();
        let html = Regex::new(r"\{\$\s*TITLE\s*\}")
            .unwrap()
            .replace_all(&html, case::upper_first(&config.package.name))
            .to_string();
        html
    } else {
        file
    }
}
