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

#![feature(termination_trait_lib, process_exitcode_placeholder, try_from)]

mod mac;


use netdevice::{get_hardware, set_hardware};

use libc::c_int;

use std::io::{Error, Result};
use std::process::ExitCode;

use colored::Colorize;

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


const MODES: [&str; 7] = [
    "show",
    "ending",
    "another",
    "any",
    "permanent",
    "random",
    "mac",
];

fn conflicts(mode: &str) -> Vec<&str> {
    let mut modes = MODES.to_vec();

    modes.retain(|x| *x != mode);
    modes
}


fn run() -> std::result::Result<(), String> {
    use clap::{App, Arg};

    let matches = App::new("macchanger")
        .version("0.1")
        .author("Urs Schulz <github@ursschulz.de>")
        .about("Program to change MAC addresses")
        .arg(
            Arg::with_name("show")
                .short("s")
                .long("show")
                .help("Print the MAC address and exit")
                .conflicts_with_all(&conflicts("show")),
        )
        .arg(
            Arg::with_name("ending")
                .short("e")
                .long("ending")
                .help("Don't change the vendor bytes")
                .conflicts_with_all(&conflicts("ending")),
        )
        .arg(
            Arg::with_name("another")
                .short("a")
                .long("another")
                .help("Set random vendor MAC of the same kind")
                .conflicts_with_all(&conflicts("another")),
        )
        .arg(
            Arg::with_name("any")
                .short("A")
                .long("any")
                .help("Set random vendor MAC of any kind")
                .conflicts_with_all(&conflicts("any")),
        )
        .arg(
            Arg::with_name("permanent")
                .short("p")
                .long("permanent")
                .help("Reset to original, permanent hardware MAC")
                .conflicts_with_all(&conflicts("permanent")),
        )
        .arg(
            Arg::with_name("random")
                .short("r")
                .long("random")
                .help("Set fully random MAC")
                .conflicts_with_all(&conflicts("random")),
        )
        .arg(
            Arg::with_name("mac")
                .short("m")
                .long("mac")
                .takes_value(true)
                .value_name("XX:XX:XX:XX:XX:XX")
                .help("Set the MAC XX:XX:XX:XX:XX:XX")
                .conflicts_with_all(&conflicts("mac")),
        )
        .arg(
            Arg::with_name("bia")
                .short("b")
                .long("bia")
                .requires("random")
                .help("Pretend to be a burned-in-address"),
        )
        .arg(
            Arg::with_name("device")
                .required(true)
                .index(1)
                .empty_values(false),
        )
        .get_matches();

    let ifname = matches.value_of("device").unwrap();
    let bia = matches.is_present("bia");

    let sock = get_socket().map_err(|e| format!("Failed to open socket: {}", e))?;
    let cur_addr =
        get_mac(sock, &ifname).map_err(|e| format!("Failed to get hardware address: {}", e))?;

    println!("Current MAC:   {}", cur_addr);
    println!("Permanent MAC: {}", "TODO");

    let new_addr = if matches.is_present("show") {
        return Ok(());
    } else if matches.is_present("ending") {
        let mut new = cur_addr.clone();
        new.set_ending(MAC::new_random(false).get_ending());
        new
    } else if matches.is_present("another") {
        return Err("This option is currently not implemented.".to_string());
    } else if matches.is_present("any") {
        return Err("This option is currently not implemented.".to_string());
    } else if matches.is_present("permanent") {
        return Err("This option is currently not implemented.".to_string());
    } else if matches.is_present("random") {
        MAC::new_random(bia)
    } else if matches.is_present("mac") {
        matches
            .value_of("mac")
            .unwrap()
            .parse()
            .map_err(|e: mac::ParseMACError| e.to_string())?
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
