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
mod ethtool;


use netdevice::{get_hardware, set_hardware};

use libc::c_int;

use std::io::{Error, Result};
use std::process::ExitCode;

use colored::Colorize;

use mac::MAC;
use ethtool::get_permanent_mac;


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

const MODES: [&str; 7] = [
    "show",
    "ending",
    "another",
    "any",
    "permanent",
    "random",
    "mac",
];

fn run() -> std::result::Result<(), String> {
    use clap::builder::{Arg, Command};

    let matches = Command::new("macchanger")
        .arg(
            Arg::new("show")
                .short('s')
                .long("show")
                .help("Print the MAC address and exit")
                .exclusive(true),
        )
        .arg(
            Arg::new("ending")
                .short('e')
                .long("ending")
                .help("Don't change the vendor bytes")
                .exclusive(true),
        )
        .arg(
            Arg::new("another")
                .short('a')
                .long("another")
                .help("Set random vendor MAC of the same kind")
                .exclusive(true),
        )
        .arg(
            Arg::new("any")
                .short('A')
                .long("any")
                .help("Set random vendor MAC of any kind")
                .exclusive(true),
        )
        .arg(
            Arg::new("permanent")
                .short('p')
                .long("permanent")
                .help("Reset to original, permanent hardware MAC")
                .exclusive(true),
        )
        .arg(
            Arg::new("random")
                .short('r')
                .long("random")
                .help("Set fully random MAC")
                .exclusive(true),
        )
        .arg(
            Arg::new("mac")
                .short('m')
                .long("mac")
                .value_name("XX:XX:XX:XX:XX:XX")
                .help("Set the MAC XX:XX:XX:XX:XX:XX")
                .exclusive(true),
        )
        .arg(
            Arg::new("bia")
                .short('b')
                .long("bia")
                .requires("random")
                .help("Pretend to be a burned-in-address"),
        )
        .arg(
            Arg::new("device")
                .required(true)
                .index(1),
        )
        .get_matches();

    let ifname = matches.get_one::<String>("device").expect("device is required");
    let bia = matches.get_one::<bool>("bia").is_some();

    let sock = get_socket().map_err(|e| format!("Failed to open socket: {}", e))?;
    let cur_addr =
        get_mac(sock, &ifname).map_err(|e| format!("Failed to get hardware address: {}", e))?;

    let prm_addr = get_permanent_mac(sock, &ifname).map_err(|e| format!("Failed to get permanent MAC: {}", e))?;

    println!("Current MAC:   {}", cur_addr);
    println!("Permanent MAC: {}", prm_addr);

    let new_addr = if matches.get_one::<bool>("show").is_some() {
        return Ok(());
    } else if matches.get_one::<bool>("ending").is_some() {
        let mut new = cur_addr.clone();
        new.set_ending(MAC::new_random(false).get_ending());
        new
    } else if matches.get_one::<bool>("another").is_some() {
        return Err("This option is currently not implemented.".to_string());
    } else if matches.get_one::<bool>("any").is_some() {
        return Err("This option is currently not implemented.".to_string());
    } else if matches.get_one::<bool>("permanent").is_some() {
        prm_addr
    } else if matches.get_one::<bool>("random").is_some() {
        MAC::new_random(bia)
    } else if let Some(mac) = matches.get_one::<String>("mac") {
            mac.parse().map_err(|e: mac::ParseMACError| e.to_string())?
    } else {
        return Err(
            "Exactly one of the following options is required: ".to_string() + &MODES.join(", "),
        );
    };

    println!("New MAC:       {}", new_addr);

    if new_addr == cur_addr {
        println!("{}", "It's the same MAC!!".yellow());
    }

    set_mac(sock, &ifname, &new_addr)
        .map_err(|e| format!("Failed to set new hardware address: {}", e))?;

    Ok(())
}


fn main() -> ExitCode {
    let res = run();
    match res {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            println!("[{}] {}", "ERROR".red(), e);
            ExitCode::FAILURE
        }
    }
}
