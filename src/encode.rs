use clap::{App, SubCommand, ArgMatches, Arg};
use crate::util::{make_input_arg, read_raw_input, CmdError};
use crate::util;

pub fn make_encode_cmd<'a, 'b>() -> App<'a, 'b> {
    let encode_cmd = SubCommand::with_name("hex")
        .arg(make_input_arg("the input to encode. if - is provided, will read from stdin"))
        .arg(Arg::with_name("input-encoding")
            .short("-e")
            .help("the input's encoding")
            .possible_values(&["utf-8"])
            .default_value("utf-8"))
        .about("encodes the input as hex");

    SubCommand::with_name("encode")
        .subcommand(encode_cmd)
        .about("Convert data from one format to another.")
}

pub fn execute_encode_cmd(matches: &ArgMatches) -> util::Res<String> {
    match matches.subcommand() {
        ("hex", Some(sub)) => execute_encode_hex_cmd(sub.value_of("input").unwrap(), sub.value_of("input-encoding").unwrap()),
        (c, _) => Err(CmdError::UnknownSubcommand(String::from(c)).into())
    }
}

fn execute_encode_hex_cmd(input: &str, encoding: &str) -> util::Res<String> {
    let buf = read_raw_input(input)?;
    let res = match encoding {
        "utf-8" => {
            let encoded = hex::encode(buf);
            String::from(encoded)
        }
        _ => {
            panic!("invalid input encoding; should have been caught by CLI crate");
        }
    };

    Ok(format!("0x{}", res))
}