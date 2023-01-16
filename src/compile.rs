use std::{error::Error, fs, path::Path};

use regex::Regex;

use crate::{case, convert, utils::separate_filename_ext, Config, MyError};

pub fn compile(config: Config) -> Result<(), Box<dyn Error>> {
    // Remove build directory recursively if exists
    if Path::new(&config.files.build).exists() {
        fs::remove_dir_all(&config.files.build)?;
    }
    // Create new build directory
    fs::create_dir(&config.files.build)?;

    // Template file
    let template_html = if Path::new(&config.files.template).exists() {
        Some(fs::read_to_string(&config.files.template)?)
    } else {
        None
    };

    let mut files: Vec<(String, String)> = Vec::new();

    // ? Merge these 2 loops ?
    for entry in fs::read_dir(&config.files.source)?.flatten() {
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
        files.push((filename, fs::read_to_string(entry.path())?));
    }

    for (filepath, file) in &mut files {
        let (filepath_no_ext, ext) = separate_filename_ext(filepath);

        match ext {
            "ling" => {
                *file = use_template_html(convert::ling_to_html(file), &template_html, &config);
                *filepath = filepath_no_ext + ".html";
            }

            _ => return Err(Box::new(MyError("Unknown file type".to_string()))),
        }
        fs::write(format!("{}/{}", config.files.build, filepath), file)?;
    }

    Ok(())
}

fn use_template_html(file: String, template: &Option<String>, config: &Config) -> String {
    if let Some(template) = template {
        //TODO Use lazy_static for regex
        // ? Use replace_all ?
        let html = Regex::new(r"\{\$\s*BODY\s*\}")
            .unwrap()
            .replace(template, &file)
            .to_string();
        let html = Regex::new(r"\{\$\s*TITLE\s*\}")
            .unwrap()
            //TODO Fix title
            .replace(&html, case::upper_first(&config.package.name))
            .to_string();
        html
    } else {
        file
    }
}
