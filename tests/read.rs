
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
    let v = data[..n].to_vec();
    let s = String::from_utf8(v).expect("Found invalid UTF-8");
    println!("data: {:?}", s);

    // Initiate Data Stream
    slc.start().expect("bad write");

    let mut buf = vec![];
    // Read Response
    loop {
        println!("Waiting on read ...");
        let n = slc.read(&mut data).expect("bad read");
        buf.extend(data[..n].iter().cloned());

        if buf.len() >= 520 {
            // Parse data
            let (num, rec) = seedlink::parse(&mut buf).unwrap();
            println!("{}: {}", num, rec);
            break;
        }
    }
    // Say Good bye
    slc.bye().expect("bad bye");
}
