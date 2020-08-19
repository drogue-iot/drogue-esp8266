use crate::parser;
use nom::IResult;
use crate::protocol::Response;
use nom::error::ErrorKind;
use moveslice::{Moveslice, Error};

pub(crate) struct Buffer {
    buffer: [u8; 4096],
    pos: usize,
    needs_parse: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            buffer: [0; 4096],
            pos: 0,
            needs_parse: false,
        }
    }

    pub fn write(&mut self, octet: u8) -> Result<(), u8> {
        //log::info!( "add [{}] {}", octet as char, octet);
        if self.pos >= self.buffer.len() {
            Err(octet)
        } else {
            self.buffer[self.pos] = octet;
            self.pos += 1;
            self.needs_parse = true;
            Ok(())
        }
    }

    pub fn parse(&mut self) -> Result<Response, ()> {
        if self.pos == 0 {
            return Ok(Response::None);
        }
        if ! self.needs_parse {
            return Ok(Response::None);
        }
        self.needs_parse = false;
        log::info!("parsing [{}]", core::str::from_utf8(&self.buffer[0..self.pos]).unwrap());
        let result = parser::parse(&self.buffer[0..self.pos]);
        //log::info!("parsing finished {:?}", result);
        match result {
            Ok((remainder, response)) => {
                let len = remainder.len();
                //log::info!("remaining: {}", len);
                if len > 0 {
                    //log::info!("remainder [{}]", core::str::from_utf8( remainder ).unwrap());
                    let start = self.pos - len;
                    //log::info!( "move {}..{} to 0", start, start+len);
                    self.buffer.moveslice(start..start+len, 0);
                    self.pos = len;
                    self.needs_parse = true;
                } else {
                    self.pos = 0;
                }
                //log::info!("here***");
                //
                // self.last_parse_pos = self.pos;
                //log::info!("leaving [{}]", core::str::from_utf8(&self.buffer[0..self.pos]).unwrap());
                //log::info!("ok {:?}", response);
                return Ok(response);
            }
            Err(nom::Err::Incomplete(_)) => {
                //info!( "incomplete");
                //Err(())
            }
            Err(nom::Err::Error(w)) => {
                //info!( "parse error {} {:?}", w.0.len(), w.1 );
                //Err(())
            }
            Err( nom::Err::Failure(_)) => {
                //info!( "parse failure");
                //Err(())
            }
        }

        Ok(Response::None)
    }
}
