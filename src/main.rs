extern crate clap;

use clap::App;
use std::process;

pub mod abi;
pub mod crypto;
pub mod util;
pub mod encode;
pub mod units;
pub mod address;

fn main() {
    let matches = App::new("ethtool")
        .version("0.1.0")
        .author("Matthew Slipper <me@matthewslipper.com>")
        .about("A CLI multi-tool for Ethereum development.")
        .subcommand(crypto::make_crypto_cmd())
        .subcommand(abi::make_abi_cmd())
        .subcommand(encode::make_encode_cmd())
        .subcommand(units::make_units_cmd())
        .subcommand(address::make_address_cmd())
        .get_matches();

    let res = match matches.subcommand() {
        ("crypto", Some(sub)) => crypto::execute_crypto_cmd(sub),
        ("abi", Some(sub)) => abi::execute_abi_cmd(sub),
        ("encode", Some(sub)) => encode::execute_encode_cmd(sub),
        ("units", Some(sub)) => units::execute_units_cmd(sub),
        ("address", Some(sub)) => address::execute_address_cmd(sub),
        _ => {
            println!("invalid subcommand");
            process::exit(1)
        }
    };

    match res {
        Ok(out) => println!("{}", out),
        Err(e) => println!("error: {}", e)
    }
}
