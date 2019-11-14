extern crate monit;

use std::env;
use std::process;
use std::thread;
use std::collections::BTreeMap;
use std::time::{Instant, Duration};
use monit::config::{Config, RegexBundle};
use monit::mongointerface::MongoInterface;


fn main() {

    let args : Vec<String> = env::args().collect();
    println!("{:?}", args);

    // cargo run -- 127.0.0.1 27017 test rustConfig
    
    let host = args[1].clone();
    let port: u16 = args[2].parse().expect("Cannot parse port number");

    let database = args[3].clone();
    let configCollection = args[4].clone();

    let t0 = Instant::now();
    // this is mutable because held db client object changes on get_config
    let mut mongodb = MongoInterface::new(host, port);

    let list_of_config_results = mongodb.get_config(database, configCollection);
    let mut configs_by_file: BTreeMap<String, RegexBundle> = BTreeMap::new();
    let mut list_of_rec_cfgs   = Vec::new();

    for config_result in list_of_config_results {
        match config_result {
            Ok((rec_cfg, parse_cfg)) => {
                
                let filename = parse_cfg.base_config.filename.clone();

                
                // borrow checker can't tell that these are two different func calls?
                // use entry api more simply, don't chain method call to it.  
                let regex_entry = configs_by_file.entry(filename.clone())
                    .or_insert(RegexBundle::new(filename).unwrap());
                
                regex_entry.add_parse_config(parse_cfg);
                
                list_of_rec_cfgs.push(rec_cfg);
            },
            Err(e) => {
                println!("Error retrieving and setting up configs! {}", e);
                process::exit(1);
            }
        }
    }

    // reorg the configs so that all parserconfigs are by filename.
    // maybe for now just iterate through a list of them and optimize later.
    let log_read_handler = thread::spawn(move || {
        loop {
            // iter through a map yields 
            for (filename, regex_bundle) in configs_by_file.iter_mut() {
                let read_res = monit::logreader::read_log(&mut regex_bundle.file_handler,
                                                          &mut regex_bundle.parser_configs);
                match read_res {
                    Ok(lineCt) => {
                        //println!("Parsed {} lines from log {}", lineCt, filename);
                        if lineCt == 0 {
                            thread::sleep(Duration::from_millis(10));
                        }
                    },
                    Err(e) => {
                        println!("File read error: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
    });

    let mongo_insert_handler = thread::spawn(move || {
        loop {
            for config in list_of_rec_cfgs.iter() {
                mongodb.insertRecords(&config);
                println!("Sleeping 100ms");
                thread::sleep(Duration::from_millis(100));
            }
        }
    });

    log_read_handler.join().unwrap();
    mongo_insert_handler.join().unwrap();
}

