mod processor;

use std::{env, error::Error, fs, io};

use csv::{ReaderBuilder, Trim, Writer};
use processor::Processor;
use serde::{Deserialize, Serialize};

// We use this type both in the input and for HashMap index so giving it a name may improve readability.
type ClientId = u16;

// «You can assume the type is a string, the client column is a valid u16 client ID, the tx is a valid u32
// transaction ID, and the amount is a decimal value with a precision of up to four places past the decimal»
#[derive(Debug, Deserialize)]
pub struct Transaction<K> {
    r#type: Operation,
    client: K,
    tx: u32,
    amount: f64,
}

// This is only needed for the CSV output
#[derive(Debug, Serialize)]
struct ClientOut {
    #[serde(rename = "client")]
    id: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Operation {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
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
fn process(reader: impl io::Read, writer: impl io::Write) -> Result<(), Box<dyn Error>> {
    // «Whitespaces and decimal precisions (up to four places past the decimal) must be accepted by your program.»
    let mut rdr = ReaderBuilder::new().trim(Trim::All).from_reader(reader);

    let mut processor: Processor<ClientId> = Default::default();
    for result in rdr.deserialize() {
        let record: Transaction<ClientId> = result?;
        if let Err(e) = processor.process(record) {
            eprintln!("{:?}", e);
        }
    }

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
