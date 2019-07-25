use serde_json::{Result, Value};
use serde::{Deserialize};
use serde_json::map::{Map};
use crate::cache::{Cache};
use std::collections::HashMap;
//use serde_json::Value::{Array, Object};
use select::document::Document;
use select::predicate::{Attr};//, Class, Name, Predicate, Element};
use std::path::{Path};
use crate::config::{Config};

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
    pub publishers: Value, // can be null Vec<Publisher>,
    pub trove_showcase_css: Option<String>, // can be null
    pub youtube_link: Option<String>, // can be null
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

pub fn get_page(cache: &Cache, index: u8) -> Vec<Product> {
    let text = cache.retrieve(format!("https://www.humblebundle.com/api/v1/trove/chunk?index={}", index).as_str());
    let chunk: Vec<Product> = serde_json::from_str(text.as_str()).unwrap();
    chunk
}

pub fn get_products(cache: &Cache) -> Vec<Product> {
    let mut products = Vec::new();
    for i in 0..5 {
        products.extend_from_slice(&get_page(cache, i));
    }
    products
}

pub struct Trove {
}

impl Trove {
    pub fn new(config: &Config) {
        let cache = Cache::new(&config.system.cache);
        let text = cache.retrieve("https://www.humblebundle.com/monthly/trove");
        let doc = Document::from(text.as_str());
        let data = doc.find(Attr("id", "webpack-monthly-trove-data")).next().unwrap().text();
        let mut data: Data = serde_json::from_str(data.as_str()).unwrap();
        data.standard_products = get_products(&cache);
        data.standard_products.sort_by_key(|p| p.date_added);
        data.standard_products.reverse();

        println!("Newly added:");
        for product in &data.newly_added {
            println!("{}", product.human_name);
        }
        let local_root = &config.trove.root;
        assert!(local_root.exists());
        println!("All:");
        let mut count = 0;
        for product in &mut data.standard_products {
            let installer = Path::new(&product.downloads["windows"].url.web).file_name().unwrap().to_str().unwrap();
            product.downloaded = local_root.join(installer).exists();
            if product.downloaded { count += 1; }
            println!("{} {} {} {}", product.date_added, product.human_name, installer, product.downloaded);
            cache.retrieve(&product.image);
        }
        let downloads = Path::new(&config.system.downloads);
        assert!(downloads.exists());
        for product in &data.standard_products {
            let installer = Path::new(&product.downloads["windows"].url.web).file_name().unwrap().to_str().unwrap();
            if downloads.join(&installer).exists() {
                println!("Stray download {:?} {:?}", downloads, installer);
            }
        }
        println!("Downloaded: {}; Total: {}", &count, &data.standard_products.len());
    }
}
