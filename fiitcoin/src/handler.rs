use std::collections::HashSet;

use rsa::{pkcs1v15::Signature, signature::Verifier, RsaPublicKey};

use crate::{
    tx::{raw_tx_from_one_input, Input, Tx},
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
    fn handle(&mut self, possible_txs: Vec<&'a Tx>) -> Vec<&'a Tx>;

    fn pool(&self) -> &UTXOPool;

    /// Checks if:
    ///     1. All UTXO inputs are in pool
    ///     2. Signatures on inputs are valid
    ///     3. No UTXO is used more than once
    ///     4. Sum of outputs is not negative
    ///     5. Sum of inputs >= Sum of outputs
    fn is_tx_valid(&self, tx: &Tx) -> bool {
        let mut in_sum = 0;
        for (i, input) in tx.inputs().iter().enumerate() {
            let output = match self.pool().utxo_output(&input_to_utxo(input)) {
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

            let raw_tx =
                match raw_tx_from_one_input(tx.inputs(), tx.outputs(), i.try_into().unwrap()) {
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

            in_sum += output.value();
        }

        let out_sum: u32 = tx.outputs().iter().map(|out| out.value()).sum();

        out_sum > 0 && in_sum >= out_sum
    }
}

pub struct Handler {
    pool: UTXOPool,
}

impl Handler {
    pub fn new(pool: UTXOPool) -> Self {
        Self { pool }
    }

    pub fn balance_of(&self, pub_key: &RsaPublicKey) -> u64 {
        self.pool
            .utxos_of(pub_key)
            .iter()
            .map(|output| output.value() as u64)
            .sum()
    }

    fn apply_tx(&mut self, tx: &Tx) {
        for input in tx.inputs().iter() {
            self.pool.remove_utxo(&input_to_utxo(input));
        }
        for (i, output) in tx.outputs().iter().enumerate() {
            let utxo = UTXO::new(tx.hash(), i.try_into().unwrap());
            // clone is here necessary, because I want to return the tx back to
            // caller, so I can't consume it
            self.pool.add_utxo(utxo, output.clone())
        }
    }

    fn is_input_in_pool(&self, input: &Input) -> bool {
        self.pool.contains(&input_to_utxo(input))
    }

    /// Filters independent txs from dependent ones, applies them and returns both sets
    fn handle_independent<'a>(&mut self, txs: Vec<&'a Tx>) -> (Vec<&'a Tx>, Vec<&'a Tx>) {
        let mut handled = vec![];
        let mut dependent = vec![];
        let tx_set: HashSet<[u8; 32]> = txs.iter().map(|&tx| tx.hash()).collect();

        for &tx in txs.iter() {
            if tx.inputs().iter().all(|i| self.is_input_in_pool(i)) {
                // tx is only dependent on outputs in pool
                if self.is_tx_valid(tx) {
                    self.apply_tx(tx);
                    handled.push(tx);
                }
            } else if tx
                .inputs()
                .iter()
                .any(|i| tx_set.contains(&i.output_tx_hash()))
            {
                // tx is dependent on some outputs from this batch
                dependent.push(tx)
            }
        }

        (handled, dependent)
    }
}

impl<'a> TxHandler<'a> for Handler {
    fn handle(&mut self, possible_txs: Vec<&'a Tx>) -> Vec<&'a Tx> {
        let mut handled: Vec<&'a Tx> = vec![];
        let mut to_handle = possible_txs;

        loop {
            let (independent, dependent) = self.handle_independent(to_handle);
            handled.extend(independent);
            if dependent.is_empty() {
                break;
            }
            to_handle = dependent;
        }

        handled
    }

    fn pool(&self) -> &UTXOPool {
        &self.pool
    }
}

pub struct MaxFeeHandler {
    pool: UTXOPool,
}

impl<'a> TxHandler<'a> for MaxFeeHandler {
    fn handle(&mut self, possible_txs: Vec<&'a Tx>) -> Vec<&'a Tx> {
        todo!()
    }

    fn pool(&self) -> &UTXOPool {
        &self.pool
    }
}

fn input_to_utxo(input: &Input) -> UTXO {
    UTXO::new(input.output_tx_hash(), input.output_idx())
}
