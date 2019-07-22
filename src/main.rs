extern crate reqwest;
extern crate select;
extern crate sha2;
extern crate hex;
extern crate serde_json;
extern crate serde;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::str;
use std::fs::{self};//, DirEntry};
use std::path::{Path, PathBuf};
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate, Element};
use sha2::{Digest};
use std::fs::{File};
use serde_json::{Result, Value};
use serde::{Deserialize};
use serde_json::map::{Map};
use serde_json::Value::{Array, Object};

fn sha256(url: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.input(url.as_bytes());
    hex::encode(&hasher.result())
}

struct Cache {
    root: PathBuf,
}

impl Cache {
    fn new(root: &str) -> Cache {
        let cache = Cache { root: Path::new(root).to_path_buf() };
        if ! cache.root.exists() {
            println!("creating: {}", cache.root.display());
            if let Err(result) = fs::create_dir_all(&cache.root) {
                println!("error: {:?}", result);
            }
        }
        return cache;
    }

    fn retrieve(&self, url: &str) -> String {
        let hash = sha256(url);
        let cached = self.root.join(&hash);
        println!("{:?}", hash);
        if ! cached.exists() {
            // TODO: Add cache expiration
            println!("caching: {}", url);
            let mut resp = reqwest::get(url).unwrap();
            assert!(resp.status().is_success());
            let text = resp.text().unwrap();
            File::create(self.root.join(format!("{}.url", &hash))).unwrap().write(url.as_bytes()).unwrap();
            File::create(cached).unwrap().write(text.as_bytes()).unwrap();
            text
        }
        else {
            println!("using cached value: {}", &url);
            let mut contents = String::new();
            File::open(cached).unwrap().read_to_string(&mut contents).unwrap();
            contents
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TroveTimerOptions {
    current_time: String,
    next_addition_time: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
struct TroveUrl {
    web: String,
    bittorrent: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
struct TroveDownload {
    machine_name: String,
    name: String,
    url: TroveUrl,
    file_size: u64,
    //small: u8,
    md5: String,
    size: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
struct CarouselContent {
    youtube_link: Option<Vec<String>>,
    thumbnail: Vec<String>,
    screenshot: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
struct TrovePublisher {
    publisher_name: String,
    publisher_uri: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
struct Product {
    background_image: Option<String>, // can be null
    background_color: Option<String>, // can be null
    carousel_content: CarouselContent,
    date_added: u32,
    description_text: String,
    downloads: HashMap<String, TroveDownload>,
    #[serde(skip_deserializing)]
    downloaded: bool,
    human_name: String,
    humble_original: Option<bool>, // can be null
    image: String, // can be null
    logo: Option<String>,
    #[serde(rename = "machine_name")]
    machine_name: String,
    marketing_blurb: Value, //Map {text, style} or String,
    popularity: u16,
    publishers: Value, // can be null Vec<TrovePublisher>,
    trove_showcase_css: Option<String>, // can be null
    youtube_link: Option<String>, // can be null
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TroveData {
    all_access: Vec<String>,
    download_platform_order: Vec<String>,
    newly_added: Vec<Product>,
    display_item_data: Value,
    countdown_timer_options: TroveTimerOptions,
    standard_products: Vec<Product>,
}

//https://www.humblebundle.com/monthly/p/july_2019_monthly

struct Library {
    games: Vec<Game>
}

enum Launcher {
    Ubisoft,
    Twitch,
    Steam,
    Monthly,
    Trove,
}

struct Game {
    human_name: String,
    machine_name: String,
    installer: Option<String>,
    installed: bool,
    process: String,
    icon: String,
    screenshots: Option<Vec<String>>,
    trailer: Option<String>,
    launcer: Launcher,
}

fn get_trove_page(cache: &Cache, index: u8) -> Vec<Product> {
    let text = cache.retrieve(format!("https://www.humblebundle.com/api/v1/trove/chunk?index={}", index).as_str());
    let chunk: Vec<Product> = serde_json::from_str(text.as_str()).unwrap();
    chunk
}

fn get_trove_products(cache: &Cache) -> Vec<Product> {
    let mut products = Vec::new();
    for i in 0..5 {
        products.extend_from_slice(&get_trove_page(cache, i));
    }
    products
}

fn main() {
    println!("Hello, world!");
    let cache = Cache::new("./cache");
    let text = cache.retrieve("https://www.humblebundle.com/monthly/trove");
    let doc = Document::from(text.as_str());
    let data = doc.find(Attr("id", "webpack-monthly-trove-data")).next().unwrap().text();
    let mut data: TroveData = serde_json::from_str(data.as_str()).unwrap();
    data.standard_products = get_trove_products(&cache);
    data.standard_products.sort_by_key(|p| p.date_added);
    data.standard_products.reverse();

    println!("Newly added:");
    for product in &data.newly_added {
        println!("{}", product.human_name);
    }
    let local_root = Path::new("./trove");
    println!("All:");
    for product in &mut data.standard_products {
        let installer = &product.downloads["windows"].url.web;
        println!("{} {} {}", product.date_added, product.human_name, installer);
        product.downloaded = local_root.join(installer).exists();
    }
    println!("{}", &data.standard_products.len());
    //let data: Map<String, Value> = serde_json::from_str(data.as_str()).unwrap();
    //data.keys().for_each(|k| println!("{}", k));
    //println!("{}", data.has_key("displayItemData"));
    /*let data: &Map<String, Value> = match &data.get("standardProducts").unwrap()[0] {
        Object(o) => o,
        _ => panic!("Fail!")
    };*/
    //let data: Product = serde_json::from_value(data.get("standardProducts").unwrap()[0].clone()).unwrap();
    //data.keys().for_each(|k| println!("{}", k));
    //println!("{:?}", data);

    //println!("{}", data.is_array());
    //data.members().for_each(|k| println!("{}", k));
    // nodes have name, attrs, children
    // attrs are tuples
    /*for node in doc.find(Name("script")) {
        println!("{:?}", node.name());
        for attr in node.attrs() {
            println!("  {:?} {:?}", attr.0, attr.1);
        }
    }*/
    //doc.find(Element).for_each(|node| println!("{:?}", node.name()));
}
