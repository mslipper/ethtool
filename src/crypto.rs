use clap::{App, SubCommand, ArgMatches, Arg};
use crypto::sha3::Sha3;
use crypto::digest::Digest;
use crate::util::{make_input_arg, read_hex_input, decode_hex, encode_hex, CmdError};
use secp256k1::{Secp256k1, SecretKey, Message};
use std::io::Write;
use crypto::sha2::Sha256;
use std::{error, fmt};
use crate::util;
use crypto::ripemd160::Ripemd160;

#[derive(Debug)]
pub enum CryptoCmdError {
    InvalidSignatureLength,
    InvalidInputLength(usize, usize),
    InvalidPrivateKey,
}

impl fmt::Display for CryptoCmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            CryptoCmdError::InvalidSignatureLength => write!(f, "invalid signature length"),
            CryptoCmdError::InvalidPrivateKey => write!(f, "invalid private key"),
            CryptoCmdError::InvalidInputLength(exp_len, recv_len) => write!(f, "invalid input length, expected {} but got {}", exp_len, recv_len),
        }
    }
}

impl error::Error for CryptoCmdError {}

pub fn make_crypto_cmd<'a, 'b>() -> App<'a, 'b> {
    let keccak_256_cmd = SubCommand::with_name("keccak256")
        .arg(make_input_arg("The input to hash"))
        .about("Generates the keccak256 hash of the input");
    let sha2_256_cmd = SubCommand::with_name("sha2-256")
        .arg(make_input_arg("The input to hash"))
        .about("Generates the SHA2-256 hash of the input");
    let ripemd_160_cmd = SubCommand::with_name("ripemd-160")
        .arg(make_input_arg("The input to hash."))
        .about("Generates the RIPEMD-160 hash of the input");
    let eth_signed_msg_cmd = SubCommand::with_name("esmh")
        .arg(make_input_arg("The input to hash"))
        .about("Generates a message hash compatible with eth_sign.");
    let decompose_sig_cmd = SubCommand::with_name("decompose-sig")
        .arg(make_input_arg("The signature to decompose"))
        .about("Decomposes a signature into its V, R, and S components");
    let sign_cmd = SubCommand::with_name("ecdsa-sign")
        .arg(make_input_arg("The data to sign"))
        .arg(Arg::with_name("private-key")
            .short("-k")
            .required(true)
            .takes_value(true)
            .help("A hex-encoded private key to sign with."))
        .about("Signs the provided message");

    SubCommand::with_name("crypto")
        .subcommand(keccak_256_cmd)
        .subcommand(sha2_256_cmd)
        .subcommand(ripemd_160_cmd)
        .subcommand(eth_signed_msg_cmd)
        .subcommand(decompose_sig_cmd)
        .subcommand(sign_cmd)
        .about("Hash, sign, and verify data.")
}

pub fn execute_crypto_cmd(matches: &ArgMatches) -> util::Res<String> {
    match matches.subcommand() {
        ("keccak256", Some(sub)) => execute_keccak256(sub.value_of("input").unwrap()),
        ("sha2-256", Some(sub)) => execute_sha2_256(sub.value_of("input").unwrap()),
        ("ripemd-160", Some(sub)) => execute_ripemd_160(sub.value_of("input").unwrap()),
        ("esmh", Some(sub)) => execute_eth_signed_msg_cmd(sub.value_of("input").unwrap()),
        ("decompose-sig", Some(sub)) => execute_decompose_sig_cmd(sub.value_of("input").unwrap()),
        ("ecdsa-sign", Some(sub)) => execute_sign_cmd(sub.value_of("input").unwrap(), sub.value_of("private-key").unwrap()),
        (c, _) => Err(CmdError::UnknownSubcommand(String::from(c)).into())
    }
}

fn execute_keccak256(input: &str) -> util::Res<String> {
    let buf = read_hex_input(input)?;
    let mut hasher = Sha3::keccak256();
    hasher.input(buf.as_slice());
    Ok(format!("0x{}", hasher.result_str()))
}

fn execute_sha2_256(input: &str) -> util::Res<String> {
    let buf = read_hex_input(input)?;
    let mut hasher = Sha256::new();
    hasher.input(buf.as_slice());
    Ok(format!("0x{}", hasher.result_str()))
}

fn execute_ripemd_160(input: &str) -> util::Res<String> {
    let buf = read_hex_input(input)?;
    let mut hasher = Ripemd160::new();
    hasher.input(buf.as_slice());
    Ok(format!("0x{}", hasher.result_str()))
}

fn execute_eth_signed_msg_cmd(input: &str) -> util::Res<String> {
    let buf = read_hex_input(input)?;
    let mut hasher = Sha3::keccak256();
    hasher.input_str("\x19Ethereum Signed Message:\n");
    hasher.input_str(buf.len().to_string().as_str());
    hasher.input(buf.as_slice());
    Ok(format!("0x{}", hasher.result_str()))
}

fn execute_decompose_sig_cmd(input: &str) -> util::Res<String> {
    let buf = read_hex_input(input)?;

    if buf.len() != 65 {
        return Err(CryptoCmdError::InvalidSignatureLength.into());
    }

    Ok(format!("R: {}\nS:{}\nV: {}", hex::encode(&buf[0..32]), hex::encode(&buf[32..64]), buf[64]))
}

fn execute_sign_cmd(input: &str, pk_hex: &str) -> util::Res<String> {
    let input_buf = read_hex_input(input)?;
    let pk_buf = decode_hex(pk_hex).map_err(|_| {
        CryptoCmdError::InvalidPrivateKey
    })?;

    if input_buf.len() != 32 {
        return Err(CryptoCmdError::InvalidInputLength(32, input_buf.len()).into());
    }

    let secp = Secp256k1::new();
    let pk = SecretKey::from_slice(pk_buf.as_slice()).expect("32 bytes, within curve order");
    let msg = Message::from_slice(input_buf.as_slice()).expect("32 bytes");
    let sig = secp.sign_recoverable(&msg, &pk);
    let ser = sig.serialize_compact();
    let id = ser.0.to_i32() as u8;
    let mut out = ser.1.to_vec();
    out.write(&[id + 27])?;
    Ok(encode_hex(&out))
}