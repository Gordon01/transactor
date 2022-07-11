use std::{env, fs, io};

use csv::{ReaderBuilder, Trim};
use serde::Deserialize;

/// You can assume the type is a string, the client column is a valid u16 client ID, the tx is a valid u32
/// transaction ID, and the amount is a decimal value with a precision of up to four places past the decimal
#[derive(Debug, Deserialize)]
struct Transaction {
    r#type: String,
    client: u16,
    tx: u32,
    amount: f64,
}

fn main() {
    // Read a transactions file with the name provided as a first argument.
    // In the production project I would use the `clap` crate. In this prototype we only use one argument as defined
    // in the description of the problem, so `std::env::args()` is used.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: {} <transactions_file>", args[0]);
    }
    let transactions = fs::File::open(&args[1]).expect("Failed to open transactions file");

    let res = process(transactions);

    println!("{:?}", res);
}

// We accept any type of reader, so a file can be easily replaced with TCP stream as said in the problem description.
fn process(reader: impl io::Read) -> io::Result<String> {
    // "Whitespaces and decimal precisions (up to four places past the decimal) must be accepted by your program."
    let mut rdr = ReaderBuilder::new().trim(Trim::All).from_reader(reader);
    for result in rdr.deserialize() {
        let record: Transaction = result?;
        println!("{:?}", record);
    }

    Ok("lol ok".to_string())
}
