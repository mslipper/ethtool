use clap::{App, SubCommand, ArgMatches};
use crate::util;
use crate::util::{make_input_arg, decode_hex, encode_hex, CmdError};
use std::str::FromStr;
use std::error;
use std::fmt;
use std::io::Write;
use num_bigint::{BigUint, BigInt};

#[derive(Debug)]
pub enum ABIError {
    IOError(String),
    InvalidSize(u16),
    InvalidFieldType(String),
    InvalidFieldDefinition,
    InvalidValue(String),
    ByteSizeMismatch,
    Unimplemented,
}

impl fmt::Display for ABIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ABIError::IOError(s) => write!(f, "{}", s),
            ABIError::InvalidSize(s) => write!(f, "{} is an invalid size", s),
            ABIError::InvalidFieldType(t) => write!(f, "{} is an invalid type", t),
            ABIError::InvalidFieldDefinition => write!(f, "invalid field definition"),
            ABIError::InvalidValue(s) => write!(f, "{} is an invalid value", s),
            ABIError::ByteSizeMismatch => write!(f, "the size of the byte field specified does not match its actual size"),
            ABIError::Unimplemented => write!(f, "this feature is unimplemented")
        }
    }
}

impl error::Error for ABIError {}

impl From<std::io::Error> for ABIError {
    fn from(e: std::io::Error) -> Self {
        ABIError::IOError(String::from(e.to_string()))
    }
}

impl From<hex::FromHexError> for ABIError {
    fn from(e: hex::FromHexError) -> Self {
        ABIError::IOError(e.to_string())
    }
}

impl From<num_bigint::ParseBigIntError> for ABIError {
    fn from(e: num_bigint::ParseBigIntError) -> Self {
        ABIError::InvalidValue(e.to_string())
    }
}

impl From<std::num::ParseIntError> for ABIError {
    fn from(e: std::num::ParseIntError) -> Self {
        ABIError::InvalidValue(e.to_string())
    }
}

enum ABIField {
    Address,
    Boolean,
    String,
    Bytes,
    BytesN(u16),
    IntN(u16),
    UintN(u16),
    FixedN(u16, u16),
    UFixedN(u16, u16),
}

impl ABIField {
    fn parse_bytes(s: &str) -> Result<ABIField, ABIError> {
        field_size_from_name("bytes", s).and_then(|size| {
            if size == 0 {
                Ok(ABIField::Bytes)
            } else if size <= 32 {
                Ok(ABIField::BytesN(size))
            } else {
                Err(ABIError::InvalidSize(size))
            }
        })
    }

    fn parse_int(s: &str) -> Result<ABIField, ABIError> {
        field_size_from_name("int", s).and_then(|size| {
            if size == 0 {
                Ok(ABIField::IntN(256))
            } else if size <= 256 && size % 8 == 0 {
                Ok(ABIField::IntN(size))
            } else {
                Err(ABIError::InvalidSize(size))
            }
        })
    }

    fn parse_uint(s: &str) -> Result<ABIField, ABIError> {
        field_size_from_name("uint", s).and_then(|size| {
            if size == 0 {
                Ok(ABIField::UintN(256))
            } else if size <= 256 && size % 8 == 0 {
                Ok(ABIField::UintN(size))
            } else {
                Err(ABIError::InvalidSize(size))
            }
        })
    }

    fn parse_fixed(s: &str) -> Result<ABIField, ABIError> {
        fixed_field_size_from_name("fixed", s).and_then(|mn| {
            let (m, n) = mn;

            if m < 8 || m > 256 || m % 8 != 0 {
                return Err(ABIError::InvalidSize(m));
            }
            if n > 80 {
                return Err(ABIError::InvalidSize(n));
            }

            Ok(ABIField::FixedN(m, n))
        })
    }

    fn parse_ufixed(s: &str) -> Result<ABIField, ABIError> {
        fixed_field_size_from_name("fixed", s).and_then(|mn| {
            let (m, n) = mn;

            if m < 8 || m > 256 || m % 8 != 0 {
                return Err(ABIError::InvalidSize(mn.0));
            }
            if n > 80 {
                return Err(ABIError::InvalidSize(mn.1));
            }

            Ok(ABIField::UFixedN(m, n))
        })
    }

    fn encode_packed(&self, data: &str, buf: &mut Vec<u8>) -> Result<(), ABIError> {
        match self {
            ABIField::Address => encode_packed_address(data, buf),
            ABIField::String => encode_packed_string(data, buf),
            ABIField::UintN(size) => encode_packed_uintn(data, size, buf),
            ABIField::IntN(size) => encode_packed_intn(data, size, buf),
            ABIField::Bytes => encode_packed_bytes(data, buf),
            ABIField::BytesN(size) => encode_packed_bytesn(data, size, buf),
            ABIField::Boolean => encode_packed_bool(data, buf),
            _ => Err(ABIError::Unimplemented)
        }
    }
}

impl FromStr for ABIField {
    type Err = ABIError;

    fn from_str(s: &str) -> Result<ABIField, ABIError> {
        match s {
            "address" => Ok(ABIField::Address),
            "bool" => Ok(ABIField::Boolean),
            "string" => Ok(ABIField::String),
            name => {
                if s.starts_with("bytes") {
                    ABIField::parse_bytes(s)
                } else if s.starts_with("int") {
                    ABIField::parse_int(s)
                } else if s.starts_with("uint") {
                    ABIField::parse_uint(s)
                } else if s.starts_with("fixed") {
                    ABIField::parse_fixed(s)
                } else if s.starts_with("ufixed") {
                    ABIField::parse_ufixed(s)
                } else {
                    Err(ABIError::InvalidFieldType(String::from(name)))
                }
            }
        }
    }
}

fn field_size_from_name(prefix: &str, name: &str) -> Result<u16, ABIError> {
    let stripped = name.trim_start_matches(prefix);
    if stripped.len() == 0 {
        return Ok(0);
    }

    let parsed = stripped.parse()?;
    Ok(parsed)
}

fn fixed_field_size_from_name(prefix: &str, name: &str) -> Result<(u16, u16), ABIError> {
    let stripped = name.trim_start_matches(prefix);
    if stripped.len() == 0 {
        return Err(ABIError::InvalidFieldDefinition);
    }

    let sizes: Vec<&str> = stripped.split("x").collect();
    if sizes.len() != 2 {
        return Err(ABIError::InvalidFieldDefinition);
    }

    let m = sizes[0].parse()?;
    let n = sizes[1].parse()?;

    return Ok((m, n));
}

fn encode_packed_address(data: &str, buf: &mut Vec<u8>) -> Result<(), ABIError> {
    let dec = decode_hex(data)?;
    if dec.len() != 20 {
        return Err(ABIError::InvalidValue(String::from("invalid address")));
    }
    buf.write(dec.as_slice())?;
    Ok(())
}

fn encode_packed_bool(data: &str, buf: &mut Vec<u8>) -> Result<(), ABIError> {
    if data == "true" {
        let val: [u8; 1] = [0x01];
        buf.write(&val)?;
    } else if data == "false" {
        let val: [u8; 1] = [0x00];
        buf.write(&val)?;
    } else {
        return Err(ABIError::InvalidValue(String::from("invalid boolean value")));
    }

    Ok(())
}

fn encode_packed_string(data: &str, buf: &mut Vec<u8>) -> Result<(), ABIError> {
    buf.write(data.to_string().as_bytes())?;
    Ok(())
}

fn encode_packed_uintn(data: &str, size: &u16, buf: &mut Vec<u8>) -> Result<(), ABIError> {
    let num = BigUint::from_str(data)?;
    let mut b = num.to_bytes_be();
    let mut pad: Vec<u8> = vec![0; (size / 8) as usize - b.len()];
    pad.append(&mut b);
    buf.write(pad.as_slice())?;
    Ok(())
}

fn encode_packed_intn(data: &str, size: &u16, buf: &mut Vec<u8>) -> Result<(), ABIError> {
    let num = BigInt::from_str(data)?;
    let mut b = num.to_signed_bytes_be();
    let mut pad: Vec<u8> = vec![0; (size / 8) as usize - b.len()];
    pad.append(&mut b);
    buf.write(pad.as_slice())?;
    Ok(())
}

fn encode_packed_bytes(data: &str, buf: &mut Vec<u8>) -> Result<(), ABIError> {
    let mut data_buf = decode_hex(data)?;
    buf.append(&mut data_buf);
    Ok(())
}

fn encode_packed_bytesn(data: &str, size: &u16, buf: &mut Vec<u8>) -> Result<(), ABIError> {
    let mut data_buf = decode_hex(data)?;
    if data_buf.len() != *size as usize {
        return Err(ABIError::ByteSizeMismatch);
    }

    buf.append(&mut data_buf);
    Ok(())
}

pub fn make_abi_cmd<'a, 'b>() -> App<'a, 'b> {
    let encode_cmd = SubCommand::with_name("encode-packed")
        .arg(make_input_arg("the data to encode and its schema. If - is provided, will read from stdin"));

    SubCommand::with_name("abi")
        .subcommand(encode_cmd)
        .about("Encode and decode data using Ethereum's ABI.")
}

pub fn execute_abi_cmd(matches: &ArgMatches) -> util::Res<String> {
    match matches.subcommand() {
        ("encode-packed", Some(sub)) => execute_encode_packed_cmd(sub.value_of("input").unwrap()),
        (c, _) => Err(CmdError::UnknownSubcommand(String::from(c)).into()),
    }
}

fn execute_encode_packed_cmd(input: &str) -> util::Res<String> {
    let res = encode_abi_packed(input)?;
    Ok(encode_hex(&res))
}

pub fn encode_abi_packed(data: &str) -> Result<Vec<u8>, ABIError> {
    let fields = data.split(",");
    let mut buf: Vec<u8> = Vec::new();

    for field in fields {
        encode_field(field, &mut buf)?
    }

    Ok(buf)
}

pub fn encode_field(field: &str, buf: &mut Vec<u8>) -> Result<(), ABIError> {
    let values: Vec<&str> = field.split(":").collect();
    if values.len() != 2 {
        return Err(ABIError::InvalidFieldDefinition);
    }

    ABIField::from_str(values[0])?
        .encode_packed(values[1], buf)?;

    Ok(())
}