pub struct Library {
    pub games: Vec<Game>,
}

pub enum Launcher {
    Ubisoft,
    Twitch,
    Steam,
    Monthly,
    Trove,
}

pub struct Game {
    pub human_name: String,
    pub machine_name: String,
    pub installer: Option<String>,
    pub installed: bool,
    pub process: String,
    pub icon: String,
    pub screenshots: Option<Vec<String>>,
    pub trailer: Option<String>,
    pub launcher: Launcher,
}
