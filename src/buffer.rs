use crate::parser;
use crate::protocol::Response;
use moveslice::Moveslice;

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
        if !self.needs_parse {
            return Ok(Response::None);
        }
        self.needs_parse = false;
        log::trace!(
            "parsing [{}]",
            core::str::from_utf8(&self.buffer[0..self.pos]).unwrap()
        );
        if let Ok((remainder, response)) = parser::parse(&self.buffer[0..self.pos]) {
            let len = remainder.len();
            if len > 0 {
                let start = self.pos - len;
                self.buffer.moveslice(start..start + len, 0);
                self.pos = len;
                self.needs_parse = true;
            } else {
                self.pos = 0;
            }
            return Ok(response);
        }

        Ok(Response::None)
    }
}
