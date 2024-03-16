use fiitcoin::{handler::TxHandler, tx::Tx, utxo::UTXOPool};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

use crate::{block::Block, tx_pool::TxPool};

pub const CUT_OFF_AGE: usize = 12;

pub type BlockNode = (Block, UTXOPool);

#[derive(Debug)]
pub struct Blockchain {
    chain: ConstGenericRingBuffer<BlockNode, CUT_OFF_AGE>,
    mempool: TxPool,
}

impl Blockchain {
    pub fn new(genesis: Block, utxo_pool: UTXOPool) -> Self {
        let mut chain = ConstGenericRingBuffer::new();
        chain.push((genesis, utxo_pool));
        let mempool = TxPool::new();
        Self { chain, mempool }
    }

    pub fn at_block_hash(&self, hash: [u8; 32]) -> Option<&BlockNode> {
        self.chain.iter().find(|bn| bn.0.hash() == hash)
    }

    pub fn block_at_max_height(&self) -> &Block {
        &self
            .chain
            .back()
            .expect("can't have no blocks, where is genesis?")
            .0
    }

    pub fn utxo_pool_at_max_height(&self) -> &UTXOPool {
        &self
            .chain
            .back()
            .expect("can't have no utxo pools, where is genesis?")
            .1
    }

    pub fn tx_pool_at_max_height(&self) -> &TxPool {
        &self.mempool
    }

    pub fn add_block(&mut self, block: Block) -> bool {
        let node = match self.at_block_hash(block.prev()) {
            Some(parent) => parent,
            None => return false,
        };

        let mut handler = fiitcoin::handler::Handler::new(node.1.clone());
        let txs: Vec<&fiitcoin::tx::Tx> = block.txs().iter().map(|tx| tx).collect();

        if handler.handle(txs).len() != block.txs().len() {
            log::warn!("Block contained invalid txs!");
            return false;
        };

        for tx in block.txs().iter() {
            self.mempool.remove(tx.hash());
        }
        self.chain.push((block, handler.move_pool()));

        true
    }

    pub fn add_tx(&mut self, tx: Tx) {
        self.mempool.add(tx);
    }
}
