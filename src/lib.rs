/*!
 * edf-reader parse metadata of EDF file and can read block of data from this EDF file
 * spec of EDF format : https://www.edfplus.info/specs/edf.html
 *
 */

extern crate chrono;
extern crate positioned_io_preview;

mod parser;

#[macro_use]
extern crate serde_derive;

use chrono::prelude::*;
use chrono::Utc;
use parser::*;
use positioned_io_preview::RandomAccessFile;
use positioned_io_preview::ReadAt;
use std::mem::transmute;
use std::path::Path;

const EDF_HEADER_BYTE_SIZE: usize = 256;

// EDF

#[derive(Serialize, Deserialize, Debug)]
pub struct EDFChannel {
    pub label: String,                         // 16 ascii
    pub transducter_type: String,              // 80 ascii
    pub physical_dimension: String,            // 8 ascii
    pub physical_minimum: f32,                 // 8 ascii
    pub physical_maximum: f32,                 // 8 ascii
    pub digital_minimum: i64,                  // 8 ascii
    pub digital_maximum: i64,                  // 8 ascii
    pub prefiltering: String,                  // 80 ascii
    pub number_of_samples_in_data_record: u64, // 8 ascii
    pub scale_factor: f32,
}

/**
 * EDFHeader :
 *  - 256 bytes of common metadata
 *  - NumberOfChannels * channel metadata = N * 256 bytes
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct EDFHeader {
    pub file_version: String,
    pub local_patient_identification: String,
    pub local_recording_identification: String,
    pub start_date: String,
    pub start_time: String,
    pub record_start_time_in_ms: i64,
    pub byte_size_header: u64,
    pub number_of_blocks: u64,
    pub block_duration: u64,
    pub number_of_signals: u64,

    pub channels: Vec<EDFChannel>,
}

pub struct EDFReader<T: ReadAt> {
    pub edf_header: EDFHeader,
    file_reader: T,
}

impl EDFReader<RandomAccessFile> {
    /**
     Init an EDFReader with the path of a local file
     */
    pub fn init<P: AsRef<Path>>(
        file_path: P,
    ) -> Result<EDFReader<RandomAccessFile>, std::io::Error> {
        let file = RandomAccessFile::open(file_path)?;
        EDFReader::init_with_file_reader(file)
    }
}

impl<T: ReadAt> EDFReader<T> {

    /**
     Init an EDFReader with a custom FileReader.  
     It can be usefull if the EDF file is not located in the system file. (ie : we cannot use RandomAccessFile).
     An example of use : read the file with DOM FileAPI in Webassembly
     */
    pub fn init_with_file_reader(file_reader: T) -> Result<EDFReader<T>, std::io::Error> {
        let general_header_raw =
            EDFReader::read_bytes(&file_reader, 0, EDF_HEADER_BYTE_SIZE as u64)?;

        let mut edf_header = EDFHeader::build_general_header(general_header_raw);

        let channel_headers_raw = EDFReader::read_bytes(
            &file_reader,
            256,
            edf_header.number_of_signals * EDF_HEADER_BYTE_SIZE as u64,
        )?;

        edf_header.build_channel_headers(channel_headers_raw);

        Ok(EDFReader {
            edf_header,
            file_reader,
        })
    }

    fn read_bytes(file: &T, offset: u64, length: u64) -> Result<Vec<u8>, std::io::Error> {
        let mut data = vec![0; length as usize];

        file.read_at(offset, &mut data[..])?;

        Ok(data)
    }

    pub fn read_data_window(
        &self,
        start_time_ms: u64, // in mS
        duration_ms: u64,   // in mS
    ) -> Result<Vec<Vec<f32>>, String> {
        self.check_bounds(start_time_ms, duration_ms)?;

        // calculate the corresponding blocks to get

        let first_block_start_time = start_time_ms - start_time_ms % self.edf_header.block_duration;

        let first_block_index = first_block_start_time / self.edf_header.block_duration;

        let number_of_blocks_to_get =
            (duration_ms as f64 / self.edf_header.block_duration as f64).ceil() as u64;

        let offset =
            self.edf_header.byte_size_header + first_block_index * self.get_size_of_data_block();

        let mut data =
            vec![0u8; (number_of_blocks_to_get * self.get_size_of_data_block()) as usize];

        // TODO : better handle of errors
        if let Err(e) = self.file_reader.read_exact_at(offset, &mut data[..]) {
            return Err(String::from("failed to read data : ") + &e.to_string());
        }

        let mut result: Vec<Vec<f32>> = Vec::new();

        for _ in 0..self.edf_header.number_of_signals {
            result.push(Vec::new());
        }

        let mut index = 0;

        for _ in 0..number_of_blocks_to_get {
            for (j, channel) in self.edf_header.channels.iter().enumerate() {
                for _ in 0..channel.number_of_samples_in_data_record {
                    let sample = get_sample(&data, index) as f32;
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

    fn get_size_of_data_block(&self) -> u64 {
        self.edf_header
            .channels
            .iter()
            .map(|channel| channel.number_of_samples_in_data_record * 2)
            .sum()
    }

    fn check_bounds(&self, start_time: u64, duration: u64) -> Result<(), String> {
        if start_time + duration > self.edf_header.block_duration * self.edf_header.number_of_blocks
        {
            return Err(String::from("Window is out of bounds"));
        } else {
            Ok(())
        }
    }
}

fn get_sample(data: &Vec<u8>, index: usize) -> i16 {
    unsafe { transmute::<[u8; 2], i16>([data[2 * index].to_le(), data[2 * index + 1].to_le()]) }
}

impl EDFHeader {
    fn build_general_header(data: Vec<u8>) -> EDFHeader {
        let mut parser: Parser = Parser::new(data);

        let mut edf_header = EDFHeader {
            file_version: parser.parse_string(8),
            local_patient_identification: parser.parse_string(80),
            local_recording_identification: parser.parse_string(80),
            start_date: parser.parse_string(8),
            start_time: parser.parse_string(8),
            record_start_time_in_ms: 0,
            byte_size_header: parser.parse_number::<u64>(8),
            number_of_blocks: parser.move_offset(44).parse_number::<u64>(8),
            block_duration: parser.parse_number::<u64>(8) * 1000, // to get in mS
            number_of_signals: parser.parse_number::<u64>(4),
            channels: Vec::new(),
        };

        // TODO : create record_start_time
        edf_header.create_start_time();

        return edf_header;
    }

    pub fn build_channel_headers(&mut self, data: Vec<u8>) {
        // check if given data has the good size

        let mut parser = Parser::new(data);

        let label_list = parser.parse_string_list(self.number_of_signals, 16);
        let transductor_type_list = parser.parse_string_list(self.number_of_signals, 80);
        let physical_dimension_list = parser.parse_string_list(self.number_of_signals, 8);
        let physical_minimum_list = parser.parse_number_list::<f32>(self.number_of_signals, 8);
        let physical_maximum_list = parser.parse_number_list::<f32>(self.number_of_signals, 8);
        let digital_minimum_list = parser.parse_number_list::<isize>(self.number_of_signals, 8);
        let digital_maximum_list = parser.parse_number_list::<isize>(self.number_of_signals, 8);
        let prefiltering_list = parser.parse_string_list(self.number_of_signals, 80);
        let number_of_samples_in_data_record_list =
            parser.parse_number_list::<u64>(self.number_of_signals, 8);

        self.channels = (0..self.number_of_signals as usize)
            .map(|v| EDFChannel {
                label: label_list[v].clone(),
                transducter_type: transductor_type_list[v].clone(),
                physical_dimension: physical_dimension_list[v].clone(),
                physical_minimum: physical_minimum_list[v],
                physical_maximum: physical_maximum_list[v],
                digital_minimum: digital_minimum_list[v] as i64,
                digital_maximum: digital_maximum_list[v] as i64,
                prefiltering: prefiltering_list[v].clone(),
                number_of_samples_in_data_record: number_of_samples_in_data_record_list[v],
                scale_factor: (physical_maximum_list[v] - physical_minimum_list[v])
                    / (digital_maximum_list[v] - digital_minimum_list[v]) as f32,
            })
            .collect();
    }

    fn create_start_time(&mut self) {
        if self.start_date != "" && self.start_time != "" {
            let get_integers = |s: &String| -> Vec<u32> {
                s.split(".").map(|v| v.parse::<u32>().unwrap()).collect()
            };

            let splitted_date = get_integers(&self.start_date);
            let splitted_time = get_integers(&self.start_time);

            let real_year: i32 = 2000 + splitted_date[2] as i32;

            let date = Utc
                .ymd(real_year, splitted_date[1], splitted_date[0])
                .and_hms(splitted_time[0], splitted_time[1], splitted_time[2]);

            self.record_start_time_in_ms = date.timestamp_millis();
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn convert_byte_array_to_u16() {
        /**
        Javascript code to retreive the same results :

        const buffer = new ArrayBuffer(2*2);

        let view = new DataView(buffer);

        view.setInt16(0, 456,true);
        view.setInt16(2, -4564,true);

        console.log(new Uint8Array(buffer));  ==> Uint8Array [ 200, 1, 44, 238 ]
        */

        assert_eq!(456, get_sample(&vec![200, 1], 0));
        assert_eq!(-4564, get_sample(&vec![44, 238], 0));
    }

}
