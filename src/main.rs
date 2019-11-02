extern crate monit;

use std::env;
use std::process;
use std::thread;
use std::time::{Instant, Duration};
use monit::config::Config;
use monit::mongointerface::MongoInterface;
use monit::recordprocessor::Record;


fn main() {

    let args : Vec<String> = env::args().collect();
    println!("{:?}", args);

    /*
    let regex    = Regex::new(args[1].as_str()).unwrap();
    let filename = args[2].clone();
     */
    
    let host = args[1].clone();
    let port: u16 = args[2].parse().expect("Cannot parse port number");

    let database = args[3].clone();
    let configCollection = args[4].clone();

    let t0 = Instant::now();
    // this is mutable because held db client object changes on get_config
    let mut mongodb = MongoInterface::new(host, port);

    let config_result = mongodb.get_config(database, configCollection);
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
/*    
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

