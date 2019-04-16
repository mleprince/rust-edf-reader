extern crate edf_reader;

use assert_approx_eq::assert_approx_eq;
use edf_reader::async_reader::AsyncEDFReader;
use edf_reader::file_reader::LocalFileReader;
use edf_reader::sync_reader::SyncEDFReader;
use futures::future::Future;
use edf_reader::model::*;

use std::sync::Arc;

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

const FILE_PATH: &str = "tests/test_generator_2.edf";
const SAMPLING_RATE: usize = 200;

#[test]
fn parse_header() {
    let async_edf_reader: AsyncEDFReader<LocalFileReader> = get_async_reader();
    let sync_edf_reader: SyncEDFReader<LocalFileReader> = get_sync_reader();

    assert_eq!(async_edf_reader.edf_header.channels.len(), 12);
    assert_eq!(sync_edf_reader.edf_header.channels.len(), 12);

    assert_eq!(
        sync_edf_reader.edf_header,
        Arc::try_unwrap(async_edf_reader.edf_header).unwrap()
    );

    println!("{:?}", sync_edf_reader.edf_header);
}

/**
 * We will read the first 00s->10s , 10s->20s and 60s->70s
 * The first signal must contains a square ramp of 0.1Hz (10sec period) of amplitude 100uV and sampling rate 200Hz
 * We test :
 * - the length of the result data
 * - the values with an error margin = 0.1uV (the signal has a lsb of 0.03uV, we can find that with the phy max and digi max);
 */

#[test]
fn read_sync_multiples_windows() {
    read_sync(0);
    read_sync(10 * 1000);
    read_sync(60 * 1000);
}

#[test]
fn read_async_multiples_windows() {
    read_async(0);
    read_async(10 * 1000);
    read_async(60 * 1000);
}

fn read_sync(offset: u64) {
    let edf_reader = get_sync_reader();

    let data = edf_reader.read_data_window(offset, 10 * 1000).unwrap();

    run_assert(&data,&edf_reader.edf_header);
}

fn read_async(offset: u64) {
    let edf_reader = get_async_reader();

    let data = edf_reader
        .read_data_window(offset, 10 * 1000)
        .wait()
        .unwrap();

    run_assert(&data,&edf_reader.edf_header);
}

fn run_assert(data: &Vec<Vec<f32>>, edf_header : &EDFHeader) {

    assert_eq!(edf_header.channels.len(), data.len());

    let data_ch0 = &data[0];

    assert_eq!(SAMPLING_RATE * 10, data_ch0.len());

    let delta_error = 0.1;

    for i in 0..SAMPLING_RATE * 5 as usize {
        assert_approx_eq!(data_ch0[i], 100 as f32, delta_error);
    }

    for i in 5 * SAMPLING_RATE..10 * SAMPLING_RATE as usize {
        assert_approx_eq!(data_ch0[i], -100 as f32, delta_error);
    }
}

fn get_async_reader() -> AsyncEDFReader<LocalFileReader> {
    AsyncEDFReader::init(FILE_PATH).wait().unwrap()
}

fn get_sync_reader() -> SyncEDFReader<LocalFileReader> {
    SyncEDFReader::init(FILE_PATH).unwrap()
}
