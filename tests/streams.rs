
extern crate seedlink;

use seedlink::SeedLinkClient;

#[test]
#[ignore]
fn streams() {
    let mut slc = SeedLinkClient::new("rtserve.iris.washington.edu", 18000);

    slc.connect(true).expect("bad hello");

    let info = slc.available_streams().expect("bad streams");
    let streams = info.streams();
    for s in streams {
        println!("{}", s);
    }
}
