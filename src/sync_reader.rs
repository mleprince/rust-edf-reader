//! Read an EDF file synchronously

use crate::file_reader::SyncFileReader;

use crate::model::*;

use std::io::Error;

pub struct SyncEDFReader<T: SyncFileReader> {
    pub edf_header: EDFHeader,
    file_reader: T,
}

impl<T: SyncFileReader> SyncEDFReader<T> {
    /**
    Init an EDFReader with a custom FileReader.
    It can be usefull if the EDF file is not located in the system file. (ie : we cannot use RandomAccessFile).
    An example of use : read the file with DOM FileAPI in Webassembly
    */
    pub fn init_with_file_reader(file_reader: T) -> Result<SyncEDFReader<T>, Error> {
        let general_header_raw = file_reader.read(0, 256)?;

        let mut edf_header = EDFHeader::build_general_header(general_header_raw);

        let channel_headers_raw = file_reader.read(
            256,
            edf_header.number_of_signals * EDF_HEADER_BYTE_SIZE as u64,
        )?;

        edf_header.build_channel_headers(channel_headers_raw);

        Ok(SyncEDFReader {
            edf_header,
            file_reader,
        })
    }

    pub fn read_data_window(
        &self,
        start_time_ms: u64, // in mS
        duration_ms: u64,   // in mS
    ) -> Result<Vec<Vec<f32>>, Error> {
        super::check_bounds(start_time_ms, duration_ms, &self.edf_header)?;

        // calculate the corresponding blocks to get

        let first_block_start_time = start_time_ms - start_time_ms % self.edf_header.block_duration;

        let first_block_index = first_block_start_time / self.edf_header.block_duration;

        let number_of_blocks_to_get =
            (duration_ms as f64 / self.edf_header.block_duration as f64).ceil() as u64;

        let offset = self.edf_header.byte_size_header
            + first_block_index * self.edf_header.get_size_of_data_block();

        let mut data;

        // TODO : better handle of errors

        match self.file_reader.read(
            offset,
            number_of_blocks_to_get * self.edf_header.get_size_of_data_block(),
        ) {
            Ok(d) => data = d,
            Err(e) => return Err(e),
        }

        let mut result: Vec<Vec<f32>> = Vec::new();

        for _ in 0..self.edf_header.number_of_signals {
            result.push(Vec::new());
        }

        let mut index = 0;

        for _ in 0..number_of_blocks_to_get {
            for (j, channel) in self.edf_header.channels.iter().enumerate() {
                for _ in 0..channel.number_of_samples_in_data_record {
                    let sample = super::get_sample(&data, index) as f32;
                    result[j].push(
                        (sample - channel.digital_minimum as f32) * channel.scale_factor
                            + channel.physical_minimum,
                    );
                    index += 1;
                }
            }
        }

        Ok(result)
    }
}
