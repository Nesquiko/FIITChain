use multisig::{
    block::{Block, IncompleteBlock},
    block_handler::BlockHandler,
    blockchain::Blockchain,
    handler::Handler,
    tx::{Tx, UnsignedTx},
    utxo::{UTXOPool, UTXO},
};
use rsa::{
    pkcs1v15::{SigningKey, VerifyingKey},
    signature::Keypair,
    RsaPrivateKey,
};
use sha2::{Digest, Sha256};
use std::sync::Once;

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

#[derive(Debug, Clone)]
pub struct Wallet {
    keys: Vec<KeyPair>,
    threshold: usize,
}

impl Wallet {
    pub fn new(key: KeyPair) -> Self {
        Self {
            keys: vec![key],
            threshold: 1,
        }
    }

    pub fn random(n: usize, threshold: usize) -> Self {
        let mut keys = vec![];
        for _ in 0..n {
            keys.push(KeyPair::new());
        }
        Self { keys, threshold }
    }

    pub fn multisig(keys: Vec<KeyPair>, threshold: usize) -> Self {
        Self { keys, threshold }
    }

    pub fn keys(&self) -> &[KeyPair] {
        &self.keys
    }

    pub fn verifiers(&self) -> Vec<&VerifyingKey<Sha256>> {
        self.keys.iter().map(|kp| &kp.vk).collect()
    }

    pub fn threshold(&self) -> usize {
        self.threshold
    }
}

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub sk: SigningKey<Sha256>,
    pub vk: VerifyingKey<Sha256>,
}

impl KeyPair {
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
    pub signer: &'a Wallet,
    pub inputs: Vec<UTXO>,
    pub outputs: Vec<(&'a Wallet, u32)>,
    pub return_to_sender: Option<u32>,
}

pub fn new_tx(params: NewTxParams) -> Tx {
    let tx = create_unsigned_tx(&params);
    let signers: Vec<&SigningKey<Sha256>> = params.signer.keys().iter().map(|s| &s.sk).collect();
    tx.finalize(signers)
}

pub fn new_tx_first_n_signers_only(params: NewTxParams, n: usize) -> Tx {
    let tx = create_unsigned_tx(&params);
    let signers: Vec<&SigningKey<Sha256>> = params.signer.keys().iter().map(|s| &s.sk).collect();
    tx.finalize(signers[0..n].to_vec())
}

fn create_unsigned_tx(params: &NewTxParams) -> UnsignedTx {
    let NewTxParams {
        signer,
        inputs,
        outputs,
        return_to_sender,
    } = params;

    let mut tx = UnsignedTx::new();
    for input in inputs.iter() {
        tx.add_input(input.tx_hash(), input.output_idx());
    }
    for output in outputs.iter() {
        let verifiers = output.0.keys.iter().map(|kp| &kp.vk).collect();
        tx.add_output(output.1, verifiers, output.0.threshold);
    }

    if let Some(to_return) = return_to_sender {
        let sks = signer.keys.iter().map(|kp| &kp.vk).collect();
        tx.add_output(*to_return, sks, signer.threshold);
    }
    tx
}

pub fn setup_block_handler(receiver: &Wallet) -> (BlockHandler, Tx) {
    let verifiers = receiver.verifiers();
    let genesis = IncompleteBlock::new([0; 32], verifiers, receiver.threshold).finalize();
    let (pool, genesis_tx) = setup_genesis_pool(&genesis);
    let chain = Blockchain::new(genesis, pool);
    (BlockHandler::new(chain), genesis_tx)
}

pub fn setup_genesis_pool(genesis_block: &Block) -> (UTXOPool, Tx) {
    let mut utxo_pool = UTXOPool::new();
    let coinbase = genesis_block.coinbase();
    let root_utxo = UTXO::new(coinbase.hash(), 0);
    utxo_pool.add_utxo(root_utxo, &coinbase.output(0).unwrap());
    (utxo_pool, coinbase.clone())
}

pub fn setup_handler(receiver: &Wallet, output_value: u32, root_outputs: u8) -> (Handler, Tx) {
    let (utxo_pool, genesis_tx) = setup_pool(receiver, output_value, root_outputs);
    (Handler::new(utxo_pool), genesis_tx)
}

pub fn setup_pool(receiver: &Wallet, output_value: u32, root_outputs: u8) -> (UTXOPool, Tx) {
    let mut hasher = Sha256::new();
    hasher.update("genesis-hash");
    let genesis_hash: [u8; 32] = hasher.finalize().into();

    let mut root_tx = UnsignedTx::new();
    for _ in 0..root_outputs {
        let verifier_keys = receiver.keys.iter().map(|kp| &kp.vk).collect();
        root_tx.add_output(output_value, verifier_keys, receiver.threshold);
    }
    root_tx.add_input(genesis_hash, 0);
    let signers = receiver.keys.iter().map(|kp| &kp.sk).collect();
    let root_tx = root_tx.finalize(signers);

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
