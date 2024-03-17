mod common;
use common::{initialize, new_tx, Wallet};
use multisig::utxo::UTXO;

use crate::common::{new_tx_first_n_signers_only, setup_block_handler, NewTxParams};

#[test]
fn block_with_normal_tx() {
    initialize();

    let bob = Wallet::random(1, 1);
    let alice = Wallet::random(1, 1);
    let (mut handler, genesis_tx) = setup_block_handler(&bob);

    let tx1 = new_tx(NewTxParams {
        signer: &bob,
        inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
        outputs: vec![(&alice, 400)],
        return_to_sender: Some(100),
    });
    handler.process_tx(tx1);
    let block = handler.create_block(bob.verifiers(), bob.threshold());
    assert!(handler.process_block(block));
}

#[test]
fn block_with_many_tx() {
    common::initialize();

    let bob = Wallet::random(3, 2);
    let alice = Wallet::random(3, 2);
    let charlie = Wallet::random(3, 2);
    let (mut handler, genesis_tx) = setup_block_handler(&bob);

    let tx1 = new_tx_first_n_signers_only(
        NewTxParams {
            signer: &bob,
            inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
            outputs: vec![(&alice, 100), (&alice, 100), (&charlie, 200)],
            return_to_sender: Some(100),
        },
        2,
    );
    let tx2 = new_tx_first_n_signers_only(
        NewTxParams {
            signer: &alice,
            inputs: vec![(UTXO::new(tx1.hash(), 0)), (UTXO::new(tx1.hash(), 1))],
            outputs: vec![(&charlie, 150)],
            return_to_sender: Some(50),
        },
        2,
    );
    let tx3_not_enough_signers = new_tx_first_n_signers_only(
        NewTxParams {
            signer: &charlie,
            inputs: vec![(UTXO::new(tx2.hash(), 0)), (UTXO::new(tx1.hash(), 2))],
            outputs: vec![(&bob, 340)],
            return_to_sender: Some(10),
        },
        1,
    );

    handler.process_tx(tx1);
    handler.process_tx(tx2);
    handler.process_tx(tx3_not_enough_signers);

    let block = handler.create_block(charlie.verifiers(), charlie.threshold());
    assert_eq!(2, block.txs().len());
    assert!(handler.process_block(block));
}

#[test]
fn block_with_invalid_1_out_of_3() {
    initialize();

    let bob = Wallet::random(3, 3);
    let alice = Wallet::random(1, 1);
    let (mut handler, genesis_tx) = setup_block_handler(&bob);

    let tx1 = new_tx_first_n_signers_only(
        NewTxParams {
            signer: &bob,
            inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
            outputs: vec![(&alice, 400)],
            return_to_sender: Some(100),
        },
        1,
    );

    handler.process_tx(tx1);
    let block = handler.create_block(bob.verifiers(), bob.threshold());
    assert_eq!(0, block.txs().len());
    assert!(handler.process_block(block));
}

#[test]
fn block_with_invalid_2_out_of_3() {
    initialize();

    let bob = Wallet::random(3, 3);
    let alice = Wallet::random(1, 1);
    let (mut handler, genesis_tx) = setup_block_handler(&bob);

    let tx1 = new_tx_first_n_signers_only(
        NewTxParams {
            signer: &bob,
            inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
            outputs: vec![(&alice, 400)],
            return_to_sender: Some(100),
        },
        2,
    );
    handler.process_tx(tx1);
    let block = handler.create_block(bob.verifiers(), bob.threshold());
    assert_eq!(0, block.txs().len());
    assert!(handler.process_block(block));
}

#[test]
fn block_with_valid_1_out_of_3() {
    initialize();

    let bob = Wallet::random(3, 1);
    let alice = Wallet::random(1, 1);
    let (mut handler, genesis_tx) = setup_block_handler(&bob);

    let tx1 = new_tx_first_n_signers_only(
        NewTxParams {
            signer: &bob,
            inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
            outputs: vec![(&alice, 400)],
            return_to_sender: Some(100),
        },
        1,
    );
    handler.process_tx(tx1);
    let block = handler.create_block(bob.verifiers(), bob.threshold());
    assert_eq!(1, block.txs().len());
    assert!(handler.process_block(block));
}

#[test]
fn block_with_valid_2_out_of_3() {
    initialize();

    let bob = Wallet::random(3, 1);
    let alice = Wallet::random(1, 1);
    let (mut handler, genesis_tx) = setup_block_handler(&bob);

    let tx1 = new_tx_first_n_signers_only(
        NewTxParams {
            signer: &bob,
            inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
            outputs: vec![(&alice, 400)],
            return_to_sender: Some(100),
        },
        2,
    );
    handler.process_tx(tx1);
    let block = handler.create_block(bob.verifiers(), bob.threshold());
    assert_eq!(1, block.txs().len());
    assert!(handler.process_block(block));
}

#[test]
fn block_with_valid_3_out_of_3() {
    initialize();

    let bob = Wallet::random(3, 3);
    let alice = Wallet::random(1, 1);
    let (mut handler, genesis_tx) = setup_block_handler(&bob);

    let tx1 = new_tx(NewTxParams {
        signer: &bob,
        inputs: vec![(UTXO::new(genesis_tx.hash(), 0))],
        outputs: vec![(&alice, 400)],
        return_to_sender: Some(100),
    });
    handler.process_tx(tx1);
    let block = handler.create_block(bob.verifiers(), bob.threshold());
    assert_eq!(1, block.txs().len());
    assert!(handler.process_block(block));
}
