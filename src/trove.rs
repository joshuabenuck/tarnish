use crate::cache::Cache;
use crate::config::Config;
use crate::trove_feed::{Feed, Product};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
//use serde_json::Value::{Array, Object};
use log::warn;
use select::document::Document;
use select::predicate::Attr; //, Class, Name, Predicate, Element};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Data {
    pub machine_name: String,
    pub human_name: String,
    pub description: String,
    pub downloaded: bool, // eventually HashMap
    pub installed: bool,
    pub executable: PathBuf,
    pub downloads: HashMap<String, PathBuf>,
    pub screenshots: Vec<String>,
    pub trailer: Option<String>,
    pub last_seen_on: String,
    pub removed_from_trove: bool,
}

pub struct Trove {
    pub feed: Feed,
    pub cache: PathBuf,
    pub downloads: PathBuf,
    pub root: PathBuf,
    pub number_downloaded: u32,
    pub total: u32,
    //pub games: HashMap<String, Data>,
    //pub installed_games: Vec<String>,
    //pub downloaded_games: Vec<String>,
    //pub not_downloaded_games: Vec<String>,
}

fn get_page(cache: &Cache, index: u8) -> Vec<Product> {
    let text = cache.retrieve(
        format!(
            "https://www.humblebundle.com/api/v1/trove/chunk?index={}",
            index
        )
        .as_str(),
    );
    let chunk: Vec<Product> = serde_json::from_str(text.as_str()).unwrap();
    chunk
}

fn get_products(cache: &Cache) -> Vec<Product> {
    let mut products = Vec::new();
    for i in 0..5 {
        products.extend_from_slice(&get_page(cache, i));
    }
    products
}

impl Trove {
    pub fn new(config: &Config) -> Trove {
        let cache = Cache::new(&config.system.cache);
        let text = cache.retrieve("https://www.humblebundle.com/monthly/trove");
        let doc = Document::from(text.as_str());
        let data = doc
            .find(Attr("id", "webpack-monthly-trove-data"))
            .next()
            .unwrap()
            .text();
        let mut data: Feed = serde_json::from_str(data.as_str()).unwrap();
        data.standard_products = get_products(&cache);
        data.standard_products.sort_by_key(|p| p.date_added);
        data.standard_products.reverse();

        let mut trove = Trove {
            feed: data,
            downloads: config.system.downloads.clone(),
            cache: config.system.cache.clone(),
            root: config.trove.root.clone(),
            number_downloaded: 0,
            total: 0,
        };
        assert!(trove.root.exists());
        trove.update_download_status();
        trove.feed.standard_products.iter().for_each(|product| {
            cache.retrieve(&product.image);
        });
        println!(
            "Downloaded: {}; Total: {}",
            &trove.number_downloaded, &trove.total
        );
        trove
    }

    pub fn update_download_status(&mut self) {
        let mut count = 0;
        for product in &mut self.feed.standard_products {
            let installer = Path::new(&product.downloads["windows"].url.web)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            product.downloaded = self.root.join(installer).exists();
            if product.downloaded {
                count += 1;
            }
        }
        self.number_downloaded = count;
        self.total = self.feed.standard_products.len() as u32;
    }

    pub fn downloaded(&self) -> Vec<&Product> {
        (&self.feed.standard_products)
            .iter()
            .filter(|p| p.downloaded)
            .collect()
    }

    pub fn not_downloaded(&self) -> Vec<&Product> {
        (&self.feed.standard_products)
            .iter()
            .filter(|p| !p.downloaded)
            .collect()
    }

    pub fn format(&self, p: &Product) -> String {
        format!("{} {} {}", p.date_added, p.human_name, p.downloaded)
    }

    pub fn stray_downloads(&self) -> Vec<PathBuf> {
        let downloads = Path::new(&self.downloads);
        assert!(downloads.exists());
        (&self.feed.standard_products)
            .iter()
            .filter_map(|product| {
                let installer = Path::new(&product.downloads["windows"].url.web)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap();
                let full_installer_path = downloads.join(&installer);
                match full_installer_path.exists() {
                    true => Some(full_installer_path),
                    false => None,
                }
            })
            .collect()
    }

    pub fn move_downloads(&self) -> Vec<PathBuf> {
        self.stray_downloads()
            .iter()
            .filter_map(|download| {
                let dest = self.root.join(download.file_name().unwrap());
                println!(
                    "Moving {} to {}.",
                    download.to_str().unwrap(),
                    dest.to_str().unwrap()
                );
                if dest.exists() {
                    warn!("exists, skipping: {}", dest.to_str().unwrap());
                    return Some(download);
                }
                let result = fs::copy(download, &dest);
                match result {
                    Err(e) => {
                        warn!("{}: {}", e, dest.to_str().unwrap());
                        Some(download)
                    }
                    Ok(_) => {
                        let result = fs::remove_file(download);
                        match result {
                            Err(e) => {
                                warn!("{}: removing {}", e, download.to_str().unwrap());
                                Some(download)
                            }
                            Ok(_) => None,
                        }
                    }
                }
            })
            .cloned()
            .collect()
    }
}
