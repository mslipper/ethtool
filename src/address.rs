use clap::{App, SubCommand, Arg, ArgMatches};
use crate::util;
use crate::util::{CmdError, encode_hex};
use secp256k1::Secp256k1;
use rand::OsRng;
use crypto::sha3::Sha3;
use crypto::digest::Digest;

pub fn make_address_cmd<'a, 'b>() -> App<'a, 'b> {
    let generate_cmd = SubCommand::with_name("generate")
        .arg(Arg::with_name("count")
            .short("-c")
            .help("Number of addresses to generate.")
            .required(true)
            .takes_value(true)
            .default_value("1"))
        .about("Generates a set of Ethereum addresses. Outputs both the address and its private key.");

    SubCommand::with_name("address")
        .subcommand(generate_cmd)
        .about("Generate, manipulate, and validate addresses.")
}

pub fn execute_address_cmd(matches: &ArgMatches) -> util::Res<String> {
    match matches.subcommand() {
        ("generate", Some(sub)) => execute_generate_cmd(sub.value_of("count").unwrap()),
        (c, _) => Err(CmdError::UnknownSubcommand(String::from(c)).into())
    }
}

fn execute_generate_cmd(input: &str) -> util::Res<String> {
    let count: u16 = input.parse()?;

    let secp = Secp256k1::new();
    let mut s = String::new();
    let mut h = Sha3::keccak256();
    let mut rng = OsRng::new().expect("OsRng");
    for i in 0..count {
        let (priv_k, pub_k) = secp.generate_keypair(&mut rng);
        let ser = pub_k.serialize_uncompressed();
        let deprefixed = &ser[1..];
        h.input(deprefixed);
        let mut out: [u8; 32] = [0; 32];
        h.result(&mut out);
        let addr = out[12..].to_vec();
        h.reset();

        s.push_str(format!("0x{} ", priv_k.to_string()).as_str());
        s.push_str(format!("{}", encode_hex(&addr)).as_str());
        if i != count - 1 {
            s.push_str("\n");
        }
    }

    Ok(s)
}