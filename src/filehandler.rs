use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::io;
use std::fs;
use std::io::{Result, Error};
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

pub struct FileHandler {
    pub filename: String,
    pub file: File,
    pub reader: BufReader<File>,
    pub inode: u64,
}

impl FileHandler {
    pub fn new(filename: String) -> Result<FileHandler> {
        let file = OpenOptions::new().read(true).open(&filename)?;
        let buf_cpy = file.try_clone()?;
        let metadata = file.metadata()?;
        let reader = BufReader::new(buf_cpy);
        let inode = metadata.ino();

        Ok(FileHandler { filename,
                         file,
                         reader,
                         inode })
    }

    pub fn read_file(&mut self, text: &mut String) -> Result<usize> {
        text.clear();
        let len = self.reader.read_to_string(text)?;
        //println!("read_to_string len {}", len);
        if len == 0 {
            // check if rolled
            //let maybe_rolled_file = OpenOptions::new().read(true).open(&self.filename)?;
            //let possibly_new_inode = maybe_rolled_file.metadata().unwrap().ino();
            let possibly_new_inode = fs::metadata(&self.filename).unwrap().ino();
            //println!("inode: {}, possibly_new_inode: {}", self.inode, possibly_new_inode);
            if possibly_new_inode != self.inode {
                // now we should try to read from the new file
                //self.file = maybe_rolled_file;
                self.file = OpenOptions::new().read(true).open(&self.filename)?;
                let buf_cpy = self.file.try_clone()?;
                self.reader = BufReader::new(buf_cpy);
                let len = self.reader.read_to_string(text)?;
                return Ok(len);
            }
        }
        return Ok(len);
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::fs;
    use filehandler::FileHandler;
    
    #[test]
    fn roll_log() {
        let text_block1 = "06/30 03:09:19.110 {I} : {\"requestHeaders\" : {\"Host\" : \"127.0.0.1:8265\", \"Accept\" : \"*/*\", \"Content-Length\" : \"502\", \"Content-Type\" : \"application/x-www-form-urlencoded\"}, \"queryParameters\" : {}, \"requestBody\" : {\"accountId\":\"AC26f91427cd2fadb33869b09f8b0428e67f82be6f\",\"requestId\":\"RQ8721e09386c95d59d0d57e8ba7b8d4875866bcf6\",\"vcsCallId\":\"CA26fc016661c6aee25d4ee2cc8b624662c3a4f18c\",\"vcsScriptId\":\"08d6c688a93d5c521704899d1947edfb86cdc84f\",\"interruptIndex\":0,\"event\":\"final\",\"vclResult\":{\"lastCommand\":0,\"commandName\":\"AddToConference\",\"lastNestedCommand\":null,\"nestedCommandName\":null,\"diagnostics\":\"successful completion\",\"returncode\":0,\"resultData\":{}},\"request_number\":\"907207\",\"location\":\"172.25.104.5:5237\"}}";

        // only different timestamp and requestId, change to length as well (location at the end...)
        let text_block2 = "06/30 03:09:19.111 {I} : {\"requestHeaders\" : {\"Host\" : \"127.0.0.1:8265\", \"Accept\" : \"*/*\", \"Content-Length\" : \"502\", \"Content-Type\" : \"application/x-www-form-urlencoded\"}, \"queryParameters\" : {}, \"requestBody\" : {\"accountId\":\"AC26f91427cd2fadb33869b09f8b0428e67f82be6f\",\"requestId\":\"RQ8721e09386c95d59d0d57e8ba7b8d4875866bcf7\",\"vcsCallId\":\"CA26fc016661c6aee25d4ee2cc8b624662c3a4f18c\",\"vcsScriptId\":\"08d6c688a93d5c521704899d1947edfb86cdc84f\",\"interruptIndex\":0,\"event\":\"final\",\"vclResult\":{\"lastCommand\":0,\"commandName\":\"AddToConference\",\"lastNestedCommand\":null,\"nestedCommandName\":null,\"diagnostics\":\"successful completion\",\"returncode\":0,\"resultData\":{}},\"request_number\":\"907207\",\"location\":\"172.25.104.5:5237abcd\"}}";

        let logfile_name = "logfile.log";
        let rotated_filename = "logfile1.log";

        // First create a file, have filehandler read it.
        // Then rename that file and create a log file, have filehandler read it.
        let mut logfile = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(logfile_name)
            .unwrap();

        let text1_len = text_block1.len();
        assert_eq!(text1_len, logfile.write(text_block1.as_bytes()).unwrap());

        let mut file_handle = FileHandler::new(logfile_name.clone().to_string()).unwrap();

        let mut some_text = String::new();

        assert_eq!(text1_len, file_handle.read_file(&mut some_text).unwrap());

        assert_eq!(some_text, text_block1);

        fs::rename(&logfile_name, &rotated_filename);

        let mut new_logfile = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(logfile_name)
            .unwrap();

        let text2_len = text_block2.len();
        assert_eq!(text2_len, new_logfile.write(text_block2.as_bytes()).unwrap());

        assert_eq!(text2_len, file_handle.read_file(&mut some_text).unwrap());

        assert_eq!(some_text, text_block2);

        assert_eq!((), fs::remove_file(logfile_name).unwrap());
        assert_eq!((), fs::remove_file(rotated_filename).unwrap());
    }
}
