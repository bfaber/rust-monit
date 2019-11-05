//#[macro_use(bson, doc)]

use regex::Regex;
use std::error::Error;

use mongodb::{Bson, bson, doc};
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use std::process;

use config::Config;
use config::RecordConfig;
use config::ParserConfig;

use recordprocessor::Record;

pub struct MongoInterface {
    host: String,
    port: u16,
    client: Client,
}

impl MongoInterface {

    pub fn new(host:String, port: u16) -> MongoInterface {

        let client = Client::connect(&host, port).expect("Problem connecting to Mongo");

        MongoInterface { host, port, client }
    }
    
    pub fn get_config(&mut self, database: String, configCollection: String) -> Vec<Result<(RecordConfig, ParserConfig), Box<dyn Error>>> {
        let mut configs = Vec::new();
        
        let coll = self.client.db(database.as_str()).collection(configCollection.as_str());
        
        // Find the document and receive a cursor
        let cursor = coll.find(Some(doc!{}), None)
            .ok().expect("Failed to execute find.");

        // cursor.next() returns an Option<Result<Document>>
        for result in cursor {
            let mut regStr         = String::new();
            let mut filename       = String::new();
            let mut key            = String::new();
            let mut collectionName = String::new();

            if let Ok(item) = result {
                if let Some(&Bson::String(ref filenm)) = item.get("filename") {
                    println!("filename: {}", filenm);
                    filename = filenm.clone();
                }
                if let Some(&Bson::String(ref regx)) = item.get("regex") {
                    println!("regex: {}", regx);
                    regStr = regx.clone();
                }            
                if let Some(&Bson::String(ref k)) = item.get("key") {
                    println!("key: {}", k);
                    key = k.clone();
                }            
                if let Some(&Bson::String(ref collNm)) = item.get("collectionName") {
                    println!("collectionName: {}", collNm);
                    collectionName = collNm.clone();
                }            
            }
            configs.push(Config::new(regStr, filename, key, collectionName));
        }

        configs

    }

    pub fn insertRecords(&mut self, config: &RecordConfig) {
        loop {
            let mut docs = Vec::new();
            
            let mut recv_iter = config.channel_rx.try_iter();
            
            while let Some(val) = recv_iter.next() {
                docs.push(doc!{config.base_config.key.clone(): val});
                if docs.len() == 100000 {
                    break;
                }            
            }
            
            let docs_len = docs.len();
            if docs_len == 0 {
                break;
            }
            
            println!("Pre-insert records count: {}", docs_len);

            // need to figure out how to use this db obj on self so that I can call
            // insertRecords in a loop
            // I think actually changing to &mut self in the sig fixes this
            let coll = self.client.db("test").collection(config.base_config.collectionName.as_str());

            let insert_result = coll.insert_many(docs, None);

            match insert_result {
                Ok(res) => {
                    if res.acknowledged {
                        let ack_len = res.inserted_ids.unwrap().len();
                        println!("Post insert ack: {}", ack_len);
                        if ack_len == docs_len {
                            // only loop if theres more docs in this iteration
                            break; 
                        }
                    } else {
                        println!("Unacknowledged insert!");
                    }
                },
                Err(e) => {
                    println!("Insert error: {}", e);
                    process::exit(1);
                },
            }
        }
    }
}


