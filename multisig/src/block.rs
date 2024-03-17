use rsa::pkcs1v15::VerifyingKey;
use sha2::{Digest, Sha256};

use crate::tx::Tx;

pub const COINBASE: u32 = 625;

pub type Sha256Digest = [u8; 32];

#[derive(Debug)]
pub struct IncompleteBlock {
    prev: Sha256Digest,
    coinbase: Tx,
    txs: Vec<Tx>,
}

impl IncompleteBlock {
    pub fn new(prev: Sha256Digest, address: Vec<&VerifyingKey<Sha256>>, threshold: usize) -> Self {
        let coinbase = Tx::coinbase(COINBASE, address, threshold);
        Self {
            prev,
            coinbase,
            txs: vec![],
        }
    }

    pub fn finalize(self) -> Block {
        let raw = self.raw();

        let mut hasher = Sha256::new();
        hasher.update(raw);

        Block {
            hash: hasher.finalize().into(),
            prev: self.prev,
            coinbase: self.coinbase,
            txs: self.txs,
        }
    }

    pub fn add_tx(&mut self, tx: Tx) {
        self.txs.push(tx);
    }

    fn raw(&self) -> Vec<u8> {
        let mut b = vec![];

        if !self.prev.iter().all(|&x| x == 0) {
            // not a genesis block
            b.extend(self.prev);
        }

        for tx in self.txs.iter() {
            b.extend(tx.hash());
        }

        b
    }
}

#[derive(Debug)]
pub struct Block {
    hash: Sha256Digest,
    prev: Sha256Digest,
    coinbase: Tx,
    txs: Vec<Tx>,
}

impl Block {
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }

    pub fn coinbase(&self) -> &Tx {
        &self.coinbase
    }

    pub fn txs(&self) -> &Vec<Tx> {
        &self.txs
    }

    pub fn prev(&self) -> [u8; 32] {
        self.prev
    }

    /// # DO NOT USE, don't use this function outside tests!
    pub fn set_prev(&mut self, prev: Sha256Digest) {
        self.prev = prev;
    }
}
