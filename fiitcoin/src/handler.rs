use rsa::{pkcs1v15::Signature, signature::Verifier};

use crate::{
    tx::{Finalized, Input, Tx},
    utxo::{UTXOPool, UTXO},
};

pub trait TxHandler<'a> {
    /// Each epoch accepts unordered vector of proposed transactions.
    /// Checks validity of each, internally updates the UTXO pool, and
    /// returns vector of valid ones.
    ///
    /// # Beware
    /// Transactions can be dependent on other ones. Also, multiple
    /// transactions can reference same output.
    fn handle(&mut self, possible_txs: Vec<Tx<Finalized>>) -> Vec<Tx<Finalized>>;

    /// Checks if:
    ///     1. All UTXO inputs are in pool
    ///     2. Signatures on inputs are valid
    ///     3. No UTXO is used more than once
    ///     4. Sum of outputs is not negative
    ///     5. Sum of inputs >= Sum of outputs
    fn is_tx_valid(&self, tx: &Tx<Finalized>) -> bool;
}

pub struct Handler<'a> {
    pool: UTXOPool<'a>,
}

impl<'a> Handler<'a> {
    pub fn new(pool: UTXOPool<'a>) -> Self {
        Self { pool }
    }

    fn input_to_utxo(&self, input: &Input) -> UTXO {
        UTXO::new(input.output_tx_hash(), input.output_idx())
    }

    fn apply_tx(&mut self, tx: &Tx<Finalized>) {
        for input in tx.inputs().iter() {
            self.pool.set_utxo_as_used(&self.input_to_utxo(input));
        }
    }
}

impl<'a> TxHandler<'a> for Handler<'a> {
    fn handle(&mut self, possible_txs: Vec<Tx<Finalized>>) -> Vec<Tx<Finalized>> {
        possible_txs
            .into_iter()
            .filter(|tx| {
                if !self.is_tx_valid(tx) {
                    return false;
                }
                self.apply_tx(tx);
                true
            })
            .collect()
    }

    fn is_tx_valid(&self, tx: &Tx<Finalized>) -> bool {
        let mut in_sum = 0;
        for (i, input) in tx.inputs().iter().enumerate() {
            let output = match self.pool.utxo_output(&self.input_to_utxo(input)) {
                Some(out) => out,
                None => {
                    log::debug!(
                        "output from {:x?} and index {} not found",
                        input.output_tx_hash(),
                        input.output_idx()
                    );
                    return false;
                }
            };

            let signature = match input.signature() {
                Some(sig) => sig,
                None => {
                    log::debug!("unsigned signature here?");
                    return false;
                }
            };

            let signature = match Signature::try_from(signature.as_ref()) {
                Ok(sig) => sig,
                Err(err) => {
                    log::debug!("failed to convert into signature, {:?}", err);
                    return false;
                }
            };

            let raw_tx = match tx.raw_tx_from_one_input(i.try_into().unwrap()) {
                Ok(raw) => raw,
                Err(err) => {
                    log::debug!("failed to get raw tx, {:?}", err);
                    return false;
                }
            };

            match output.verifying_key().verify(&raw_tx, &signature) {
                Ok(_) => {}
                Err(err) => {
                    log::debug!("invalid signature, {:?}", err);
                    return false;
                }
            }

            if output.used() {
                log::debug!(
                    "output from {:x?} and index {} not found",
                    input.output_tx_hash(),
                    input.output_idx()
                );
                return false;
            }

            in_sum += output.value();
        }

        let out_sum: u32 = tx.outputs().iter().map(|out| out.value()).sum();

        out_sum > 0 && in_sum >= out_sum
    }
}
