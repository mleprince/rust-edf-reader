//! Metadata structures of an EDF file

use crate::parser::Parser;
use chrono::prelude::*;
use chrono::Utc;

pub const EDF_HEADER_BYTE_SIZE: usize = 256;

#[derive(Serialize, Deserialize, Debug,Clone,PartialEq)]
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
#[derive(Serialize, Deserialize, Debug,Clone,PartialEq)]
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

impl EDFHeader {
    pub fn build_general_header(data: Vec<u8>) -> EDFHeader {
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

    pub fn get_size_of_data_block(&self) -> u64 {
        self.channels
            .iter()
            .map(|channel| channel.number_of_samples_in_data_record * 2)
            .sum()
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
