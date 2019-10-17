use clap::Arg;
use std::{io, error};
use std::io::{Read, Error, ErrorKind};
use hex::FromHexError;
use std::fmt;

pub type Res<T> = std::result::Result<T, Box<error::Error>>;

#[derive(Debug)]
pub enum CmdError {
    UnknownSubcommand(String)
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            CmdError::UnknownSubcommand(c) => write!(f, "unknown command {}", c)
        }
    }
}

impl error::Error for CmdError {}

pub fn make_input_arg(help: &str) -> Arg {
    Arg::with_name("input")
        .short("-i")
        .required(true)
        .takes_value(true)
        .help(help)
}

pub fn read_hex_input(input: &str) -> Result<Vec<u8>, Box<error::Error>> {
    if input == "-" {
        let mut vec = Vec::new();
        io::stdin().read_to_end(&mut vec)?;
        let s = String::from_utf8(vec).map_err(|e| {
            io::Error::new(ErrorKind::InvalidData, e)
        })?;
        read_hex_input(s.trim())
    } else if input.starts_with("0x") {
        decode_hex(input).map_err(|e| {
            io::Error::new(ErrorKind::InvalidData, e).into()
        })
    } else {
        Err(io::Error::new(ErrorKind::InvalidData, "data is not hex-encoded").into())
    }
}

pub fn read_raw_input(input: &str) -> Result<Vec<u8>, Error> {
    if input == "-" {
        let mut vec = Vec::new();
        io::stdin().read_to_end(&mut vec)?;
        Ok(vec)
    } else {
        let s = String::from(input);
        Ok(s.into_bytes())
    }
}

pub fn encode_hex<'a>(input: &Vec<u8>) -> String {
    let out = format!("0x{}", hex::encode(input));
    out.to_owned()
}

pub fn decode_hex(input: &str) -> Result<Vec<u8>, FromHexError> {
    let stripped = if input.starts_with("0x") {
        input.trim_start_matches("0x")
    } else {
        input
    };

    hex::decode(stripped)
}