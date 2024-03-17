use std::collections::HashMap;

use crate::tx::{Hash, Tx};

#[derive(Debug)]
pub struct TxPool {
    txs: HashMap<Hash, Tx>,
}

impl TxPool {
    pub fn new() -> Self {
        Self {
            txs: HashMap::new(),
        }
    }

    pub fn txs(&self) -> Vec<&Tx> {
        self.txs.values().collect()
    }

    pub fn tx(&self, hash: [u8; 32]) -> Option<&Tx> {
        self.txs.get(&hash)
    }

    pub fn add(&mut self, tx: Tx) {
        self.txs.insert(tx.hash(), tx);
    }

    pub fn remove(&mut self, hash: [u8; 32]) {
        self.txs.remove(&hash);
    }
}
