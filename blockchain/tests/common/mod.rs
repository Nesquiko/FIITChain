use std::sync::Once;

use blockchain::{
    block::{Block, IncompleteBlock},
    blockchain::Blockchain,
    handler::BlockHandler,
};
use fiitcoin::{
    tx::{Tx, UnsignedTx},
    utxo::{UTXOPool, UTXO},
};
use rsa::{
    pkcs1v15::{SigningKey, VerifyingKey},
    signature::Keypair,
    RsaPrivateKey,
};
use sha2::Sha256;

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

pub struct Participant {
    pub sk: SigningKey<Sha256>,
    pub vk: VerifyingKey<Sha256>,
}

/// Only run in tests! Generated keys are of length 1024, not sufficiently
/// secure by today's standards.
impl Participant {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let bits = 1024;
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let sk = SigningKey::<Sha256>::new(priv_key);
        let vk = sk.verifying_key();

        Self { sk, vk }
    }
}

pub struct NewTxParams<'a> {
    pub sender: &'a Participant,
    pub inputs: &'a [(&'a Tx, u8)],
    pub outputs: &'a [(&'a Participant, u32)],
    pub return_to_sender: Option<u32>,
}

pub fn new_tx(params: NewTxParams) -> Tx {
    let tx = create_unsigned_tx(&params);
    tx.sing_inputs_and_finalize(&params.sender.sk).unwrap()
}

fn create_unsigned_tx(params: &NewTxParams) -> UnsignedTx {
    let NewTxParams {
        sender,
        inputs,
        outputs,
        return_to_sender,
    } = params;

    let mut tx = UnsignedTx::new();
    for input in inputs.iter() {
        tx.add_input(input.0.hash(), input.1);
    }
    for output in outputs.iter() {
        tx.add_output(output.1, &output.0.vk);
    }

    if let Some(to_return) = return_to_sender {
        tx.add_output(*to_return, &sender.vk);
    }
    tx
}

pub fn setup_handler(receiver: &Participant) -> (BlockHandler, Tx) {
    let genesis = IncompleteBlock::new([0; 32], &receiver.vk).finalize();
    let (pool, genesis_tx) = setup_pool(&genesis);
    let chain = Blockchain::new(genesis, pool);
    (BlockHandler::new(chain), genesis_tx)
}

pub fn setup_pool(genesis_block: &Block) -> (UTXOPool, Tx) {
    let mut utxo_pool = UTXOPool::new();
    let coinbase = genesis_block.coinbase();
    let root_utxo = UTXO::new(coinbase.hash(), 0);
    utxo_pool.add_utxo(root_utxo, &coinbase.output(0).unwrap());
    (utxo_pool, coinbase.clone())
}
