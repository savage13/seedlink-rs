
extern crate seedlink;
extern crate miniseed;
extern crate glob;

use std::io::Read;
use std::fs::File;

use miniseed::ms_record;

#[test]
fn file() {
    for entry in glob::glob("tests/ff*").unwrap() {
        if let Ok(f) = entry {

            let mut file = File::open(&f).unwrap();
            let mut buf = vec![];
            let _ = file.read_to_end(&mut buf).unwrap();

            let num = seedlink::parse_header(&buf).unwrap();
            buf.drain(..8);

            let msr = ms_record::parse(&mut buf);
            buf.drain(..8);
            println!("{}: {} [{}]", num, msr, buf.len());
        }
    }
}
