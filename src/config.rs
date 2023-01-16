use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub package: Package,
    #[serde(default)]
    pub files: Files,
    #[serde(default)]
    pub options: Options,
}

impl Config {
    pub fn from(file: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(file)
    }
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub code: String,
    pub author: Option<String>,
    #[serde(default)]
    pub translations: Vec<(String, String)>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Options {
    pub minify: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { minify: true }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Files {
    pub build: String,
    pub source: String,
    pub main: String,
    pub phoner: String,
    pub dict: String,
    pub template: String,
}

impl Default for Files {
    fn default() -> Self {
        Files {
            build: String::from("./build/"),
            source: String::from("./assets/"),
            main: String::from("main.ling"),
            phoner: String::from("phoner"),
            dict: String::from("dict.dlst"),
            template: String::from("template.html"),
        }
    }
}
