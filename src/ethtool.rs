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

use std::io::Result;

use libc::{c_int, c_ulong};

use crate::mac::MAC;


const IFNAMSIZ: usize = 16;
const MAX_ADDR_LEN: usize = 32;


const ETHTOOL_GPERMADDR: u32 = 0x00000020;
const SIOCETHTOOL: c_ulong = 0x8946;


#[repr(C)]
struct ifreq_data {
    name: [u8; IFNAMSIZ],
    data: *mut (),
}


#[repr(C)]
struct ethtool_perm_addr {
    cmd: u32,
    size: u32,
    data: [u8; MAX_ADDR_LEN],
}


pub fn get_permanent_mac(sock: c_int, ifname: &str) -> Result<MAC> {
    use libc::ioctl;
    use std::io::{Error, Write};

    let mut epa = ethtool_perm_addr {
        cmd: ETHTOOL_GPERMADDR,
        size: MAX_ADDR_LEN as u32,
        data: [0u8; MAX_ADDR_LEN],
    };

    let mut req = ifreq_data {
        name: [0; IFNAMSIZ],
        data: &mut epa as *mut _ as *mut (),
    };

    req.name.as_mut().write_all(ifname.as_bytes())?;

    if unsafe { ioctl(sock, SIOCETHTOOL, &mut req) } < 0 {
        return Err(Error::last_os_error());
    }

    Ok(MAC::from_slice(&epa.data[0..6]))
}
