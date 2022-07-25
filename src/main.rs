mod processor;

use std::{env, error::Error, fs, io};

use csv::{ReaderBuilder, Trim, Writer};
use processor::Processor;
use rust_decimal::prelude::*;
use serde::Serialize;

// This is only needed for the CSV output
#[derive(Debug, Serialize)]
struct ClientOut {
    #[serde(rename = "client")]
    id: u16,
    #[serde(with = "rust_decimal::serde::str")]
    available: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    held: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    total: Decimal,
    locked: bool,
}

fn main() {
    // Read a transactions file with the name provided as a first argument.
    // In the production project I would use the `clap` crate. In this prototype we only use one argument as defined
    // in the description of the problem, so `std::env::args()` is used.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: {} <transactions_file>", args[0]);
    }
    let transactions = fs::File::open(&args[1]).expect("open transactions file");

    // «Output should be written to std out»
    // Can be easly replaced with a file or stream if needed.
    process(transactions, io::stdout()).expect("process transactions");
}

// We accept any type of reader, so a file can be easily replaced with TCP stream as said in the problem description.
pub fn process(reader: impl io::Read, writer: impl io::Write) -> Result<(), Box<dyn Error>> {
    // «Whitespaces and decimal precisions (up to four places past the decimal) must be accepted by your program.»
    let mut rdr = ReaderBuilder::new().flexible(true).trim(Trim::All).from_reader(reader);
    // Deserealization errors are ignored
    let iter = rdr.deserialize().filter_map(|r| r.ok());
    let processor = Processor::from_iter(iter);

    let mut wtr = Writer::from_writer(writer);

    for (&id, client) in processor.clients() {
        let client_out = ClientOut {
            id,
            available: client.available,
            held: client.held,
            total: client.total,
            locked: client.locked,
        };
        wtr.serialize(&client_out)?;
    }
    wtr.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // We use HashMap iterator for tests, so we need to sort output for consistency.
    fn sort_output_by_client_id(string: &mut String) {
        let mut lines: Vec<_> = string.lines().collect();
        let slice = &mut lines[1..];
        slice.sort_by(|a, b| {
            let a_id: u16 = a.split(",").next().unwrap().parse().unwrap();
            let b_id: u16 = b.split(",").next().unwrap().parse().unwrap();
            a_id.cmp(&b_id)
        });
        *string = lines.join("\n");
    }

    // Wrap up `process()` a little more to get rid of the all repetitive boilerplate.
    fn run_process(input: &str) -> String {
        let mut result = Vec::new();
        super::process(input.as_bytes(), &mut result).expect("process transactions");
        let mut data = String::from_utf8(result).expect("convert result to string");
        sort_output_by_client_id(&mut data);

        data
    }

    #[test]
    fn deposits() {
        let transactions = "\
type,client,tx,amount
deposit,1,1,100.50
deposit,1,2,200.05
deposit,2,1,300.01
deposit,2,2,400.0001";
        let output = "\
client,available,held,total,locked
1,300.55,0,300.55,false
2,700.0101,0,700.0101,false";
        assert_eq!(run_process(transactions), output);
    }

    #[test]
    fn deposits_and_withdrawals() {
        let transactions = "\
type,client,tx,amount
deposit,1,1,100.00
withdrawal,1,2,200.00
deposit,2,1,300.00
withdrawal,2,2,200.00";
        let output = "\
client,available,held,total,locked
1,100,0,100,false
2,100,0,100,false";
        assert_eq!(run_process(transactions), output);
    }

    #[test]
    fn dispute() {
        let transactions = "\
type,client,tx,amount
deposit,1,1,100.00
dispute,1,1";
        let output = "\
client,available,held,total,locked
1,0,100,100,false";
        assert_eq!(run_process(transactions), output);
    }

    #[test]
    fn dispute_no_id() {
        let transactions = "\
type,client,tx,amount
deposit,1,1,100.00
dispute,1,2";
        let output = "\
client,available,held,total,locked
1,100,0,100,false";
        assert_eq!(run_process(transactions), output);
    }

    #[test]
    fn dispute_and_resolve() {
        let transactions = "\
type,client,tx,amount
deposit,1,1,100.00
dispute,1,1
resolve,1,1";
        let output = "\
client,available,held,total,locked
1,100,0,100,false";
        assert_eq!(run_process(transactions), output);
    }

    #[test]
    fn dispute_and_chargeback() {
        let transactions = "\
type,client,tx,amount
deposit,1,1,100.00
deposit,1,2,200.00
dispute,1,2
chargeback,1,2";
        let output = "\
client,available,held,total,locked
1,100,0,100,true";
        assert_eq!(run_process(transactions), output);
    }

    #[test]
    fn chargeback_wrong_state() {
        let transactions = "\
type,client,tx,amount
deposit,1,1,100.00
deposit,1,2,200.00
chargeback,1,2";
        let output = "\
client,available,held,total,locked
1,300,0,300,false";
        assert_eq!(run_process(transactions), output);
    }
}
