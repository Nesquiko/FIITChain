use rsa::pkcs1v15::VerifyingKey;
use sha2::Sha256;

use crate::{
    block::{Block, IncompleteBlock},
    blockchain::Blockchain,
    handler::Handler,
    tx::Tx,
};

#[derive(Debug)]
pub struct BlockHandler {
    chain: Blockchain,
}

impl BlockHandler {
    pub fn new(chain: Blockchain) -> Self {
        Self { chain }
    }

    pub fn hash_at_max_height(&self) -> [u8; 32] {
        self.chain.block_at_max_height().hash()
    }

    pub fn process_block(&mut self, block: Block) -> bool {
        self.chain.add_block(block)
    }

    pub fn process_tx(&mut self, tx: Tx) {
        self.chain.add_tx(tx);
    }

    pub fn create_block(&self, address: Vec<&VerifyingKey<Sha256>>, threshold: usize) -> Block {
        let parent = self.chain.block_at_max_height();
        let mut new_b = IncompleteBlock::new(parent.hash(), address, threshold);

        let utxo_pool = self.chain.utxo_pool_at_max_height();
        let mut handler = Handler::new(utxo_pool.clone());

        let tx_pool = self.chain.tx_pool_at_max_height();
        let txs = tx_pool.txs();
        let handled = handler.handle(txs);

        for &tx in handled.iter() {
            new_b.add_tx(tx.clone());
        }
        new_b.finalize()
    }

    pub fn create_fork(
        &self,
        parent_hash: [u8; 32],
        addresses: Vec<&VerifyingKey<Sha256>>,
        threshold: usize,
    ) -> Option<Block> {
        let (parent, utxo_pool) = self.chain.at_block_hash(parent_hash)?;
        let mut new_b = IncompleteBlock::new(parent.hash(), addresses, threshold);
        let mut handler = Handler::new(utxo_pool.clone());

        let tx_pool = self.chain.tx_pool_at_max_height();
        let txs = tx_pool.txs();
        let handled = handler.handle(txs);

        for &tx in handled.iter() {
            new_b.add_tx(tx.clone());
        }
        Some(new_b.finalize())
    }
}
