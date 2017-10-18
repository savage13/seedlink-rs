
//! SeedLink Client
//!
//! More information and SeedLink Documentation at:
//!
//! https://www.seiscomp3.org/wiki/doc/applications/seedlink
//!
//! ```rust,ignore
//! extern crate seedlink;
//!
//! use seedlink::SeedLinkClient;
//!
//! fn read() {
//!
//!    let addr = "rtserve.iris.washington.edu";
//!    let port = 18000;
//!    let mut slc = SeedLinkClient::new(addr, port);
//!
//!    // Say Hello
//!    slc.hello().expect("bad hello");
//!
//!    // Read Response
//!    let mut data = vec![0u8;2048];
//!    let n = slc.read(&mut data).expect("bad read");
//!    let v = data[..n].to_vec();
//!    let s = String::from_utf8(v).expect("Found invalid UTF-8");
//!    println!("data: {:?}", s);
//!
//!    // Initiate Data Stream
//!    slc.start().expect("bad start");
//!
//!    let mut buf = vec![];
//!    // Read Response
//!    loop {
//!        println!("Waiting on read ...");
//!        let n = slc.read(&mut data).expect("bad read");
//!        buf.extend(data[..n].iter().cloned());
//!
//!        // 520 bytes = 8 for header + 512 for data
//!        if buf.len() >= 520 {
//!            // Parse data
//!            let (num, rec) = seedlink::parse(&mut buf).unwrap();
//!            println!("{}: {}", num, rec);
//!            break;
//!        }
//!    }
//!    // Say Good bye
//!    slc.bye().expect("bad bye");
//! }
//! ```


#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_xml_rs;

use serde_xml_rs::deserialize;

extern crate chrono;
extern crate miniseed;

use std::net::TcpStream;
use std::io::{Write,Read};
use std::io;

use std::time::Duration;
use chrono::DateTime;
use chrono::Utc;

use miniseed::ms_record;

/// SeedLink Client
#[derive(Debug)]
pub struct SeedLinkClient {
    /// Address:Port
    addr: String,
    /// Active tcp Stream
    stream: TcpStream,
    /// Verbosity
    verbose: bool
}

/// Stream Identifier representing the network, station, location and channel
#[derive(Debug,Clone)]
pub struct StreamID {
    /// Network, typically a two character string
    network: String,
    /// Station Identifier
    station: String,
    /// Location, e.g. 00, 10, 20, ...
    location: String,
    /// Channel, e.g. BHZ, HHE, LHN, ...
    channel: String,
}
impl StreamID {
    /// Generate a StreamID from a set of Strings
    pub fn new(net: &str, sta: &str, loc: &str, cha: &str) -> Self {
        StreamID {network:  net.to_owned(),
                  station:  sta.to_owned(),
                  location: loc.to_owned(),
                  channel:  cha.to_owned()
        }
    }
}

/// SeedLink Error
#[derive(Debug)]
pub enum SLError {
    String(String),
    Io(io::Error),
    Int(std::num::ParseIntError),
}

impl SeedLinkClient {
    /// Create a Client that connects to host:port.
    ///  Connection will attempt to be established.
    pub fn new(host: &str, port: i64) -> SeedLinkClient {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr)
            .expect("Cannot connect to server");
        SeedLinkClient{ stream: stream,
                        addr: addr,
                        verbose: false}

    }
    /// Set timeout in milliseconds during data reads from the server
    pub fn timeout(&mut self, millis: u64) -> Result<usize,SLError> {
        let s = &self.stream;
        let duration = Duration::from_millis( millis ); 
        match s.set_read_timeout(Some(duration)) {
            Ok(_) => {},
            Err(err) => {return Err(SLError::Io(err)); }
        }
        Ok(0)
    }
    /// Read data from the server.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize,SLError> {
        let mut s = &self.stream;
        s.read(buf).map_err(SLError::Io)
    }
    /// Write data to the server.  This is typically not used directly
    pub fn write<S: Into<String>>(&mut self, buf: S) -> Result<usize,SLError> {
        let mut s = &self.stream;
        s.write(&buf.into().as_bytes()).map_err(SLError::Io)
    }
    /// Send raw command to the server
    pub fn cmd(&mut self, cmd: &str) -> Result<usize,SLError> {
        if self.verbose {
            println!("SEND: {}", cmd);
        }
        self.write([cmd.to_owned() + "\r\n"].join(""))
    }
    /// Close the connection, say "BYE"
    pub fn bye(&mut self) -> Result<usize,SLError> {
        self.cmd("BYE")
    }
    /// Ask for list of stations, say "CAT"
    pub fn cat(&mut self) -> Result<usize,SLError> {
        self.cmd("CAT")
    }
    /// Initiate handshaking, say "HELLO"
    pub fn hello(&mut self) -> Result<usize,SLError> {
        self.cmd("HELLO")
    }
    /// Start data transfer (or end multi-station mode handshaking),
    /// say "END"
    pub fn end(&mut self) -> Result<usize,SLError> {
        self.cmd("END")
    }
    /// Start data transfer, see end(), say "END"
    pub fn start(&mut self) -> Result<usize,SLError> {
        self.end()
    }
    /// Start data transfer, see end(), say "END"
    pub fn data_please(&mut self) -> Result<usize,SLError> {
        self.end()
    }
    /// Request data from a timestamp until now, say "TIME when"
    pub fn backfill(&mut self, when: DateTime<Utc>) -> Result<usize,SLError> {
        let s = format!("TIME {}", when.format("%Y,%m,%d,%H,%M,%S"));
        self.cmd(s.as_str())
    }
    /// Select station to retrieve, calls station() and select()
    pub fn stream(&mut self, id: &StreamID) -> Result<usize, SLError> {
        self.station(id)?;
        self.select(id)
    }
    /// Select station to retrieve, say "STATION station network"
    pub fn station(&mut self, sid: &StreamID) -> Result<usize, SLError> {
        let s = format!("STATION {} {}", sid.station, sid.network);
        self.cmd(s.as_str())?;
        self.expect_ok()
    }
    /// Select Location and Channel, say "SELECT {location}{channel}"
    pub fn select(&mut self, sid: &StreamID) -> Result<usize, SLError> {
        let s = format!("SELECT {:2}{:3}", sid.location, sid.channel);
        try!(self.cmd(s.as_str()));
        self.expect_ok()
    }
    /// Read and expect an "OK" response from the server
    pub fn expect_ok(&mut self) -> Result<usize, SLError> {
        let mut rbuf = [0u8;2048];
        let n = try!(self.read(&mut rbuf));
        let s = String::from_utf8_lossy(&rbuf[..n]);
        if self.verbose {
            println!("===>: {:?}", s);
        }
        if s == "ERROR\r\n" {
            return Err(SLError::String(String::from("Seedlink returned an Error")));
        }
        if s != "OK\r\n" {
            return Err(SLError::String(format!("Seedlink returned an unexpected message: {}", s)));
        }
        return Ok(0);
    }
    /// Request data within a time range from server, say "TIME time1 time2"
    pub fn time_range(&mut self, t0: DateTime<Utc>, t1: DateTime<Utc>) -> Result<usize, SLError> {
        let st0 = t0.format("%Y,%m,%d,%H,%M,%S").to_string();
        let st1 = t1.format("%Y,%m,%d,%H,%M,%S").to_string();
        if st0 == st1 {
            return Err(SLError::String(String::from("Time Range has no Duration")));
        }
        let s = format!("TIME {} {}", st0, st1);
        self.cmd(s.as_str())?;
        self.expect_ok()
    }
    /// Handshaking, say "HELLO", read response, return number of bytes read
    pub fn connect(&mut self, verbose: bool) -> Result<usize, SLError> {
        try!(self.hello());
        self.verbose = verbose;
        // Read Response
        let mut data = vec![0u8;2048];
        let n = try!(self.read(&mut data));
        let v = data[..n].to_vec();
        let s = String::from_utf8(v).expect("Found invalid UTF-8");
        if self.verbose {
            println!("===>: {:?}", s);
        }

        Ok(n)
    }
    /// Determine available streams.  Be carefule, for public servers
    ///   this can take a bit of time to transfer the underlying data.
    /// Say "INFO STREAMS"
    pub fn available_streams(&mut self) -> Result<Seedlink, SLError> {
        let mut txt = String::with_capacity(1024);
        let mut rbuf = [0u8;4096];
        let mut buf = vec![];
        try!(self.cmd("INFO STREAMS"));
        loop {
            let mut id = 0;
            let n = try!(self.read(&mut rbuf));
            buf.extend(rbuf[..n].iter().cloned());
            while buf.len() >= 520 {
                id = parse_header(&buf)?;
                buf.drain(..8);
                let msr = ms_record::parse(&buf);
                buf.drain(..512);
                let chunk = msr.as_string().unwrap();
                txt += &chunk;
            }
            if id < 0 {
                break;
            }
        }
        match deserialize(txt.as_bytes()) {
            Ok(sl) => Ok(sl),
            Err(_) => Err(SLError::String(String::from("Error parsing xml"))),
        }
    }
}

/// Parse a SeedLink Packet
// https://stackoverflow.com/a/29682542
pub fn parse(mut buf: &mut Vec<u8>) -> Result<(i64,ms_record),SLError> {
    // SeedLink Header
    let num = parse_header(&buf)?;
    buf.drain(..8);

    // MiniSeed Record
    //println!("Parse Record: {:?}", buf[..512].to_vec());
    let msr = ms_record::parse(&mut buf);
    buf.drain(..512);

    Ok((num, msr))
}

// Convert a u8 array to a String
//pub fn u8_to_string(data: &[u8], n: usize) -> String {
//    String::from_utf8(data[..n].to_vec()).unwrap()
//}

/// Parse a SeedLink Packet Header
pub fn parse_header(buf: &[u8]) -> Result<i64,SLError> {
    if buf[0] as char != 'S' || buf[1] as char != 'L' {
        return Err(SLError::String(String::from("Not SeedLink Packet")));
    }
    let ss = String::from_utf8_lossy(&buf[2..8]);
    if ss == "INFO *" {
        return Ok(0);
    }
    if ss == "INFO  " {
        return Ok(-1);
    }
    let seqnum = i64::from_str_radix(&ss, 16).map_err(SLError::Int)?;

    Ok(seqnum)
}


/// SeedLink Metadata
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Seedlink {
    /// Software string
    software: String,
    /// Organization responsible for server
    organization: String,
    started: String,
    /// Station metadata
    station: Vec<Station>,
}
/// SeedLink Station Metadata
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Station {
    /// Station name
    name: String,
    /// Station Network
    network: String,
    /// Station Description
    description: String,
    /// Begin Sequence
    begin_seq: String,
    /// Ending Sequence
    end_seq: String,
    /// If checks are enabled on the stream?
    stream_check: String,
    /// Stream metadata
    stream: Vec<Stream>
}
/// SeedLink Stream Metadata
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Stream {
    /// Channel Name
    seedname: String,
    /// Location String
    location: String,
    /// Data Type
    #[serde(rename = "type")]
    stype: String,
    /// Data Begin Time
    begin_time: String,
    /// Data End Time
    end_time: String,
    begin_recno: Option<String>,
    end_recno: Option<String>,
    gap_check: Option<String>,
    gap_threshold: Option<String>,
}
use std::fmt;
impl fmt::Display for Stream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}", self.location.trim(), self.seedname.trim())
    }
}
impl fmt::Display for Station {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}", self.network.trim(), self.name.trim())
    }
}
impl Seedlink {
    /// Return available streams
    pub fn streams(&self) -> Vec<String> {
        let mut st = vec![];
        for sta in &self.station {
            for cha in &sta.stream {
                st.push( format!("{}_{}", sta,cha) );
            }
        }
        st.sort();
        st
    }
    #[doc(hidden)]
    pub fn read(file: &str) -> Seedlink {
        let mut fp = std::fs::File::open(file).expect("Error opening file");
        let mut buf = vec![];
        let _ = fp.read_to_end(&mut buf).unwrap();
        let txt = String::from_utf8(buf).unwrap();
        deserialize(txt.as_bytes()).unwrap()
    }
}



#[cfg(test)]
mod tests {

}
