// Documentation at:
//   https://www.seiscomp3.org/wiki/doc/applications/seedlink

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

extern crate glob;

#[derive(Debug)]
pub struct SeedLinkClient {
    addr: String,
    stream: TcpStream,
    verbose: bool
}

#[derive(Debug,Clone)]
pub struct StreamID {
    network: String,
    station: String,
    location: String,
    channel: String,
}
impl StreamID {
    pub fn new(net: &str, sta: &str, loc: &str, cha: &str) -> Self {
        StreamID {network:  net.to_owned(),
                  station:  sta.to_owned(),
                  location: loc.to_owned(),
                  channel:  cha.to_owned()
        }
    }
}

#[derive(Debug)]
pub enum SLError {
    String(String),
    Io(io::Error),
    Int(std::num::ParseIntError),
}

impl SeedLinkClient {
    pub fn new(host: &str, port: i64) -> SeedLinkClient {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr)
            .expect("Cannot connect to server");
        SeedLinkClient{ stream: stream,
                        addr: addr,
                        verbose: false}

    }
    pub fn timeout(&mut self, millis: u64) -> Result<usize,SLError> {
        let s = &self.stream;
        let duration = Duration::from_millis( millis ); 
        match s.set_read_timeout(Some(duration)) {
            Ok(_) => {},
            Err(err) => {return Err(SLError::Io(err)); }
        }
        Ok(0)
    }
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize,SLError> {
        let mut s = &self.stream;
        s.read(buf).map_err(SLError::Io)
    }
    pub fn write<S: Into<String>>(&mut self, buf: S) -> Result<usize,SLError> {
        let mut s = &self.stream;
        s.write(&buf.into().as_bytes()).map_err(SLError::Io)
    }
    pub fn cmd(&mut self, cmd: &str) -> Result<usize,SLError> {
        if self.verbose {
            println!("SEND: {}", cmd);
        }
        self.write([cmd.to_owned() + "\r\n"].join(""))
    }
    pub fn bye(&mut self) -> Result<usize,SLError> {
        self.cmd("BYE")
    }
    pub fn cat(&mut self) -> Result<usize,SLError> {
        self.cmd("CAT")
    }
    pub fn hello(&mut self) -> Result<usize,SLError> {
        self.cmd("HELLO")
    }
    pub fn end(&mut self) -> Result<usize,SLError> {
        self.cmd("END")
    }
    pub fn start(&mut self) -> Result<usize,SLError> {
        self.end()
    }
    pub fn data_please(&mut self) -> Result<usize,SLError> {
        self.end()
    }
    pub fn backfill(&mut self, when: DateTime<Utc>) -> Result<usize,SLError> {
        let s = format!("TIME {}", when.format("%Y,%m,%d,%H,%M,%S"));
        self.cmd(s.as_str())
    }
    pub fn stream(&mut self, id: &StreamID) -> Result<usize, SLError> {
        self.station(id)?;
        self.select(id)
    }
    pub fn station(&mut self, sid: &StreamID) -> Result<usize, SLError> {
        let s = format!("STATION {} {}", sid.station, sid.network);
        self.cmd(s.as_str())?;
        self.expect_ok()
    }
    pub fn select(&mut self, sid: &StreamID) -> Result<usize, SLError> {
        let s = format!("SELECT {:2}{:3}", sid.location, sid.channel);
        try!(self.cmd(s.as_str()));
        self.expect_ok()
    }
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
    /// Handshaking
    /// Say HELLO, Read Response
    /// Return number of bytes read
    pub fn connect(&mut self, verbose: bool) -> Result<usize, SLError> {
        try!(self.hello());
        self.verbose = verbose;
        // Read Response
        let mut data = vec![0u8;2048];
        let n = try!(self.read(&mut data));
        let s = u8_to_string(&data, n);
        if self.verbose {
            println!("===>: {:?}", s);
        }

        Ok(n)
    }
    pub fn available_streams(&mut self) -> Result<Seedlink, SLError> {
        let mut txt = String::with_capacity(1024);
        let mut rbuf = [0u8;4096];
        let mut buf = vec![];
        try!(self.cmd("INFO STREAMS"));
        loop {
            let mut id = 0;
            let n = try!(self.read(&mut rbuf));
            buf.extend(rbuf[..n].iter().cloned());
            while buf.len() > 8 {
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
        //println!("{}", txt);
        match deserialize(txt.as_bytes()) {
            Ok(sl) => Ok(sl),
            Err(_) => Err(SLError::String(String::from("Error parsing xml"))),
        }
    }
}

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

pub fn u8_to_string(data: &[u8], n: usize) -> String {
    String::from_utf8(data[..n].to_vec()).unwrap()
}

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



#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Seedlink {
    software: String,
    organization: String,
    started: String,
    station: Vec<Station>,
}
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Station {
    name: String,
    network: String,
    description: String,
    begin_seq: String,
    end_seq: String,
    stream_check: String,
    stream: Vec<Stream>
}
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Stream {
    seedname: String,
    location: String,
    #[serde(rename = "type")]
    stype: String,
    begin_time: String,
    end_time: String,
    begin_recno: String,
    end_recno: String,
    gap_check: String,
    gap_threshold: String,
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
}

#[cfg(test)]
mod tests {

}
