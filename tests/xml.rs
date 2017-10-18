
extern crate seedlink;

#[test]
fn xml() {
    let link = seedlink::Seedlink::read("tests/streams.xml");
    for s in link.streams() {
        println!("{}", s);
    }
}
