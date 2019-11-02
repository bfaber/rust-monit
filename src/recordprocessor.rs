use config::Config;

pub struct Record {
    pub collectionName: String,
    pub key: String,
    pub records: Vec<String>,
}

// validation of the config object should be happening in the Config::new
// validation of the mongo config record should be happening in mongointerface
// so there shouldnt be much to validate here, except that the results are not empty
// Its not an error to not have any results, but we don't want to consider this
// record in the mongointerface.
/*
impl Record {
    pub fn new(config: Config) -> Option<Record> {

        if config.results.len() == 0 {
            return None;
        }

        let key = config.key;
        let collectionName = config.collectionName;
        let mut records = Vec::new();
        
        for rec in config.results {
            records.push(rec);
        }
        
        Some(Record{ collectionName, key, records })
    }

}
*/


