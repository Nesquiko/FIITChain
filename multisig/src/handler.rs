use std::collections::HashSet;

use rsa::signature::Verifier;

use crate::{
    tx::{Input, Tx},
    utxo::{UTXOPool, UTXO},
};

impl<'a> From<&'a Input> for UTXO {
    fn from(input: &'a Input) -> Self {
        UTXO::new(input.output_tx_hash(), input.output_idx())
    }
}

pub struct Handler {
    pool: UTXOPool,
}

impl Handler {
    pub fn new(pool: UTXOPool) -> Self {
        Self { pool }
    }

    /// Each epoch accepts unordered vector of proposed transactions.
    /// Checks validity of each, internally updates the UTXO pool, and
    /// returns vector of valid ones.
    ///
    /// # Beware
    /// Transactions can be dependent on other ones. Also, multiple
    /// transactions can reference same output.
    pub fn handle<'a>(&mut self, possible_txs: Vec<&'a Tx>) -> Vec<&'a Tx> {
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

    fn apply_tx(&mut self, tx: &Tx) {
        for input in tx.inputs().iter() {
            self.pool.remove_utxo(&input.into());
        }
        for (i, output) in tx.outputs().iter().enumerate() {
            let utxo = UTXO::new(tx.hash(), i.try_into().unwrap());
            // clone is here necessary, because I want to return the tx back to
            // caller, so I can't consume it
            self.pool.add_utxo(utxo, output.clone())
        }
    }

    /// Checks if:
    ///     1. All UTXO inputs are in pool
    ///     2. Signatures on inputs are valid and there are enough of them
    ///         to satisfy correspondings output multisig threshold
    ///     3. No UTXO is used more than once
    ///     4. Sum of outputs is not negative
    ///     5. Sum of inputs >= Sum of outputs
    pub fn is_tx_valid(&self, tx: &Tx) -> bool {
        let mut in_sum = 0;
        let mut used_outputs: HashSet<UTXO> = HashSet::new();
        for input in tx.inputs().into_iter() {
            if used_outputs.contains(&input.into()) {
                log::debug!(
                    "output {:?}-{} already used in same tx!",
                    input.output_tx_hash(),
                    input.output_idx()
                );
                return false;
            }
            used_outputs.insert(input.into());

            let output = match self.pool.utxo_output(&input.into()) {
                Some(out) => out,
                None => {
                    log::debug!(
                        "output from {:?} and index {} not found",
                        input.output_tx_hash(),
                        input.output_idx()
                    );
                    return false;
                }
            };

            let raw_tx = tx.raw_tx();
            let mut valid_sigs = 0;

            for signature in input.signatures().into_iter() {
                for verifier in output.verifiers().into_iter() {
                    if verifier.verify(&raw_tx, signature).is_ok() {
                        valid_sigs += 1;
                        break;
                    }
                }
            }
            if valid_sigs < output.threshold() {
                log::debug!(
                    "there were only {} valid signatures, need {}",
                    valid_sigs,
                    output.threshold()
                );
                return false;
            }

            in_sum += output.value();
        }

        let out_sum: u32 = tx.outputs().iter().map(|out| out.value()).sum();

        in_sum >= out_sum
    }

    /// Filters independent txs from dependent ones, applies them and returns both sets
    fn handle_independent<'a>(&mut self, txs: Vec<&'a Tx>) -> (Vec<&'a Tx>, Vec<&'a Tx>) {
        let mut handled = vec![];
        let mut dependent = vec![];
        let tx_set: HashSet<[u8; 32]> = txs.iter().map(|&tx| tx.hash()).collect();

        for &tx in txs.iter() {
            if tx.inputs().iter().all(|i| self.pool.contains(&i.into())) {
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
