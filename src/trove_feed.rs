/// This module handles the deserialization of the humble bundle monthly trove metadata feed.
/// It provides operations that deal with the contents of the feed itself.
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

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
pub struct Feed {
    pub all_access: Vec<String>,
    pub download_platform_order: Vec<String>,
    pub newly_added: Vec<Product>,
    pub display_item_data: Value,
    pub countdown_timer_options: TimerOptions,
    pub standard_products: Vec<Product>,
}


