extern crate hex;
extern crate log;
extern crate reqwest;
extern crate select;
extern crate serde;
extern crate serde_json;
extern crate simple_logger;

mod cache;
mod config;
mod library;
mod trove;

//use std::str;
//use std::fs::{self};//, DirEntry};
use config::Config;
use library::{Game, Library};
use trove::Trove;

//https://www.humblebundle.com/monthly/p/july_2019_monthly

fn main() {
    simple_logger::init_with_level(log::Level::Error).unwrap();
    let config = Config::new("./config.toml");
    let mut trove = Trove::new(&config);
    let stray = trove.stray_downloads();
    println!("In downloads: {}", stray.len());
    trove.move_downloads();
    trove.update_download_status();
    let games: Vec<Game> = (&trove).into();
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
