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
    pub channel_tx: Sender<String>,
}

pub struct RegexBundle {
    pub parser_configs: Vec<ParserConfig>,
    pub file_handler: FileHandler,
}

impl Config {
    pub fn new(regStr: String,
               filename: String,
               key: String,
               collectionName: String) -> Result<(RecordConfig, ParserConfig), Box<dyn Error>>  {

        // todo: maybe pull out the regex instantiation to make the Config generation certain
        let regex = Regex::new(regStr.as_str())?;
        let (sender, receiver) = channel();
        let channel_tx = sender;
        let channel_rx = receiver;
        let base_config = Config { filename, key, collectionName };
        let config_copy = base_config.clone();
        let record_config = RecordConfig::new( base_config, channel_rx);

        let parse_config = ParserConfig::new( config_copy, regex, channel_tx );

        Ok((record_config, parse_config))
    }               
}

impl RecordConfig {
    pub fn new(base_config: Config, channel_rx: Receiver<String>) -> RecordConfig {
        RecordConfig{ base_config, channel_rx }
    }
}

impl ParserConfig {
    pub fn new(base_config: Config, regex: Regex, channel_tx: Sender<String>) -> ParserConfig {
        ParserConfig{ base_config, regex, channel_tx }
    }
}

impl RegexBundle {
    /*
    pub fn new(filename: String, config: ParserConfig) -> Result<RegexBundle, Box<dyn Error>> {
        let file_handler = FileHandler::new(filename)?;
        let mut parser_configs = Vec::new();
        parser_configs.push(config);
        Ok(RegexBundle{ parser_configs, file_handler })
    }
     */
    pub fn new(filename: String) -> Result<RegexBundle, Box<dyn Error>> {
        let file_handler = FileHandler::new(filename)?;
        let mut parser_configs = Vec::new();
        Ok(RegexBundle{ parser_configs, file_handler })
    }

    pub fn add_parse_config(&mut self, config: ParserConfig) {
        self.parser_configs.push(config);
    }
}
