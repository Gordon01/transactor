mod processor;

use processor::{Operation, Processor, Transaction};
use processor_rpc::process_server::{Process, ProcessServer};
use processor_rpc::{Accounts, Transactions};
use rust_decimal::prelude::*;
use tonic::{transport::Server, Code, Request, Response, Status};

type ClientId = u16;

pub mod processor_rpc {
    tonic::include_proto!("processor");
}

#[derive(Debug, Default)]
pub struct MyProcessor {}

#[tonic::async_trait]
impl Process for MyProcessor {
    async fn process(&self, request: Request<Transactions>) -> Result<Response<Accounts>, Status> {
        let mut processor: Processor<ClientId> = Default::default();
        for record in request.into_inner().transactions {
            let operation = match record.r#type {
                0 => Operation::Deposit,
                1 => Operation::Withdrawal,
                2 => Operation::Dispute,
                3 => Operation::Resolve,
                4 => Operation::Chargeback,
                _ => return Err(Status::new(Code::InvalidArgument, "Invalid operation")),
            };

            let transaction: Transaction<ClientId> = Transaction {
                r#type: operation,
                client: record.client as u16,
                tx: record.tx,
                amount: Decimal::from_str(&record.amount).unwrap(),
            };
            // Sorry, errors are not handled here
            let _ = processor.process(transaction);
        }

        let mut accounts: Vec<processor_rpc::Account> = Vec::new();

        for (client, client_account) in processor.clients() {
            accounts.push(processor_rpc::Account {
                client: *client as u32,
                available: client_account.available.to_string(),
                held: client_account.held.to_string(),
                total: client_account.total.to_string(),
                locked: client_account.locked,
            });
        }

        let reply = processor_rpc::Accounts { accounts };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let processor = MyProcessor::default();

    Server::builder()
        .add_service(ProcessServer::new(processor))
        .serve(addr)
        .await?;

    Ok(())
}
