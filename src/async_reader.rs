//! Read an EDF file asynhronously (with futures)

use std::sync::Arc;
use crate::file_reader::{AsyncFileReader, LocalFileReader};
use crate::model::{EDFHeader, EDF_HEADER_BYTE_SIZE};

use futures::future::{err};
use futures::Future;
use std::io::Error;
use std::path::Path;

pub struct AsyncEDFReader<T: AsyncFileReader> {
    pub edf_header: Arc<EDFHeader>,
    file_reader: T,
}

impl AsyncEDFReader<LocalFileReader> {
    /**
    Init an EDFReader with the path of a local file
    */
    pub fn init<P: AsRef<Path>>(file_path: P) -> Box<Future<Item = Self, Error = std::io::Error>> {
        match LocalFileReader::init(file_path) {
            Ok(file_reader) => Box::new(AsyncEDFReader::init_with_file_reader(file_reader)),
            Err(e) => Box::new(err::<Self, std::io::Error>(e)),
        }
    }
}

impl<T: 'static + AsyncFileReader> AsyncEDFReader<T> {
    /**
    Init an EDFReader with a custom FileReader.
    It can be usefull if the EDF file is not located in the system file. (ie : we cannot use RandomAccessFile).
    An example of use : read the file with DOM FileAPI in Webassembly
    */
    pub fn init_with_file_reader(
        file_reader: T,
    ) -> Box<Future<Item = AsyncEDFReader<T>, Error = std::io::Error>> {
        Box::new(
            file_reader
                .read_async(0, 256)
                .map(|general_header_raw: Vec<u8>| {
                    let mut edf_header = EDFHeader::build_general_header(general_header_raw);

                    file_reader
                        .read_async(
                            256,
                            edf_header.number_of_signals * EDF_HEADER_BYTE_SIZE as u64,
                        )
                        .map(|channel_headers_raw| {
                            edf_header.build_channel_headers(channel_headers_raw);

                            AsyncEDFReader {
                                edf_header : Arc::new(edf_header),
                                file_reader,
                            }
                        })
                })
                .flatten(),
        )
    }

    /// Says "Hello, [name]" to the `Person` it is called on.
    pub fn read_data_window(
        &self,
        start_time_ms: u64, // in mS
        duration_ms: u64,   // in mS
    ) -> Box<Future<Item = Vec<Vec<f32>>, Error = std::io::Error>> {
        // check boundaries
        if let Err(e) = super::check_bounds(start_time_ms, duration_ms, &self.edf_header) {
            return Box::new(err::<Vec<Vec<f32>>, Error>(e));
        }

        // calculate the corresponding blocks to get

        let first_block_start_time = start_time_ms - start_time_ms % self.edf_header.block_duration;

        let first_block_index = first_block_start_time / self.edf_header.block_duration;

        let number_of_blocks_to_get =
            (duration_ms as f64 / self.edf_header.block_duration as f64).ceil() as u64;

        let offset = self.edf_header.byte_size_header
            + first_block_index * self.edf_header.get_size_of_data_block();

        let header = self.edf_header.clone();

        Box::new(
            self.file_reader
                .read_async(
                    offset,
                    number_of_blocks_to_get * self.edf_header.get_size_of_data_block(),
                )
                .map(move |data: Vec<u8>| {
                    let mut result: Vec<Vec<f32>> = Vec::new();

                    for _ in 0..header.number_of_signals {
                        result.push(Vec::new());
                    }

                    let mut index = 0;

                    for _ in 0..number_of_blocks_to_get {
                        for (j, channel) in header.channels.iter().enumerate() {
                            for _ in 0..channel.number_of_samples_in_data_record {
                                let sample = super::get_sample(&data, index) as f32;
                                result[j].push(
                                    (sample - channel.digital_minimum as f32)
                                        * channel.scale_factor
                                        + channel.physical_minimum,
                                );
                                index += 1;
                            }
                        }
                    }

                    result
                })
        )
    }
}

// impl<T: AsyncFileReader> AsyncEDFReader<T> {

// }
