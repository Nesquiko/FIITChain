use fiitcoin::{tx::Tx, utxo::UTXOPool};

use crate::{block::Block, tx_pool::TxPool};

pub const CUT_OFF_AGE: u32 = 12;

#[derive(Debug)]
pub struct Blockchain {}

impl Blockchain {
    pub fn new(genesis: Block) -> Self {
        Self {}
    }

    pub fn block_at_max_height(&self) -> &Block {
        todo!()
    }

    pub fn utxo_pool_at_max_height(&self) -> &UTXOPool {
        todo!()
    }

    pub fn tx_pool_at_max_height(&self) -> &TxPool {
        todo!()
    }

    pub fn add_block(&mut self, block: Block) -> bool {
        todo!()
    }

    pub fn add_tx(&mut self, tx: Tx) {
        todo!()
    }
}
