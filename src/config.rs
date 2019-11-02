use std::fs::File;
use std::error::Error;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use filehandler::FileHandler;

use regex::Regex;
// can't make fields mutable, so no mut here.

#[derive(Clone)]
pub struct Config {
    pub filename: String,
    pub key: String,
    pub collectionName: String,
}

pub struct RecordConfig {
    pub base_config: Config,
    pub channel_rx: Receiver<String>,
}

pub struct ParserConfig {
    pub base_config: Config,
    pub regex: Regex,
    pub file_handler: FileHandler,
    pub channel_tx: Sender<String>,
}

impl Config {
    pub fn new(regStr: String, filename: String, key: String, collectionName: String) -> Result<(RecordConfig, ParserConfig), Box<dyn Error>>  {

        let regex = Regex::new(regStr.as_str()).unwrap();
        let (sender, receiver) = channel();
//        let file = File::open(&filename)?;
        let fileHandle = FileHandler::new(filename.clone())?;
        let channel_tx = sender;
        let channel_rx = receiver;
        let base_config = Config { filename, key, collectionName };
        let config_copy = base_config.clone();
        let record_config = RecordConfig::new( base_config, channel_rx);

        let parse_config = ParserConfig::new( config_copy, regex, fileHandle, channel_tx );

        Ok((record_config, parse_config))
    }               
}

impl RecordConfig {
    pub fn new(base_config: Config, channel_rx: Receiver<String>) -> RecordConfig {
        RecordConfig{ base_config, channel_rx }
    }
}

impl ParserConfig {
    pub fn new(base_config: Config, regex: Regex, file_handler: FileHandler, channel_tx: Sender<String>) -> ParserConfig {
        ParserConfig{ base_config, regex, file_handler, channel_tx }
    }
}
