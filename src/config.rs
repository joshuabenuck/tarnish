extern crate toml;

use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Trove {
    pub root: PathBuf,
}

#[derive(Deserialize)]
pub struct System {
    pub downloads: PathBuf,
    pub cache: PathBuf,
}

#[derive(Deserialize)]
pub struct Config {
    pub trove: Trove,
    pub system: System,
}

impl Config {
    pub fn new(path: &str) -> Config {
        let mut contents = String::new();
        File::open(path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        return toml::from_str(contents.as_str()).unwrap();
    }
}
