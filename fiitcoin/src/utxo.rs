use std::collections::HashMap;

use crate::tx::{self};

#[derive(Eq, PartialEq, Hash)]
pub struct UTXO {
    /// hash of tx from which this utxo comes from
    tx_hash: [u8; 32],
    /// index at which this utxo is in tx
    output_idx: u8,
}

impl UTXO {
    pub fn new(tx_hash: [u8; 32], output_idx: u8) -> Self {
        Self {
            tx_hash,
            output_idx,
        }
    }
}

pub struct UTXOPool<'a> {
    /// collection of unspent UTXO mapped to corresponding tx output
    utxos: HashMap<UTXO, &'a tx::Output<'a>>,
}

impl<'a> UTXOPool<'a> {
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
        }
    }

    pub fn add_utxo(&mut self, utxo: UTXO, output: &'a tx::Output<'a>) {
        self.utxos.insert(utxo, output);
    }

    pub fn remove_utxo(&mut self, utxo: &UTXO) {
        self.utxos.remove(utxo);
    }

    pub fn utxo_output(&self, utxo: &UTXO) -> Option<&'a tx::Output<'a>> {
        self.utxos.get(utxo).map(|&out| out)
    }

    pub fn contains(&self, utxo: &UTXO) -> bool {
        self.utxos.contains_key(utxo)
    }
}
