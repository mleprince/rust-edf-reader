/*!
 * edf-reader parse metadata of EDF file and can read block of data from this EDF file
 * spec of EDF format : https://www.edfplus.info/specs/edf.html
 *
 */

extern crate chrono;
extern crate futures;
extern crate positioned_io;

#[macro_use]
extern crate serde_derive;

pub mod async_reader;
pub mod file_reader;
pub mod model;
mod parser;
pub mod sync_reader;

use std::mem::transmute;

use model::EDFHeader;

use std::io::{Error, ErrorKind};

fn get_sample(data: &Vec<u8>, index: usize) -> i16 {
    unsafe { transmute::<[u8; 2], i16>([data[2 * index].to_le(), data[2 * index + 1].to_le()]) }
}

fn check_bounds(start_time: u64, duration: u64, edf_header: &EDFHeader) -> Result<(), Error> {
    if start_time + duration > edf_header.block_duration * edf_header.number_of_blocks {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Window is out of bounds",
        ));
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::get_sample;

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
