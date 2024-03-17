use std::collections::{HashMap, HashSet};

use rsa::{pkcs1v15::Signature, signature::Verifier, RsaPublicKey};

use crate::{
    tx::{raw_tx_from_one_input, Input, Tx},
    utxo::{UTXOPool, UTXO},
};

pub fn balance_of(pool: &UTXOPool, pub_key: &RsaPublicKey) -> u64 {
    pool.utxos_of(pub_key)
        .iter()
        .map(|output| output.value() as u64)
        .sum()
}

pub trait TxHandler<'a> {
    /// Each epoch accepts unordered vector of proposed transactions.
    /// Checks validity of each, internally updates the UTXO pool, and
    /// returns vector of valid ones.
    ///
    /// # Beware
    /// Transactions can be dependent on other ones. Also, multiple
    /// transactions can reference same output.
    fn handle(&mut self, possible_txs: Vec<&'a Tx>) -> Vec<&'a Tx>;

    /// Returns reference to internal pool
    fn pool(&self) -> &UTXOPool;

    /// Returns mutable reference to internal pool
    fn pool_mut(&mut self) -> &mut UTXOPool;

    /// Moves internal pool, while consuming self
    fn move_pool(self) -> UTXOPool;

    /// Checks if:
    ///     1. All UTXO inputs are in pool
    ///     2. Signatures on inputs are valid
    ///     3. No UTXO is used more than once
    ///     4. Sum of outputs is not negative
    ///     5. Sum of inputs >= Sum of outputs
    fn is_tx_valid(&self, tx: &Tx) -> bool {
        let mut in_sum = 0;
        let mut used_outputs: HashSet<([u8; 32], u8)> = HashSet::new();
        for (i, input) in tx.inputs().iter().enumerate() {
            if used_outputs.contains(&(input.output_tx_hash(), input.output_idx())) {
                log::debug!(
                    "output {:?}-{} already used in same tx!",
                    input.output_tx_hash(),
                    input.output_idx()
                );
                return false;
            }
            used_outputs.insert((input.output_tx_hash(), input.output_idx()));

            let output = match self.pool().utxo_output(&input_to_utxo(input)) {
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

    /// Filters independent txs from dependent ones, applies them and returns both sets
    fn handle_independent(&mut self, txs: Vec<&'a Tx>) -> (Vec<&'a Tx>, Vec<&'a Tx>) {
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

    /// Applies given tx to the internal pool
    fn apply_tx(&mut self, tx: &Tx) {
        for input in tx.inputs().iter() {
            self.pool_mut().remove_utxo(&input_to_utxo(input));
        }
        for (i, output) in tx.outputs().iter().enumerate() {
            let utxo = UTXO::new(tx.hash(), i.try_into().unwrap());
            self.pool_mut().add_utxo(utxo, &output)
        }
    }

    fn is_input_in_pool(&self, input: &Input) -> bool {
        self.pool().contains(&input_to_utxo(input))
    }
}

pub struct Handler {
    pool: UTXOPool,
}

impl Handler {
    pub fn new(pool: UTXOPool) -> Self {
        Self { pool }
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

    fn pool_mut(&mut self) -> &mut UTXOPool {
        &mut self.pool
    }

    fn move_pool(self) -> UTXOPool {
        self.pool
    }

    fn apply_tx(&mut self, tx: &Tx) {
        for input in tx.inputs().iter() {
            self.pool.remove_utxo(&input_to_utxo(input));
        }
        for (i, output) in tx.outputs().iter().enumerate() {
            let utxo = UTXO::new(tx.hash(), i.try_into().unwrap());
            // clone is here necessary, because I want to return the tx back to
            // caller, so I can't consume it
            self.pool.add_utxo(utxo, &output)
        }
    }
}

pub struct MaxFeeHandler {
    pool: UTXOPool,
}

impl MaxFeeHandler {
    pub fn new(pool: UTXOPool) -> Self {
        Self { pool }
    }

    fn calc_fee(&self, tx: &Tx, tx_map: &HashMap<[u8; 32], &Tx>) -> Option<u64> {
        let mut input_value: u64 = 0;
        for input in tx.inputs().iter() {
            let output = match self.pool.utxo_output(&input_to_utxo(input)).or_else(|| {
                tx_map
                    .get(&input.output_tx_hash())?
                    .output(input.output_idx())
            }) {
                Some(output) => output,
                None => return None,
            };

            input_value += output.value() as u64;
        }

        let output_value: u64 = tx.outputs().iter().map(|o| o.value() as u64).sum();

        if input_value < output_value {
            return None;
        }
        Some(input_value - output_value)
    }
}

impl<'a> TxHandler<'a> for MaxFeeHandler {
    fn handle(&mut self, possible_txs: Vec<&'a Tx>) -> Vec<&'a Tx> {
        let tx_map: HashMap<[u8; 32], &'a Tx> =
            possible_txs.iter().map(|&tx| (tx.hash(), tx)).collect();

        let mut with_fees: Vec<(u64, &Tx)> = possible_txs
            .iter()
            .filter_map(|&tx| match self.calc_fee(tx, &tx_map) {
                Some(fee) => Some((fee, tx)),
                None => None,
            })
            .collect();
        with_fees.sort_unstable_by(|tx1, tx2| tx1.0.cmp(&tx2.0));
        with_fees.reverse();

        let mut handled: Vec<&'a Tx> = vec![];
        let mut to_handle = with_fees.iter().map(|tx| tx.1).collect();

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

    fn pool_mut(&mut self) -> &mut UTXOPool {
        &mut self.pool
    }

    fn move_pool(self) -> UTXOPool {
        self.pool
    }
}

fn input_to_utxo(input: &Input) -> UTXO {
    UTXO::new(input.output_tx_hash(), input.output_idx())
}
