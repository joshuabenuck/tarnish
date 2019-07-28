use crate::cache::Cache;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
//use serde_json::Value::{Array, Object};
use crate::config::Config;
use crate::library::{Game, Launcher};
use log::warn;
use select::document::Document;
use select::predicate::Attr; //, Class, Name, Predicate, Element};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimerOptions {
    pub current_time: String,
    pub next_addition_time: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Url {
    pub web: String,
    pub bittorrent: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Download {
    pub machine_name: String,
    pub name: String,
    pub url: Url,
    pub file_size: u64,
    //small: u8,
    pub md5: String,
    pub size: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CarouselContent {
    pub youtube_link: Option<Vec<String>>,
    pub thumbnail: Vec<String>,
    pub screenshot: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Publisher {
    pub publisher_name: String,
    pub publisher_uri: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Product {
    pub background_image: Option<String>, // can be null
    pub background_color: Option<String>, // can be null
    pub carousel_content: CarouselContent,
    pub date_added: u32,
    pub description_text: String,
    pub downloads: HashMap<String, Download>,
    #[serde(skip_deserializing)]
    pub downloaded: bool,
    pub human_name: String,
    pub humble_original: Option<bool>, // can be null
    pub image: String,
    pub logo: Option<String>,
    #[serde(rename = "machine_name")]
    pub machine_name: String,
    pub marketing_blurb: Value, //Map {text, style} or String,
    pub popularity: u16,
    pub publishers: Value,                  // can be null Vec<Publisher>,
    pub trove_showcase_css: Option<String>, // can be null
    pub youtube_link: Option<String>,       // can be null
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub all_access: Vec<String>,
    pub download_platform_order: Vec<String>,
    pub newly_added: Vec<Product>,
    pub display_item_data: Value,
    pub countdown_timer_options: TimerOptions,
    pub standard_products: Vec<Product>,
}

pub struct Trove {
    pub data: Data,
    pub cache: PathBuf,
    pub downloads: PathBuf,
    pub root: PathBuf,
    pub number_downloaded: u32,
    pub total: u32,
}

impl From<&Trove> for Vec<Game> {
    fn from(trove: &Trove) -> Vec<Game> {
        trove
            .data
            .standard_products
            .iter()
            .map(|g| g.into())
            .collect()
    }
}

impl From<&Product> for Game {
    fn from(p: &Product) -> Game {
        Game {
            human_name: p.human_name.clone(),
            icon: "".to_string(),
            installed: false,
            installer: None,
            launcher: Launcher::Trove,
            machine_name: p.machine_name.clone(),
            process: "".to_string(),
            screenshots: None,
            trailer: None,
        }
    }
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
        let mut data: Data = serde_json::from_str(data.as_str()).unwrap();
        data.standard_products = get_products(&cache);
        data.standard_products.sort_by_key(|p| p.date_added);
        data.standard_products.reverse();

        let mut trove = Trove {
            data,
            downloads: config.system.downloads.clone(),
            cache: config.system.cache.clone(),
            root: config.trove.root.clone(),
            number_downloaded: 0,
            total: 0,
        };
        assert!(trove.root.exists());
        trove.update_download_status();
        trove.data.standard_products.iter().for_each(|product| {
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
        for product in &mut self.data.standard_products {
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
        self.total = self.data.standard_products.len() as u32;
    }

    pub fn downloaded(&self) -> Vec<&Product> {
        (&self.data.standard_products)
            .iter()
            .filter(|p| p.downloaded)
            .collect()
    }

    pub fn not_downloaded(&self) -> Vec<&Product> {
        (&self.data.standard_products)
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
        (&self.data.standard_products)
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
