use crate::{
    tx::{Finalized, Tx},
    utxo::UTXOPool,
};

pub struct Handler<'a> {
    pool: UTXOPool<'a>,
}

impl<'a> Handler<'a> {
    pub fn new(pool: UTXOPool<'a>) -> Self {
        Self { pool }
    }

    /// Each epoch accepts unordered vector of proposed transactions.
    /// Checks validity of each, internally updates the UTXO pool, and
    /// returns vector of valid ones.
    ///
    /// # Beware
    /// Transactions can be dependent on other ones. Also, multiple
    /// transactions can reference same output.
    pub fn handle(&mut self, possible_txs: Vec<Tx<Finalized>>) -> Vec<Tx<Finalized>> {
        todo!()
    }

    /// Checks if:
    ///     1. All UTXO inputs are in pool
    ///     2. Signatures on inputs are valid
    ///     3. No UTXO is used more than once
    ///     4. Sum of outputs is not negative
    ///     5. Sum of inputs >= Sum of outputs
    pub fn is_tx_valid(&self, tx: &Tx<Finalized>) -> bool {
        todo!()
    }
}
