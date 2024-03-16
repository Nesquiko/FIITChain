use std::collections::HashMap;

use rsa::RsaPublicKey;

use crate::tx::Output;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct UTXOPool {
    /// collection of unspent UTXO mapped to corresponding tx output
    utxos: HashMap<UTXO, Output>,
}

impl UTXOPool {
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
        }
    }

    pub fn add_utxo(&mut self, utxo: UTXO, output: &Output) {
        self.utxos.insert(utxo, output.clone());
    }

    pub fn remove_utxo(&mut self, utxo: &UTXO) {
        self.utxos.remove(utxo);
    }

    pub fn utxo_output(&self, utxo: &UTXO) -> Option<&Output> {
        self.utxos.get(utxo)
    }

    pub fn contains(&self, utxo: &UTXO) -> bool {
        self.utxos.contains_key(utxo)
    }

    pub fn utxos_of(&self, pub_key: &RsaPublicKey) -> Vec<&Output> {
        self.utxos
            .values()
            .filter(|o| o.verifying_key().as_ref() == pub_key)
            .collect()
    }
}
