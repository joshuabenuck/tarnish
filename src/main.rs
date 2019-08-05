extern crate hex;
extern crate log;
extern crate reqwest;
extern crate rustyline;
extern crate select;
extern crate serde;
extern crate serde_json;
extern crate simple_logger;

mod cache;
mod config;
mod library;
mod trove;
mod trove_feed;
mod util;

//use std::str;
//use std::fs::{self};//, DirEntry};
use config::Config;
use trove::Trove;
use std::process::Command;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::cache::{Cache};


/*
 * Sketch of API
 * use icli::{ICLI, Cmd};
 *
 * struct ICLI {
 *   history: Vec<String>,
 *   prompt: String,
 * }
 *
 * enum CResult {
 *   Subshell(icli),
 *   Ok(String),
 *   Err(error),
 * }
 *
 * trait Cmd {
 *  fn name() -> String;
 *  fn execute(Vec<&str> args) -> CResult;
 * }
 *
 * let cli = ICLI::new()
 *  .add(trove.cmds)
 *  .add(steam.cmds)
 *  .add(monthly.cmds)
 *  .add(downloader.cmds);
 * cli.run();
 */

fn main() {
    simple_logger::init_with_level(log::Level::Error).unwrap();
    let config = Config::new("./config.toml");
    let cache = Cache::new(&config.system.cache);
    let mut trove = match Trove::new(&config, &cache) {
        Ok(unwrapped) => unwrapped,
        Err(error) => panic!("Error constructing trove: {}", error),
    };
    let stray = trove.stray_downloads();
    println!("In downloads: {}", stray.len());
    trove.move_downloads();
    trove.update_download_status();
    let mut rl = rustyline::Editor::<()>::new();

    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history(".tarnish-history").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let mut words = line.split_ascii_whitespace();
                match words.next() {
                    Some("cache_all_metadata") => {
                        if let Err(err) = trove.cache_all_metadata(&cache) {
                            println!("cache_all_metadata: {}", err);
                        }
                    }
                    Some("cache_thumbnails") => {
                        trove.cache_thumbnails(&cache);
                    }
                    Some("cache_screenshots") => {
                        trove.cache_screenshots(&cache);
                    }
                    Some("update") => {
                        trove.update_download_status();
                    }
                    Some("download") => {
                        let number: usize = words.next().unwrap().parse::<usize>().unwrap();
                        let game = trove.not_downloaded()[number];
                        println!("Downloading: {}", trove.format(game));
                        let chrome = Command::new(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe")
                            .arg(trove.root.join(&game.downloads["windows"]))
                            .status()
                            .expect("Failed to launch Chrome.");
                    }
                    Some("downloaded") => {
                        trove
                            .downloaded()
                            .iter()
                            .zip(0..)
                            .for_each(|(p, i)| println!("{} {}", i, trove.format(p)));
                    }
                    Some("not_downloaded") => {
                        trove
                            .not_downloaded()
                            .iter()
                            .zip(0..)
                            .for_each(|(p, i)| println!("{} {}", i, trove.format(p)));
                    }
                    Some("exit") => break,
                    Some(_) => {}
                    None => break,
                }
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(".tarnish-history").unwrap();
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
