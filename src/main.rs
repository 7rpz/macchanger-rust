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

mod mac;


use netdevice::{get_hardware, set_hardware};

use libc::c_int;

use std::io::{Error, Result};

use mac::MAC;


/// Returns a new UDP socket
fn get_socket() -> Result<c_int> {
    use libc::{AF_INET, IPPROTO_UDP, SOCK_DGRAM};
    let res = unsafe { libc::socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP) };

    match res {
        -1 => Err(Error::last_os_error()),
        sock => Ok(sock),
    }
}


fn get_mac(sock: c_int, ifname: &str) -> Result<MAC> {
    let addr = get_hardware(sock, ifname)?;
    Ok(addr.into())
}


fn set_mac(sock: c_int, ifname: &str, addr: &MAC) -> Result<()> {
    let mut old_addr = get_hardware(sock, ifname)?;

    old_addr.sa_data = addr.clone().into();
    set_hardware(sock, ifname, old_addr)
}


fn run() -> std::result::Result<(), String> {
    let ifname = std::env::args()
        .nth(1)
        .ok_or("No interface name given".to_string())?;

    let sock = get_socket().map_err(|e| format!("Failed to open socket: {}", e))?;
    let cur_addr =
        get_mac(sock, &ifname).map_err(|e| format!("Failed to get hardware address: {}", e))?;
    let new_addr = MAC::new_random(true);

    println!("Old address: {}", cur_addr);
    println!("New address: {}", new_addr);

    set_mac(sock, &ifname, &new_addr)
        .map_err(|e| format!("Failed to set new hardware address: {}", e))?;

    Ok(())
}


fn main() {
    let res = run();
    match res {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    };
}
