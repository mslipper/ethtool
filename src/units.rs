use clap::{App, SubCommand, Arg, ArgMatches};
use crate::util;
use crate::util::CmdError;
use std::error;
use std::fmt;
use std::str::FromStr;
use rust_decimal::Decimal;

#[derive(Debug)]
pub enum UnitError {
    InvalidUnit(String)
}

impl fmt::Display for UnitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            UnitError::InvalidUnit(u) => write!(f, "invalid unit: {}", u)
        }
    }
}

impl error::Error for UnitError {}

pub enum Unit {
    Wei,
    Kwei,
    Mwei,
    Gwei,
    Microether,
    Milliether,
    Ether,
}

impl Unit {
    fn convert_to_wei(&self, input: Decimal) -> Decimal {
        let res = match &self {
            Unit::Wei => Some(input),
            Unit::Kwei => input.checked_mul(Decimal::new(1e3 as i64, 0)),
            Unit::Mwei => input.checked_mul(Decimal::new(1e6 as i64, 0)),
            Unit::Gwei => input.checked_mul(Decimal::new(1e9 as i64, 0)),
            Unit::Microether => input.checked_mul(Decimal::new(1e12 as i64, 0)),
            Unit::Milliether => input.checked_mul(Decimal::new(1e15 as i64, 0)),
            Unit::Ether => input.checked_mul(Decimal::new(1e18 as i64, 0)),
        };
        
        res.unwrap()
    }

    fn convert_from_wei(&self, input: Decimal) -> Decimal {
        let res = match &self {
            Unit::Wei => Some(input),
            Unit::Kwei => input.checked_div(Decimal::new(1e3 as i64, 18)),
            Unit::Mwei => input.checked_div(Decimal::new(1e6 as i64, 18)),
            Unit::Gwei => input.checked_div(Decimal::new(1e9 as i64, 18)),
            Unit::Microether => input.checked_div(Decimal::new(1e12 as i64, 18)),
            Unit::Milliether => input.checked_div(Decimal::new(1e15 as i64, 18)),
            Unit::Ether => input.checked_div(Decimal::new(1e18 as i64, 18)),
        };

        res.unwrap()
    }

    fn possible_values<'a>() -> &'a [&'a str] {
        &["wei", "kwei", "babbage", "mwei", "lovelace", "gwei", "shannon",
            "microether", "szabo", "milliether", "finney", "ether", "eth"]
    }

    fn from_str(input: &str) -> Result<Unit, UnitError> {
        match input {
            "wei" => Ok(Unit::Wei),
            "kwei" | "babbage" => Ok(Unit::Kwei),
            "mwei" | "lovelace" => Ok(Unit::Mwei),
            "gwei" | "shannon" => Ok(Unit::Gwei),
            "microether" | "szabo" => Ok(Unit::Microether),
            "milliether" | "finney" => Ok(Unit::Milliether),
            "ether" | "eth" => Ok(Unit::Ether),
            u => Err(UnitError::InvalidUnit(String::from(u)))
        }
    }
}

pub fn make_units_cmd<'a, 'b>() -> App<'a, 'b> {
    let from_wei_cmd = SubCommand::with_name("from-wei")
        .arg(Arg::with_name("amount")
            .help("the amount to convert from Wei")
            .index(1)
            .required(true))
        .arg(Arg::with_name("unit")
            .help("the input unit")
            .index(2)
            .required(true)
            .default_value("ether"))
        .about("converts an amount into Ether");
    let to_wei_command = SubCommand::with_name("to-wei")
        .arg(Arg::with_name("amount")
            .help("the amount to convert to Wei")
            .index(1))
        .arg(Arg::with_name("unit")
            .help("the input unit")
            .index(2)
            .required(true)
            .possible_values(Unit::possible_values())
            .default_value("ether"))
        .about("converts an amount into Wei");

    SubCommand::with_name("units")
        .subcommand(from_wei_cmd)
        .subcommand(to_wei_command)
        .about("Convert between Ethereum's various monetary units.")
}

pub fn execute_units_cmd(matches: &ArgMatches) -> util::Res<String> {
    match matches.subcommand() {
        ("from-wei", Some(sub)) => execute_from_wei_cmd(
            sub.value_of("amount").unwrap(),
            sub.value_of("unit").unwrap(),
        ),
        ("to-wei", Some(sub)) => execute_to_wei_cmd(
            sub.value_of("amount").unwrap(),
            sub.value_of("unit").unwrap(),
        ),
        (c, _) => Err(CmdError::UnknownSubcommand(String::from(c)).into())
    }
}

fn execute_from_wei_cmd(amount: &str, unit_str: &str) -> util::Res<String> {
    let mut amount = Decimal::from_str(amount)?;
    amount.set_scale(18).expect("Scale exceeds");
    let unit = Unit::from_str(unit_str)?;
    Ok(unit.convert_from_wei(amount).to_string())
}

fn execute_to_wei_cmd(amount: &str, unit_str: &str) -> util::Res<String> {
    let amount = Decimal::from_str(amount)?;
    let unit = Unit::from_str(unit_str)?;
    Ok(unit.convert_to_wei(amount).to_string())
}