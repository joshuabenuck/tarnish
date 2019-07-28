use std::path::{PathBuf};
extern crate sha2;
use sha2::{Digest};
use std::io::{Read, Write};
use std::fs::{File, self};
extern crate log;
use log::{debug, error};

fn sha256(url: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.input(url.as_bytes());
    hex::encode(&hasher.result())
}

pub struct Cache {
    root: PathBuf,
}

impl Cache {
    pub fn new<T: Into<PathBuf>>(root: T) -> Cache {
        let cache = Cache { root: root.into() };
        if ! cache.root.exists() {
            debug!("creating: {}", cache.root.display());
            if let Err(result) = fs::create_dir_all(&cache.root) {
                error!("error: {:?}", result);
            }
        }
        return cache;
    }

    pub fn retrieve(&self, url: &str) -> String {
        let hash = sha256(url);
        let cached = self.root.join(&hash);
        debug!("{:?}", hash);
        if ! cached.exists() {
            // TODO: Add cache expiration
            debug!("caching: {}", url);
            let mut resp = reqwest::get(url).unwrap();
            assert!(resp.status().is_success());
            let text = resp.text().unwrap();
            File::create(self.root.join(format!("{}.url", &hash))).unwrap().write(url.as_bytes()).unwrap();
            File::create(cached).unwrap().write(text.as_bytes()).unwrap();
            text
        }
        else {
            debug!("using cached value: {}", &url);
            let mut contents = String::new();
            File::open(cached).unwrap().read_to_string(&mut contents).unwrap();
            contents
        }
    }
}


