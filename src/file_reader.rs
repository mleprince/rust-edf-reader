//! Contains traits/implementations for reading a file in a sync or async manner.

use std::fs::File;
use futures::future::{err, ok};
use futures::Future;
use std::path::Path;
use positioned_io::ReadAt;


/**
 * An synchronous file reader
 */
pub trait SyncFileReader {
    fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, std::io::Error>;
}

/**
 * An asynchronous file reader (returns futures)
 */
pub trait AsyncFileReader {
    fn read_async(
        &self,
        offset: u64,
        length: u64,
    ) -> Box<Future<Item = Vec<u8>, Error = std::io::Error> + Send>;
}

/**
 * A FileReader for reading local files in blocking or non-blocking way
 */
pub struct LocalFileReader {
    random_access_file: File,
}

impl LocalFileReader {
    /**
     * Init the fileReader with the path of a local file
     */
    pub fn init<P: AsRef<Path>>(file_path: P) -> Result<LocalFileReader, std::io::Error> {
        let file = File::open(file_path)?;
        Ok(LocalFileReader {
            random_access_file: file,
        })
    }
}

impl SyncFileReader for LocalFileReader {
    fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, std::io::Error> {
        let mut data = vec![0; length as usize];

        self.random_access_file.read_at(offset, &mut data[..])?;

        Ok(data)
    }
}

impl AsyncFileReader for LocalFileReader {
    fn read_async(
        &self,
        offset: u64,
        length: u64,
    ) -> Box<Future<Item = Vec<u8>, Error = std::io::Error> + Send> {
        match self.read(offset, length) {
            Ok(data) => Box::new(ok::<Vec<u8>, std::io::Error>(data)),
            Err(e) => Box::new(err::<Vec<u8>, std::io::Error>(e)),
        }
    }
}
