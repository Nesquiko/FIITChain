use std::collections::HashMap;

use rsa::pkcs1v15::VerifyingKey;
use sha2::Sha256;

use crate::tx::{self};

#[derive(Eq, PartialEq, Hash, Clone)]
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

#[derive(Clone)]
pub struct UTXOOutput<'a> {
    output: &'a tx::Output,
    used: bool,
}

impl<'a> UTXOOutput<'a> {
    pub fn verifying_key(&self) -> &VerifyingKey<Sha256> {
        self.output.verifying_key()
    }

    pub fn used(&self) -> bool {
        self.used
    }

    pub fn value(&self) -> u32 {
        self.output.value()
    }

    pub fn set_used(&mut self, used: bool) {
        self.used = used;
    }
}

#[derive(Clone)]
pub struct UTXOPool<'a> {
    /// collection of unspent UTXO mapped to corresponding tx output
    utxos: HashMap<UTXO, UTXOOutput<'a>>,
}

impl<'a> UTXOPool<'a> {
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
        }
    }

    pub fn add_utxo(&mut self, utxo: UTXO, output: &'a tx::Output) {
        self.utxos.insert(
            utxo,
            UTXOOutput {
                output,
                used: false,
            },
        );
    }

    pub fn remove_utxo(&mut self, utxo: &UTXO) {
        self.utxos.remove(utxo);
    }

    pub fn utxo_output(&self, utxo: &UTXO) -> Option<&'a UTXOOutput> {
        self.utxos.get(utxo)
    }

    pub fn set_utxo_as_used(&mut self, utxo: &UTXO) {
        self.utxos.get_mut(utxo).unwrap().used = true;
    }

    pub fn contains(&self, utxo: &UTXO) -> bool {
        self.utxos.contains_key(utxo)
    }
}
