use fiitcoin::handler::Handler;
use fiitcoin::handler::TxHandler;
use fiitcoin::tx::{Tx, TxError};
use fiitcoin::utxo::{UTXOPool, UTXO};
use rand::Rng;
use rsa::pkcs1v15::VerifyingKey;
use rsa::signature::Keypair;
use rsa::{pkcs1::DecodeRsaPrivateKey, pkcs1v15::SigningKey, RsaPrivateKey};
use sha2::{Digest, Sha256};
use std::error::Error;

mod common;

#[test]
fn simulate_transactions() -> Result<(), Box<dyn Error>> {
    common::initialize();

    let rounds = 100;
    let max_tx_per_round = 20;
    let mut participants = (1..10)
        .into_iter()
        .map(|_| common::Participant::new())
        .collect::<Vec<_>>();
    participants[0].balance = 100;

    for _ in 0..rounds {
        let txs = rand::thread_rng().gen_range(0..max_tx_per_round);
        for _ in 0..txs {}
    }

    Ok(())
}

#[test]
fn transaction_is_valid_and_possible() -> Result<(), Box<dyn Error>> {
    common::initialize();

    // generated with: openssl genrsa -traditional -out rsa_pk.pem 2048
    let bob_pem = include_str!("../../keys/bob_rsa_pk.pem");
    let bob_pk = RsaPrivateKey::from_pkcs1_pem(bob_pem)?;
    let bob_sk = SigningKey::<Sha256>::new(bob_pk);
    let bob_vk = bob_sk.verifying_key();

    let alice_pem = include_str!("../../keys/alice_rsa_pk.pem");
    let alice_pk = RsaPrivateKey::from_pkcs1_pem(alice_pem)?;
    let alice_vk = VerifyingKey::new(alice_pk.to_public_key());

    let mut hasher = sha2::Sha256::new();
    hasher.update("genesis-hash");
    let genesis_hash: [u8; 32] = hasher.finalize().into();

    let mut bob_tx = Tx::new();
    bob_tx.add_output(10, &bob_vk);
    bob_tx.add_input(genesis_hash, 0);
    let root_bob_tx = bob_tx.sing_input_and_finalize(0, &bob_sk)?;

    let mut utxo_pool = UTXOPool::new();
    let root_utxo = UTXO::new(root_bob_tx.hash(), 0);
    match root_bob_tx.output(0) {
        Some(output) => utxo_pool.add_utxo(root_utxo, output),
        None => {
            log::debug!("bob tx output at index 0 out of bounds");
            return Err(Box::new(TxError::InputIndexOutOfBounds(0)));
        }
    }

    let mut tx_to_alice = Tx::new();
    tx_to_alice.add_input(root_bob_tx.hash(), 0);
    // split the 10 into outputs of 4, 3, 2 and 1 as fee
    tx_to_alice.add_output(4, &alice_vk);
    tx_to_alice.add_output(3, &alice_vk);
    tx_to_alice.add_output(2, &alice_vk);
    let alice_tx = tx_to_alice.sing_input_and_finalize(0, &bob_sk)?;

    let mut handler = Handler::new(utxo_pool);
    let alice_tx_valid = handler.is_tx_valid(&alice_tx);
    assert!(alice_tx_valid);

    let possible_txs = handler.handle(vec![alice_tx]);
    println!("possible_txs: {:?}", possible_txs);
    assert_eq!(1, possible_txs.len());

    Ok(())
}
