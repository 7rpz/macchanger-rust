// Copyright 2018 Urs Schulz
//
// This file is part of macchanger.
//
// macchanger is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// macchanger is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with macchanger.  If not, see <http://www.gnu.org/licenses/>.

use std::convert::TryInto;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::io::Read;
use std::num::ParseIntError;
use std::str::FromStr;

use libc::sockaddr;


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MAC {
    data: [u8; 6],
}


impl MAC {
    pub fn new() -> Self {
        Self { data: [0u8; 6] }
    }

    pub fn new_random(bia: bool) -> Self {
        use std::fs::File;

        let mut out = ["/dev/urandom", "/dev/hwrng", "/dev/random"]
            .iter()
            .filter_map(|rng| File::open(rng).ok())
            .filter_map(|mut f| {
                let mut out = Self::new();
                f.read_exact(&mut out.data).ok().map(|_| out)
            })
            .next()
            .expect("No working random number generator!");

        // make sure it's not multicast and not locally-administered
        out.data[0] &= 0xfc;
        if !bia {
            // set locally-administered bit
            out.data[0] |= 0x02;
        }

        out
    }

    pub fn get_ending(&self) -> &[u8; 3] {
        self.data[3..].try_into().unwrap()
    }

    pub fn set_ending(&mut self, ending: &[u8; 3]) {
        self.data[3..].copy_from_slice(ending);
    }
}


impl From<sockaddr> for MAC {
    fn from(addr: sockaddr) -> Self {
        let mut out = Self::new();

        for (n, x) in addr.sa_data[0..6].iter().enumerate() {
            out.data[n] = *x as u8;
        }

        out
    }
}


impl Into<[i8; 14]> for MAC {
    fn into(self) -> [i8; 14] {
        let mut out = [0i8; 14];

        for (n, b) in self.data.iter().enumerate() {
            out[n] = *b as i8;
        }

        out
    }
}


impl Display for MAC {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.data[0], self.data[1], self.data[2], self.data[3], self.data[4], self.data[5]
        )
    }
}


#[derive(Debug)]
pub enum ParseMACError {
    ParseIntError(ParseIntError),
    FormatError,
}


impl Display for ParseMACError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            ParseMACError::ParseIntError(e) => write!(f, "Failed to parse integer for MAC: {}", e),
            ParseMACError::FormatError => write!(f, "MAC has invalid format"),
        }
    }
}


impl FromStr for MAC {
    type Err = ParseMACError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = Self::new();

        for (n, p) in s.split(':').enumerate() {
            if n >= out.data.len() {
                return Err(ParseMACError::FormatError);
            }

            out.data[n] = u8::from_str_radix(p, 16).map_err(|e| ParseMACError::ParseIntError(e))?;
        }

        Ok(out)
    }
}
