extern crate edf_reader;
extern crate positioned_io_preview;

use assert_approx_eq::assert_approx_eq;
use positioned_io_preview::RandomAccessFile;

use edf_reader::EDFReader;

use std::time::SystemTime;

/**
* For the tests, we use a test file.
* It come from : https://www.teuniz.net/edf_bdf_testfiles/
*
* Definition of the test file :
*  signal label/waveform  amplitude    f       sf
---------------------------------------------------
   1    squarewave        100 uV    0.1Hz   200 Hz
   2    ramp              100 uV    1 Hz    200 Hz
   3    pulse             100 uV    1 Hz    200 Hz
   4    ECG               100 uV    1 Hz    200 Hz
   5    noise             100 uV    - Hz    200 Hz
   6    sine 1 Hz         100 uV    1 Hz    200 Hz
   7    sine 8 Hz         100 uV    8 Hz    200 Hz
   8    sine 8.5 Hz       100 uV    8.5Hz   200 Hz
   9    sine 15 Hz        100 uV   15 Hz    200 Hz
  10    sine 17 Hz        100 uV   17 Hz    200 Hz
  11    sine 50 Hz        100 uV   50 Hz    200 Hz
*/

#[test]
fn parse_header() {
    let time = SystemTime::now();

    let edf_reader = get_reader();

    println!("{:?}", time.elapsed().unwrap());
    println!("{:#?}", edf_reader.edf_header);
}

fn get_reader() -> EDFReader<RandomAccessFile> {
    EDFReader::init("tests/test_generator_2.edf").unwrap()
}

/**
 * We will read the first 00s->10s , 10s->20s and 60s->70s
 * The first signal must contains a square ramp of 0.1Hz (10sec period) of amplitude 100uV and sampling rate 200Hz
 * We test :
 * - the length of the result data
 * - the values with an error margin = 0.1uV (the signal has a lsb of 0.03uV, we can find that with the phy max and digi max);
 */

#[test]
fn read_data() {
    read(0);
    read(10 * 1000);
    read(60 * 1000);
}

fn read(offset: u64) {
    let edf_reader = get_reader();

    // read data

    let sampling_rate: usize = 200;

    let data = &edf_reader.read_data_window(offset, 10 * 1000).unwrap();

    assert_eq!(edf_reader.edf_header.channels.len(),data.len());

    let data_ch0 = &data[0];

    assert_eq!(sampling_rate * 10, data_ch0.len());

    run_assert(data_ch0, sampling_rate);
}

fn run_assert(data: &Vec<f32>, sampling_rate: usize) {
    let delta_error = 0.1;

    for i in 0..sampling_rate * 5 as usize {
        assert_approx_eq!(data[i], 100 as f32, delta_error);
    }

    for i in 5 * sampling_rate..10 * sampling_rate as usize {
        assert_approx_eq!(data[i], -100 as f32, delta_error);
    }
}
