use handler::Handler;
use rsa::{pkcs1::DecodeRsaPrivateKey, pkcs1v15::SigningKey, RsaPrivateKey};
use sha2::{Digest, Sha256};
use tx::{Tx, TxError};
use utxo::{UTXOPool, UTXO};

mod handler;
mod tx;
mod utxo;

fn main() -> Result<(), FIITCoinError> {
    // generated with: openssl genrsa -traditional -out rsa_pk.pem 2048
    let bob_pem = include_str!("../../keys/bob_rsa_pk.pem");
    let bob_pk = RsaPrivateKey::from_pkcs1_pem(bob_pem)?;
    let bob_pubk = bob_pk.to_public_key();
    let bob_sk = SigningKey::<Sha256>::new(bob_pk);

    let alice_pem = include_str!("../../keys/alice_rsa_pk.pem");
    let alice_pk = RsaPrivateKey::from_pkcs1_pem(alice_pem)?;
    let alice_pubk = alice_pk.to_public_key();
    let alice_sk = SigningKey::<Sha256>::new(alice_pk);

    let mut hasher = sha2::Sha256::new();
    hasher.update("genesis-hash");
    let genesis_hash: [u8; 32] = hasher.finalize().into();

    let mut bob_tx = Tx::new();
    bob_tx.add_output(10, &bob_pubk);
    bob_tx.add_input(genesis_hash, 0);
    let bob_tx = bob_tx.sing_input_and_finalize(0, &bob_sk)?;

    let mut utxo_pool = UTXOPool::new();
    let genesis_utxo = UTXO::new(bob_tx.hash(), 0);
    match bob_tx.output(0) {
        Some(output) => utxo_pool.add_utxo(genesis_utxo, output),
        None => return Err(FIITCoinError::OutputIndexOutOfBounds(0)),
    }

    let mut alice_tx = Tx::new();
    alice_tx.add_input(bob_tx.hash(), 0);
    // split the 10 into outputs of 4, 3, 2 and 1 as fee
    alice_tx.add_output(4, &alice_pubk);
    alice_tx.add_output(3, &alice_pubk);
    alice_tx.add_output(2, &alice_pubk);
    let alice_tx = alice_tx.sing_input_and_finalize(0, &alice_sk)?;

    let mut handler = Handler::new(utxo_pool);
    let alice_tx_valid = handler.is_tx_valid(&alice_tx);
    assert!(alice_tx_valid);

    let possible_txs = handler.handle(vec![alice_tx]);
    println!("possible_txs: {:?}", possible_txs);
    assert_eq!(2, possible_txs.len());

    Ok(())
}

#[derive(Debug)]
pub enum FIITCoinError {
    RsaError(rsa::pkcs1::Error),
    TxError(tx::TxError),
    OutputIndexOutOfBounds(u8),
    Unknown,
}

impl From<rsa::pkcs1::Error> for FIITCoinError {
    fn from(error: rsa::pkcs1::Error) -> Self {
        Self::RsaError(error)
    }
}

impl From<tx::TxError> for FIITCoinError {
    fn from(error: TxError) -> Self {
        Self::TxError(error)
    }
}
