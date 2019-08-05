/// This module deals with the local install and cache status of files mentioned in the
/// feed. Its purpose is to identify those games that still need to be downloaded, categorized,
/// or that have been removed in recent copies of the feed.

/// The responsibilities of Trove end once the games have been installed.

use crate::cache::Cache;
use crate::config::Config;
use crate::trove_feed::{Feed, Product};
use crate::util::{extension, url_path_ext};
use std::collections::HashMap;
//use serde_json::Value::{Array, Object};
use log::warn;
use select::document::Document;
use select::predicate::Attr; //, Class, Name, Predicate, Element};
use std::fs;
use std::path::{Path, PathBuf};
use std::ops::{Deref, DerefMut};
use std::io::{Error, ErrorKind};
use std::str;

#[derive(Debug)]
pub struct Game {
    pub machine_name: String,
    pub human_name: String,
    pub description: String,
    pub date_added: u32,
    pub downloaded: bool, // eventually HashMap
    pub installed: bool,
    pub executable: PathBuf,
    pub download_urls: HashMap<String, String>,
    pub downloads: HashMap<String, PathBuf>,
    pub logo: Option<String>,
    pub image: String,
    pub screenshots: Vec<String>,
    pub thumbnails: Vec<String>,
    pub trailer: Option<String>,
    pub last_seen_on: String,
    pub removed_from_trove: bool,
}

pub struct Games(HashMap<String, Game>);

impl Deref for Games {
    type Target = HashMap<String, Game>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Games {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Trove {
    pub feed: Feed,
    pub cache: PathBuf,
    pub downloads: PathBuf,
    pub root: PathBuf,
    pub number_downloaded: u32,
    pub total: u32,
    pub games: Games,
    //pub installed_games: Vec<String>,
    //pub downloaded_games: Vec<String>,
    //pub not_downloaded_games: Vec<String>,
}

fn get_page(cache: &Cache, index: u8) -> Result<Vec<Product>, Error> {
    let bytes = cache.retrieve(
        format!(
            "https://www.humblebundle.com/api/v1/trove/chunk?index={}",
            index
        )
        .as_str(),
    )?;
    let chunk: Vec<Product> = serde_json::from_str(str::from_utf8(&bytes).unwrap())?;
    Ok(chunk)
}

fn get_products(cache: &Cache) -> Result<Vec<Product>, Error> {
    let mut products = Vec::new();
    for i in 0..5 {
        products.extend_from_slice(&get_page(cache, i)?);
    }
    Ok(products)
}

/*
 * trait Into<T>: Sized {fn into(self) -> T;}
 * trait From<T>: Sized {fn from(T) -> Self;}
 */
impl From<&Product> for Game {
    fn from(p: &Product) -> Game {
        let mut download_urls = HashMap::<String, String>::new();
        download_urls.insert("windows".to_string(), p.downloads["windows"].url.web.clone());
        Game {
            machine_name: p.machine_name.clone(),
            human_name: p.human_name.clone(),
            description: p.description_text.clone(),
            date_added: p.date_added,
            downloaded: false,
            installed: false,
            executable: "".to_string().into(),
            downloads: download_urls.iter().map(|(o, u)| (o.clone(), PathBuf::from(PathBuf::from(u).file_name().unwrap()).clone())).collect(),
            download_urls: download_urls,
            logo: p.logo.clone(),
            image: p.image.clone(),
            screenshots: p.carousel_content.screenshot.clone(),
            thumbnails: p.carousel_content.thumbnail.clone(),
            trailer: p.youtube_link.clone(),
            last_seen_on: "".to_string(),
            removed_from_trove: false,
        }
    }
}

impl From<Vec<Product>> for Games {
    fn from(products: Vec<Product>) -> Self {
        let result: HashMap<String, Game> = products.iter().map(|product| (product.machine_name.clone(), product.into())).collect();
        Self(result)
    }
}

impl Trove {
    pub fn new(config: &Config, cache: &Cache) -> Result<Trove, Error> {
        let text = cache.retrieve("https://www.humblebundle.com/monthly/trove")?;
        let doc = Document::from(str::from_utf8(&text).unwrap());
        let data = doc
            .find(Attr("id", "webpack-monthly-trove-data"))
            .next()
            .unwrap()
            .text();
        let mut data: Feed = serde_json::from_str(data.as_str()).unwrap();
        data.standard_products = get_products(&cache)?;
        data.standard_products.sort_by_key(|p| p.date_added);
        data.standard_products.reverse();

        let mut trove = Trove {
            feed: data.clone(),
            downloads: config.system.downloads.clone(),
            cache: config.system.cache.clone(),
            root: config.trove.root.clone(),
            number_downloaded: 0,
            total: 0,
            games: data.standard_products.into()
        };
        assert!(trove.root.exists());
        trove.update_download_status();
        trove.feed.standard_products.iter().for_each(|product| {
            if let Err(err) = cache.retrieve(&product.image) {
                println!("Warning: {}", err);
            }
        });
        println!(
            "Downloaded: {}; Total: {}",
            &trove.number_downloaded, &trove.total
        );
        Ok(trove)
    }

    pub fn update_download_status(&mut self) {
        let mut count = 0;
        for (_, game) in self.games.iter_mut() {
            let installer = game.downloads["windows"]
                .to_str()
                .unwrap();
            game.downloaded = self.root.join(installer).exists();
            if game.downloaded {
                count += 1;
            }
        }
        self.number_downloaded = count;
        self.total = self.games.len() as u32;
    }

    pub fn downloaded(&self) -> Vec<&Game> {
        (&self.games)
            .iter()
            .map(|(_, g)| g)
            .filter(|g| g.downloaded)
            .collect()
    }

    pub fn not_downloaded(&self) -> Vec<&Game> {
        (&self.games)
            .iter()
            .map(|(_, g)| g)
            .filter(|g| !g.downloaded)
            .collect()
    }

    pub fn cache_thumbnails(&self, cache: &Cache) {
        (&self.games)
            .iter()
            .map(|(_, g)| g)
            .flat_map(|p| &p.thumbnails)
            .for_each(|url| {
                println!("Caching {}", url.as_str());
                if let Err(err) = cache.retrieve(url.as_str()) {
                    println!("Warning: {}", err);
                }
            });
    }

    pub fn cache_screenshots(&self, cache: &Cache) {
        (&self.games)
            .iter()
            .map(|(_, g)| g)
            .flat_map(|p| &p.screenshots)
            .for_each(|url| {
                println!("Caching {}", url.as_str());
                if let Err(err) = cache.retrieve(url.as_str()) {
                    println!("Warning: {}", err);
                }
            });
    }

    /// Save current trove game metadata to disk
    /// Pull down copies of all game related images
    /// TODO: Throttle or rate limit this method
    pub fn cache_all_metadata(&self, cache: &Cache) -> Result<(), Error> {
        let metadata_root = self.root.join("metadata/");
        assert!(metadata_root.exists());
        for (name, game) in self.games.iter() {
            match url_path_ext(game.image.clone()) {
                None => println!("{} has no extension.", &game.image),
                Some(ext) => {
                    println!("{} is the ext for {}", ext, &game.image);
                    let image_filename = metadata_root.join(format!("{}.{}", name, ext));
                    fs::write(image_filename, cache.retrieve(&game.image)?)?;
                }
            }
            if let Some(logo) = &game.logo {
                match url_path_ext(logo.clone()) {
                    None => println!("{} has no extension.", &logo),
                    Some(ext) => {
                        println!("{} is the ext for {}", ext, &logo);
                        let image_filename = metadata_root.join(format!("{}_logo.{}", name, ext));
                        fs::write(image_filename, cache.retrieve(&logo)?)?;
                    }
                }
            }
            for (index, url) in game.thumbnails.iter().enumerate() {
                let img_format = match extension(url) {
                    Some(ext) => ext,
                    None => panic!("image doesn't have an extension"),
                };
                let target = format!("{}_t{}.{}", name, index, img_format);
                fs::write(metadata_root.join(target), cache.retrieve(url)?)?;
            }
            for (index, url) in game.screenshots.iter().enumerate() {
                let img_format = match extension(url) {
                    Some(ext) => ext,
                    None => panic!("image doesn't have an extension"),
                };
                let target = format!("{}_s{}.{}", name, index, img_format);
                fs::write(metadata_root.join(target), cache.retrieve(url)?)?;
            }
        }
        Ok(())
    }

    pub fn format(&self, g: &Game) -> String {
        format!("{} {} {}", g.date_added, g.human_name, g.downloaded)
    }

    pub fn stray_downloads(&self) -> Vec<PathBuf> {
        let downloads = Path::new(&self.downloads);
        assert!(downloads.exists());
        (&self.games)
            .iter()
            .map(|(_, g)| g)
            .filter_map(|game| {
                let installer = Path::new(&game.downloads["windows"])
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
