
use std::error::Error;
use std::io::BufReader;
use std::time::Instant;

use regex::Regex;

use config::ParserConfig;
use filehandler::FileHandler;

pub fn search<'a>(regex: &Regex, text_block: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();

    for line in text_block.lines() {
        let caps = regex.captures(line);
        if !caps.is_none() {
            results.push(caps.unwrap().get(1).unwrap().as_str());
        }
    }

    results
}

pub fn read_log(config: &mut ParserConfig) -> Result<u32, Box<dyn Error>> {
    let t0 = Instant::now();

//    let mut reader = BufReader::new(&config.file);
//    let mut file_handler = FileHandler::new(config.base_config.filename.clone());
    let mut text_block = String::new();

    let mut sendCt: u32 = 0;

    loop {
        text_block.clear();
        
        let len = config.file_handler.read_file(&mut text_block)?;
        if len == 0 {
            break;
        }
        println!("Read block {} bytes long", len);

        let results = search(&config.regex, &text_block);
        
        for res in results {
            config.channel_tx.send(String::from(res)).unwrap();
            sendCt += 1;
        }
    }

    println!("Parsed {} groups", sendCt);
    println!("Read/parse Duration: {:?}", t0.elapsed());
    Ok(sendCt)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn one_result() {
        let regex = "requestId\":\"(RQ[a-f0-9]+)";
        let re = Regex::new(regex).unwrap();
        let text_block = "06/30 03:09:19.110 {I} : {\"requestHeaders\" : {\"Host\" : \"127.0.0.1:8265\", \"Accept\" : \"*/*\", \"Content-Length\" : \"502\", \"Content-Type\" : \"application/x-www-form-urlencoded\"}, \"queryParameters\" : {}, \"requestBody\" : {\"accountId\":\"AC26f91427cd2fadb33869b09f8b0428e67f82be6f\",\"requestId\":\"RQ8721e09386c95d59d0d57e8ba7b8d4875866bcf6\",\"vcsCallId\":\"CA26fc016661c6aee25d4ee2cc8b624662c3a4f18c\",\"vcsScriptId\":\"08d6c688a93d5c521704899d1947edfb86cdc84f\",\"interruptIndex\":0,\"event\":\"final\",\"vclResult\":{\"lastCommand\":0,\"commandName\":\"AddToConference\",\"lastNestedCommand\":null,\"nestedCommandName\":null,\"diagnostics\":\"successful completion\",\"returncode\":0,\"resultData\":{}},\"request_number\":\"907207\",\"location\":\"172.25.104.5:5237\"}}";

        assert_eq!(vec!["RQ8721e09386c95d59d0d57e8ba7b8d4875866bcf6"], search(&re, &text_block));
    }

    #[test]
    fn two_results() {
        let regex = "requestId\":\"(RQ[a-f0-9]+)";
        let re = Regex::new(regex).unwrap();

        let text_block = "06/30 02:22:11.206 {I} : {\"requestHeaders\" : {\"Host\" : \"127.0.0.1:8265\", \"Accept\" : \"*/*\", \"Content-Length\" : \"502\", \"Content-Type\" : \"application/x-www-form-urlencoded\"}, \"queryParameters\" : {}, \"requestBody\" : {\"accountId\":\"AC26f91427cd2fadb33869b09f8b0428e67f82be6f\",\"requestId\":\"RQe2e19046d6ce51ee6a0e782978f68806b318328c\",\"vcsCallId\":\"CA04f4ec513e8c9a9957b61811897e2c5d9ad40302\",\"vcsScriptId\":\"1691e7c54f9397e7fcf7ffc5871cb273de7a045d\",\"interruptIndex\":0,\"event\":\"final\",\"vclResult\":{\"lastCommand\":0,\"commandName\":\"AddToConference\",\"lastNestedCommand\":null,\"nestedCommandName\":null,\"diagnostics\":\"successful completion\",\"returncode\":0,\"resultData\":{}},\"request_number\":\"389741\",\"location\":\"172.25.104.5:5237\"}}
06/30 02:22:14.266 {I} : {\"requestHeaders\" : {\"Host\" : \"127.0.0.1:8265\", \"Accept\" : \"*/*\", \"Content-Length\" : \"512\", \"Content-Type\" : \"application/x-www-form-urlencoded\"}, \"queryParameters\" : {}, \"requestBody\" : {\"accountId\":\"AC26f91427cd2fadb33869b09f8b0428e67f82be6f\",\"requestId\":\"RQb2dc824eb2ce8fcf51ed083483095d6222fe363d\",\"vcsCallId\":\"CAbfe4762269d967cdc573b9c47b986d6a3a50609e\",\"vcsScriptId\":\"c0374fd2dd4f1843b7bdb3f301d865a83a398334\",\"interruptIndex\":3,\"event\":\"addToConference\",\"vclResult\":{\"lastCommand\":0,\"commandName\":\"AddToConference\",\"lastNestedCommand\":null,\"nestedCommandName\":null,\"diagnostics\":\"successful completion\",\"returncode\":0,\"resultData\":\"\"},\"request_number\":\"402996\",\"location\":\"172.25.104.5:5237\"}}";

        assert_eq!(vec!["RQe2e19046d6ce51ee6a0e782978f68806b318328c", "RQb2dc824eb2ce8fcf51ed083483095d6222fe363d"], search(&re, &text_block));
    }

    #[test]
    fn diffUUID() {
        let regex = "requestId\": \"([-a-f0-9]{36})";
        let re = Regex::new(regex).unwrap();

        let text_block = "2019-10-23 15:57:48,899 DEBUG {\"machineId\": \"4azzz58ib\", \"userId\": \"88de5023-f5e8-11e9-ad8d-34363bd0b9a8\", \"requestId\": \"88de5023-f5e8-11e9-ad8d-34363bd0b9a8\"}";

        assert_eq!(vec!["88de5023-f5e8-11e9-ad8d-34363bd0b9a8"], search(&re, &text_block));
    }
}
