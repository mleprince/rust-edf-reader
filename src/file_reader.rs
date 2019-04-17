//! Contains traits/implementations for reading a file in a sync or async manner.

use futures::Future;

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
