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

                 
                /*
                // this works
                match configs_by_file.get_mut(&filename) {
                    Some(regex_bundle) => { regex_bundle.add_parse_config(parse_cfg); },
                    None => { configs_by_file.insert(filename.clone(),
                                                     RegexBundle::new(filename, parse_cfg).unwrap()); }
                }
                 */   

                /*           
                // this works -  does lookup twice tho
                if configs_by_file.contains_key(&filename) {
                    configs_by_file.get_mut(&filename)
                        .unwrap()
                        .add_parse_config(parse_cfg);
                } else {
                    configs_by_file.insert(filename.clone(),
                                           RegexBundle::new(filename, parse_cfg).unwrap());
                }
                */
                
                list_of_rec_cfgs.push(rec_cfg);
            },
            Err(e) => {
                println!("Error retrieving and setting up configs! {}", e);
                process::exit(1);
            }
        }
    }

//    let mut parse_cfg_map = BTreeMap::new();
    /*
    for cfg in list_of_parse_cfgs {
        parse_cfg_map.entry(cfg.base_config.filename)
            .or_insert(vec![cfg])
            .push(cfg);
    }
     
    list_of_parse_cfgs.iter()
        .for_each(
            move (cfg)
                parse_cfg_map.entry(cfg.base_config.filename)
                             .or_insert(vec![cfg])
                             .push(cfg));
                  
                                       
    

    let mut rec_cfg_map = BTreeMap::new();
    list_of_rec_cfgs.iter()
        .for_each(
            move (cfg)
                parse_cfg_map.entry(cfg.base_config.collectionName)
                             .or_insert(vec![cfg])
                             .push(cfg));
    
    */
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
                        println!("Parsed {} lines from log {}", lineCt, filename);
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
                let insertDt = Instant::now();
                mongodb.insertRecords(&config);
                thread::sleep(Duration::from_millis(100));
                println!("DT::MongoInsert::{} {:?}", config.base_config.filename, insertDt.elapsed());
            }
        }
    });

    log_read_handler.join().unwrap();
    mongo_insert_handler.join().unwrap();

    
/*    
    match config_result {
        Ok((rec_cfg, mut parse_cfg)) => {
            let log_read_handler = thread::spawn(move || {

                loop {
                    let read_res = monit::logreader::read_log(&mut parse_cfg);
                    match read_res {
                        Ok(lineCt) => {
                            println!("Parsed {} lines from log", lineCt);
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
            });

            let mongo_insert_handler = thread::spawn(move || {
                loop {
                    
                    let insertDt = Instant::now();
                    mongodb.insertRecords(&rec_cfg);
                    thread::sleep(Duration::from_millis(100));
                    println!("Insert dt: {:?}", insertDt.elapsed());
                }
            });

            log_read_handler.join().unwrap();
            mongo_insert_handler.join().unwrap();
        },
        Err(e) => {
            println!("Error retrieving config! {}", e);
        }
    }
    
    match config_result {
        Ok((mut record_config, mut parser_config)) => {

            loop {
                let read_result = monit::logreader::read_log(&mut parser_config);
                match read_result {
                    Ok(readLineCt) => {
                        println!("parsed {} lines from log", readLineCt);
                        if readLineCt == 0 {
                            break;
                        } else {
                                                        let insertDt = Instant::now();
                            mongodb.insertRecords(&mut record_config);
                            println!("Insert dt: {:?}", insertDt.elapsed());

                        }
                    },
                    Err(e) => {
                        println!("File read error: {}", e);
                        //process::exit(1);
                        break;
                    },                
                }        
            }
            
            println!("Total Duration: {:?}", t0.elapsed());
            
        },
        Err(e) => {
            println!("Could not get config! {}", e);
            process::exit(1);
        }
    }

    
    let config = Config::new(&args).unwrap_or_else( |err| {
        println!("Problem parsing args... {}", err);
        process::exit(1);
    });
     */

}

