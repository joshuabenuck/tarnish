extern crate reqwest;
extern crate select;
extern crate hex;
extern crate serde_json;
extern crate serde;

mod cache;
mod trove;
mod config;

//use std::str;
//use std::fs::{self};//, DirEntry};
use trove::{Trove};
use config::{Config};

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

fn main() {
    println!("Hello, world!");
    let config = Config::new("./config.toml");
    Trove::new(&config);
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
