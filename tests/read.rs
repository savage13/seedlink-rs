
extern crate seedlink;
extern crate miniseed;

use seedlink::SeedLinkClient;


#[test]
#[ignore]
fn read() {

    let mut slc = SeedLinkClient::new("rtserve.iris.washington.edu",18000);

    let mut data = vec![0u8;2048];

    // Say Hello
    slc.hello().expect("bad write");

    // Read Response
    let n = slc.read(&mut data).expect("bad read");
    let s = seedlink::u8_to_string(&data, n);
    println!("data: {:?}", s);

    // Initiate Data Stream
    slc.start().expect("bad write");

    // Read Response
    let n = slc.read(&mut data).expect("bad read");

    let mut buf = vec![];
    buf.extend(data[..n].iter().cloned());
    println!("{}", buf.len());


    // Parse data
    let (num, rec) = seedlink::parse(&mut buf).unwrap();

    //let msr = ms_record::parse(&mut buf[8..(8+512)]);
    println!("{}: {}", num, rec);

    // Say Good bye
    slc.bye().expect("bad bye");
}
