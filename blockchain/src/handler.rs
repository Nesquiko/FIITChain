use fiitcoin::{
    handler::{Handler, TxHandler},
    tx::Tx,
};
use rsa::pkcs1v15::VerifyingKey;
use sha2::Sha256;

use crate::{
    block::{Block, IncompleteBlock},
    blockchain::Blockchain,
};

#[derive(Debug)]
pub struct BlockHandler {
    chain: Blockchain,
}

impl BlockHandler {
    pub fn new(chain: Blockchain) -> Self {
        Self { chain }
    }

    pub fn process_block(&mut self, block: Block) -> bool {
        self.chain.add_block(block)
    }

    pub fn process_tx(&mut self, tx: Tx) {
        self.chain.add_tx(tx);
    }

    pub fn create_block(&mut self, address: VerifyingKey<Sha256>) -> bool {
        let parent = self.chain.block_at_max_height();
        let mut new_b = IncompleteBlock::new(parent.hash(), address);

        let utxo_pool = self.chain.utxo_pool_at_max_height();
        let mut handler = Handler::new(utxo_pool.clone());

        let tx_pool = self.chain.tx_pool_at_max_height();
        let txs = tx_pool.txs();
        let handled = handler.handle(txs);

        for &tx in handled.iter() {
            new_b.add_tx(tx.clone());
        }
        let b = new_b.finalize();

        self.chain.add_block(b)
    }
}
