//#[macro_use(bson, doc)]

use regex::Regex;
use std::error::Error;
use std::process;
use std::time::{Instant, Duration};

/*
use mongodb::{Bson, bson, doc};
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
 */

use config::Config;
use config::RecordConfig;
use config::ParserConfig;

use recordprocessor::Record;

use std::sync::Arc;
use mongo_driver::client::{Client,ClientPool,Uri};
use mongo_driver::cursor::Cursor;
use mongo_driver::collection::{BulkOperationOptions, BulkOperation};
use mongo_driver::write_concern::WriteConcern;
use bson::{bson,doc,Document};
use bson::ordered::OrderedDocument;

pub struct MongoInterface {
    host: String,
    port: u16,
    database: String,
    // maintain where to get config, can update later.
    // Only one config collection per monit process.
    config_coll: String, 
    //client: Client,
    client_pool: Arc<ClientPool>,
}

impl MongoInterface {

    pub fn new(host:String, port: u16, database: String, config_coll: String) -> MongoInterface {

        //let client = Client::connect(&host, port).expect("Problem connecting to Mongo");

        // just added this to see if new client will work
        let mut uriStr = String::from("mongodb://");
        uriStr.push_str(&host);
        uriStr.push(':');
        uriStr.push_str(&port.to_string());
        // "mongodb://localhost:27017/"
        let uri = Uri::new(uriStr).unwrap();
        let pool = Arc::new(ClientPool::new(uri.clone(), None));        

        MongoInterface { host, port, database, config_coll, client_pool: pool }
    }
    
    pub fn get_config(&mut self) -> Vec<Result<(RecordConfig, ParserConfig), Box<dyn Error>>> {
        let mut configs = Vec::new();
       
        //let coll = self.client.db(self.database.as_str()).collection(self.config_coll.as_str());
        // client_pool.pop() possibly blocks
        // client_pool.pop() -> Client
        // The client will be automatically added back to the pool when it goes out of scope.
        // pop() may block
        let client = self.client_pool.pop();

        if let Err(status_err) = client.get_server_status(None) {
            println!("Server returned error {:?}", status_err);
        }
        
        let coll = client.get_collection(self.database.as_str(), self.config_coll.as_str());
        
        println!("before find coll: {:?}", self.config_coll.as_str());
        println!("before find db: {:?}", self.database.as_str());
        let res_cursor = coll.find(&bson::ordered::OrderedDocument::new(), None);
        println!("after find");
        let res_cursor = match res_cursor {
            // cursor.next() returns an Option<Result<Document>>
            // find returns a Result<Cursor<Result<Document>>>

            Ok(cursor) => {
                for result in cursor {
                    println!("result in cursor {:?}", result);
                    let mut reg_str         = String::new();
                    let mut filename        = String::new();
                    let mut key             = String::new();
                    let mut collection_name = String::new();
                    
                    if let Ok(doc) = result {
                        if let Some(someBson) = doc.get("filename") {
                            println!("filename: {}", filename);
                            filename = someBson.as_str().unwrap().to_string();
                        }
                        if let Some(someBson) = doc.get("regex") {
                            reg_str = someBson.as_str().unwrap().to_string();
                        }
                        if let Some(someBson) = doc.get("key") {
                            key = someBson.as_str().unwrap().to_string();
                        }
                        if let Some(someBson) = doc.get("collectionName") {
                            collection_name = someBson.as_str().unwrap().to_string();
                        }

                    }
                    configs.push(Config::new(reg_str, filename, key, collection_name));
                }
            },
            
            Err(e) => {
                println!("Find config error: {:?}", e);
            },
        };
        println!("after match");
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

            let insertDt = Instant::now();
            println!("Pre-insert records count: {}", docs_len);

            // need to figure out how to use this db obj on self so that I can call
            // insertRecords in a loop
            // I think actually changing to &mut self in the sig fixes this
            //let coll = self.client.db("test").collection(config.base_config.collectionName.as_str());
            let client = self.client_pool.pop();
            let coll = client.get_collection(self.database.as_str(), config.base_config.collectionName.as_str());

            let bulk_opts = BulkOperationOptions{ ordered: false, write_concern: WriteConcern::default() };
            let bulk_op   = coll.create_bulk_operation(Some(bulk_opts).as_ref());
            for doc in docs {
                bulk_op.insert(&doc);
            }
            let bulk_op_result = bulk_op.execute();
            match bulk_op_result {
                Ok(res) => {
                    println!("Bulk insert successful. Inserted {} docs", docs_len);
                },
                Err(bulk_op_err) => {
                    println!("Bulk insert failed! {}", bulk_op_err);
                }                    
            }
            /*
            let insert_result = coll.insert_many(docs, None);

            match insert_result {
                Ok(res) => {
                    if res.acknowledged {
                        let ack_len = res.inserted_ids.unwrap().len();
                        println!("Post insert ack: {}", ack_len);
                        if ack_len == docs_len {
                            // only loop if theres more docs in this iteration
                            println!("DT::MongoInsert::{} {:?}",
                                     config.base_config.filename,
                                     insertDt.elapsed());
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
             */
        }
    }
}


