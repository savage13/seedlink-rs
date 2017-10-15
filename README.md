seedlinl
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
```

### Documentation

https://docs.rs/seedlink/

