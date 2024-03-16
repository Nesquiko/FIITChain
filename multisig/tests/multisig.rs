mod common;

use common::{initialize, new_tx, KeyPair, NewTxParams};
use multisig::utxo::UTXO;

use crate::common::{new_tx_first_n_signers_only, setup_handler, Wallet};

#[test]
fn normal_tx() {
    initialize();

    let bob = Wallet::random(1, 1);
    let alice = Wallet::random(1, 1);
    let (handler, genesis_tx) = setup_handler(&bob, 500, 1);

    let tx1 = new_tx(NewTxParams {
        signer: &bob,
        inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
        outputs: vec![(&alice, 400)],
        return_to_sender: Some(100),
    });
    assert!(handler.is_tx_valid(&tx1));
}

#[test]
fn invalid_1_out_of_3() {
    initialize();

    let bob = Wallet::random(3, 3);
    let alice = Wallet::random(1, 1);
    let (handler, genesis_tx) = setup_handler(&bob, 500, 1);

    let tx1 = new_tx_first_n_signers_only(
        NewTxParams {
            signer: &bob,
            inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
            outputs: vec![(&alice, 400)],
            return_to_sender: Some(100),
        },
        1,
    );
    assert!(!handler.is_tx_valid(&tx1));
}

#[test]
fn invalid_2_out_of_3() {
    initialize();

    let bob = Wallet::random(3, 3);
    let alice = Wallet::random(1, 1);
    let (handler, genesis_tx) = setup_handler(&bob, 500, 1);

    let tx1 = new_tx_first_n_signers_only(
        NewTxParams {
            signer: &bob,
            inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
            outputs: vec![(&alice, 400)],
            return_to_sender: Some(100),
        },
        2,
    );
    assert!(!handler.is_tx_valid(&tx1));
}

#[test]
fn valid_1_out_of_3() {
    initialize();

    let bob = Wallet::random(3, 1);
    let alice = Wallet::random(1, 1);
    let (handler, genesis_tx) = setup_handler(&bob, 500, 1);

    let tx1 = new_tx_first_n_signers_only(
        NewTxParams {
            signer: &bob,
            inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
            outputs: vec![(&alice, 400)],
            return_to_sender: Some(100),
        },
        1,
    );
    assert!(handler.is_tx_valid(&tx1));
}

#[test]
fn valid_2_out_of_3() {
    initialize();

    let bob = Wallet::random(3, 1);
    let alice = Wallet::random(1, 1);
    let (handler, genesis_tx) = setup_handler(&bob, 500, 1);

    let tx1 = new_tx_first_n_signers_only(
        NewTxParams {
            signer: &bob,
            inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
            outputs: vec![(&alice, 400)],
            return_to_sender: Some(100),
        },
        2,
    );
    assert!(handler.is_tx_valid(&tx1));
}

#[test]
fn valid_3_out_of_3() {
    initialize();

    let bob = Wallet::random(3, 3);
    let alice = Wallet::random(1, 1);
    let (handler, genesis_tx) = setup_handler(&bob, 500, 1);

    let tx1 = new_tx(NewTxParams {
        signer: &bob,
        inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
        outputs: vec![(&alice, 400)],
        return_to_sender: Some(100),
    });
    assert!(handler.is_tx_valid(&tx1));
}
