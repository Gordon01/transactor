use std::collections::HashMap;

use serde::Deserialize;
// «You can assume the type is a string, the client column is a valid u16 client ID, the tx is a valid u32
// transaction ID, and the amount is a decimal value with a precision of up to four places past the decimal»
#[derive(Debug, Deserialize)]
pub struct Transaction<K> {
    r#type: Operation,
    client: K,
    tx: u32,
    amount: f64,
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
#[derive(Debug)]
// We store this in the memory
pub struct Client {
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
    // Here a boolean is used to indicate if transaction is disputed.
    transactions: HashMap<u32, (f64, bool)>,
}

#[derive(Debug)]
pub enum Error {
    /// Attempted to witraw more funds than _available_. Note that _total_ amount may be higher.
    InsussficientFunds(f64),
    /// No recorded transaction with provided ID found.
    NoTransaction,
    /// Attempted to open a dispute to already disputed transaction.
    AlreadyDisputed,
    /// Attempted to close or chargeback on a non-disputed transaction.
    NotDisputed,
}

impl Client {
    pub fn deposit(&mut self, tx: u32, amount: f64) -> Result<(), Error> {
        self.available += amount;
        self.total += amount;
        // By design `tx` can't be the same, but our implementation does not catch this.
        // This can be converted to hard error easily, by checking the `transactions.contains_key()` first.
        self.transactions.insert(tx, (amount, false));
        Ok(())
    }

    /// «If a client does not have sufficient available funds the withdrawal should fail and the total amount
    /// of funds should not change.»
    ///
    /// On error, we also return how much exactly is insufficient just for fun.
    pub fn withdraw(&mut self, amount: f64) -> Result<(), Error> {
        if self.available < amount {
            return Err(Error::InsussficientFunds(amount - self.available));
        }
        self.available -= amount;
        self.total -= amount;
        Ok(())
    }

    pub fn dispute(&mut self, tx: u32) -> Result<(), Error> {
        match self.transactions.get_mut(&tx) {
            Some((amount, disputed)) => {
                if *disputed {
                    return Err(Error::AlreadyDisputed);
                }

                self.held += *amount;
                self.available -= *amount;
                *disputed = true;
                Ok(())
            }
            None => Err(Error::NoTransaction),
        }
    }

    // This is basically a reverse of the `dispute` method.
    pub fn resolve(&mut self, tx: u32) -> Result<(), Error> {
        match self.transactions.get_mut(&tx) {
            Some((amount, disputed)) => {
                if !*disputed {
                    return Err(Error::NotDisputed);
                }

                self.held -= *amount;
                self.available += *amount;
                *disputed = false;
                Ok(())
            }
            None => Err(Error::NoTransaction),
        }
    }

    pub fn chargeback(&mut self, tx: u32) -> Result<(), Error> {
        match self.transactions.get_mut(&tx) {
            Some((amount, disputed)) => {
                if !*disputed {
                    return Err(Error::NotDisputed);
                }

                self.held -= *amount;
                self.total -= *amount;
                *disputed = false;
                self.locked = true;
                Ok(())
            }
            None => Err(Error::NoTransaction),
        }
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
            transactions: HashMap::new(),
        });
        match transaction.r#type {
            // `tx` used as a new transaction ID.
            Operation::Deposit => client.deposit(transaction.tx, transaction.amount),

            // Unclear how `tx` should used, so just ignore it.
            Operation::Withdrawal => client.withdraw(transaction.amount),

            // `tx` is used as an existing transaction ID.
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