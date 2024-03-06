use std::sync::Once;

use fiitcoin::{
    tx::{Tx, UnsignedTx},
    utxo::{UTXOPool, UTXO},
};
use rsa::{
    pkcs1v15::{SigningKey, VerifyingKey},
    signature::Keypair,
    RsaPrivateKey,
};
use sha2::{Digest, Sha256};

static INIT: Once = Once::new();

pub const OUTPUT_VALUE: u32 = 100;

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

pub fn new_tx_forged_signatures(params: NewTxParams, adversary: &Participant) -> Tx {
    let tx = create_unsigned_tx(&params);
    tx.sing_inputs_and_finalize(&adversary.sk).unwrap()
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

pub fn setup_pool(receiver: &Participant, output_value: u32, root_outputs: u8) -> (UTXOPool, Tx) {
    let mut hasher = Sha256::new();
    hasher.update("genesis-hash");
    let genesis_hash: [u8; 32] = hasher.finalize().into();

    let mut root_tx = UnsignedTx::new();
    for _ in 0..root_outputs {
        root_tx.add_output(output_value, &receiver.vk);
    }
    root_tx.add_input(genesis_hash, 0);
    let root_tx = root_tx.sing_inputs_and_finalize(&receiver.sk).unwrap();

    let mut utxo_pool = UTXOPool::new();

    for output_idx in 0..root_tx.output_len() {
        let output_idx = output_idx.try_into().unwrap();
        let root_utxo = UTXO::new(root_tx.hash(), output_idx);
        match root_tx.output(output_idx) {
            Some(output) => utxo_pool.add_utxo(root_utxo, &output),
            None => panic!(
                "tx output at index {} out of bounds, outputs len {}",
                output_idx,
                root_tx.output_len()
            ),
        }
    }

    (utxo_pool, root_tx)
}
