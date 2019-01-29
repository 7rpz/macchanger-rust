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

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

use libc::sockaddr;

use std::io::Read;


#[derive(Debug, PartialEq, Clone)]
pub struct MAC {
    data: [u8; 6],
}


impl MAC {
    pub fn new() -> Self {
        Self { data: [0u8; 6] }
    }

    pub fn new_random(bia: bool) -> Self {
        let mut out = Self::new();

        // TODO: try different rngs here: urandom, hwrng, random, in this order
        let mut f = std::fs::File::open("/dev/urandom").unwrap();
        f.read_exact(&mut out.data).unwrap(); // then of course do not unwrap here...
        out.data[0] &= 0xfc; // make sure it's not multicast and not locally-administered

        if !bia {
            // set locally-administered bit
            out.data[0] |= 0x02;
        }

        out
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
