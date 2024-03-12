use std::collections::HashMap;

use fiitcoin::tx::Tx;

#[derive(Debug)]
pub struct TxPool {
    txs: HashMap<[u8; 32], Tx>,
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
