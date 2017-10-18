seedlink
========
SeedLink Library for rust

SeedLink is a protocol for retrieving seismic data in realtime and semi-realtime from available servers

For information about the data formats and protocols, see:

- SeedLink: https://www.seiscomp3.org/wiki/doc/applications/seedlink
- MiniSEED: http://ds.iris.edu/ds/nodes/dmc/data/formats/miniseed/
- SEED: http://ds.iris.edu/ds/nodes/dmc/data/formats/seed/


### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
seedlink = "1.0.0"
```

and this to your crate root:

```rust
extern crate seedlink;
```

You will probably need include the miniseed crate as well;

### Example
```rust
extern crate seedlink;
extern crate miniseed;

use seedlink::SeedLinkClient;

fn main() {

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
```

### Documentation

https://docs.rs/seedlink/

