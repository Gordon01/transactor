use std::collections::HashMap;

use crate::{Operation, Transaction};

#[derive(Debug)]
// We store this in the memory
pub struct Client {
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

#[derive(Debug)]
pub enum Error {
    InsussficientFunds(f64),
}

impl Client {
    pub fn deposit(&mut self, tx: u32, amount: f64) -> Result<(), Error> {
        self.available += amount;
        self.total += amount;
        Ok(())
    }

    /// «If a client does not have sufficient available funds the withdrawal should fail and the total amount
    /// of funds should not change.»
    ///
    /// On error, we also return how much exactly is insufficient just for fun.
    pub fn withdraw(&mut self, tx: u32, amount: f64) -> Result<(), Error> {
        if self.available < amount {
            return Err(Error::InsussficientFunds(amount - self.available));
        }
        self.available -= amount;
        self.total -= amount;
        Ok(())
    }

    pub fn dispute(&mut self, tx: u32) -> Result<(), Error> {
        unimplemented!();
    }

    pub fn resolve(&mut self, tx: u32) -> Result<(), Error> {
        unimplemented!();
    }

    pub fn chargeback(&mut self, tx: u32) -> Result<(), Error> {
        unimplemented!();
    }
}

pub struct Processor<K> {
    clients: HashMap<K, Client>,
}

impl<K> Processor<K>
where
    K: std::hash::Hash + Eq + std::fmt::Debug,
{
    pub fn process(&mut self, transaction: Transaction<K>) -> Result<(), Error> {
        let client = self.clients.entry(transaction.client).or_insert(Client {
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        });
        match transaction.r#type {
            Operation::Deposit => client.deposit(transaction.tx, transaction.amount),
            Operation::Withdrawal => client.withdraw(transaction.tx, transaction.amount),
            Operation::Dispute => client.dispute(transaction.tx),
            Operation::Resolve => client.resolve(transaction.tx),
            Operation::Chargeback => client.chargeback(transaction.tx),
        }
    }

    pub fn clients(&self) -> &HashMap<K, Client> {
        &self.clients
    }
}

impl<K> Default for Processor<K> {
    fn default() -> Self {
        Processor {
            clients: HashMap::new(),
        }
    }
}
