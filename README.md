# Normal run 
Run with
```
cargo run -- <IN> > <OUT>
```
If the return code is 0, then output file should contain a valid CSV.

# Error handling
Errors are just printed to stderr.

There are two sources of error:
* from Serialization/Deserialization, for which a serde's or io's error would be printed
* from the `Processor` itself, for which an `Error` enum is just debug-printed.

# Data type for the amount
Here, an f64 used for simplicity. For real finances, a proper decimal number representation is required, like a `rust_decimal` [crate](https://docs.rs/rust_decimal/latest/rust_decimal/).

# Tests
Integrational tests are in main.rs file. 

Unit-testing (without CSV processing) is also possible, but only a simple deposit-withdraw example is provided in processor.rs. 

# As library
The `Proccessor` module is written like a library. It can be used with a different parser or data streams. It does not require for the entire input data loaded into memory.

Example:
```rust
use processor::{Error, Operation, Processor, Transaction};
type ClientId = u16;

fn main() {
    let mut processor: Processor<ClientId> = Default::default();
    let good: Transaction<ClientId> = Transaction {
        r#type: Operation::Deposit,
        client: 1,
        tx: 1,
        amount: 100.0,
    };

    assert_eq!(Ok(()), processor.process(good));
}
```

Trait `FromIterator<Item = Transaction>` can be easily implemented with this approach.