extern crate edf_reader;
extern crate positioned_io_preview;


use positioned_io_preview::RandomAccessFile;

use edf_reader::EDFReader;

use std::time::SystemTime;

#[test]
fn parse_header() -> Result<(), std::io::Error> {
    let time = SystemTime::now();

    let edf_reader: EDFReader<RandomAccessFile> =
        EDFReader::init("tests/test_generator_2.edf")?;

    println!("{:?}", time.elapsed().unwrap());
    println!("{:#?}", edf_reader.edf_header);

    Ok(())
}