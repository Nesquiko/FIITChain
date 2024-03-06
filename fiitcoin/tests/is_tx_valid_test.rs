use common::{new_tx, setup_pool, NewTxParams, Participant};
use fiitcoin::{
    handler::{Handler, TxHandler},
    tx::raw_tx,
};
use rsa::signature::{SignatureEncoding, Signer};

use crate::common::{new_tx_forged_signatures, OUTPUT_VALUE};

mod common;

#[test]
// Phase 1 test 1
fn all_valid_transactions() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();
    let derek = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 10);
    let handler = Handler::new(utxo_pool);

    // 2 * 100 from root_tx => 200 to alice
    let to_alice_tx = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0), (&root_tx, 9)],
        outputs: &[(&alice, 2 * OUTPUT_VALUE)],
        return_to_sender: None,
    });
    assert!(handler.is_tx_valid(&to_alice_tx));

    // 3 * 100 from root_tx => 4 * 50 to charlie, 100 back to bob
    let to_charlie_tx = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 1), (&root_tx, 2), (&root_tx, 3)],
        outputs: &[
            (&charlie, 50),
            (&charlie, 50),
            (&charlie, 50),
            (&charlie, 50),
        ],
        return_to_sender: Some(100),
    });
    assert!(handler.is_tx_valid(&to_charlie_tx));

    // 100 from root_tx => 2 * 25 to derek, fee 50
    let to_charlie_tx = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 1)],
        outputs: &[(&derek, 25), (&derek, 25)],
        return_to_sender: None,
    });
    assert!(handler.is_tx_valid(&to_charlie_tx));
}

#[test]
// Phase 1 test 2
fn invalid_signature() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 2);
    let handler = Handler::new(utxo_pool);

    let mut tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, OUTPUT_VALUE)],
        return_to_sender: None,
    });
    assert!(handler.is_tx_valid(&tx1));

    let mut random_signature = bob.sk.sign(b"random data").to_bytes();
    tx1.force_signature_on_input(0, random_signature);
    assert!(!handler.is_tx_valid(&tx1));

    let mut tx2 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 1)],
        outputs: &[(&alice, OUTPUT_VALUE)],
        return_to_sender: None,
    });
    assert!(handler.is_tx_valid(&tx2));

    let raw_tx1 = raw_tx(tx1.inputs(), tx1.outputs()).unwrap();
    random_signature = bob.sk.sign(&raw_tx1).to_bytes();
    tx2.force_signature_on_input(0, random_signature);
    assert!(!handler.is_tx_valid(&tx1));
}

#[test]
// Phase 1 test 3
fn different_private_key() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 2);
    let handler = Handler::new(utxo_pool);

    let tx = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, OUTPUT_VALUE)],
        return_to_sender: None,
    });
    assert!(handler.is_tx_valid(&tx));

    let same_tx = new_tx_forged_signatures(
        NewTxParams {
            sender: &bob,
            inputs: &[(&root_tx, 0)],
            outputs: &[(&alice, OUTPUT_VALUE)],
            return_to_sender: None,
        },
        &charlie,
    );
    assert!(!handler.is_tx_valid(&same_tx));
}

#[test]
// Phase 1 test 4
fn outputs_greater_than_inputs() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 1);
    let handler = Handler::new(utxo_pool);

    let tx = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, OUTPUT_VALUE + 1)],
        return_to_sender: None,
    });
    assert!(!handler.is_tx_valid(&tx));
}

#[test]
// Phase 1 test 5
fn output_not_in_pool() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 1);
    let mut handler = Handler::new(utxo_pool);

    let valid_tx = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, OUTPUT_VALUE)],
        return_to_sender: None,
    });
    assert!(handler.is_tx_valid(&valid_tx));
    assert_eq!(1, handler.handle(vec![&valid_tx]).len());

    let invalid_tx = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, OUTPUT_VALUE)],
        return_to_sender: None,
    });
    assert!(!handler.is_tx_valid(&invalid_tx));
}

#[test]
// Phase 1 test 6
fn one_output_multiple_times() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 1);
    let handler = Handler::new(utxo_pool);

    let tx = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0), (&root_tx, 0)],
        outputs: &[(&alice, OUTPUT_VALUE)],
        return_to_sender: None,
    });
    assert!(!handler.is_tx_valid(&tx));
}

// Phase 1 test 7 - this test is meaningless, because outputs have values of
// type u32. Even if I serialized a negative value, it would only be treated
// as a really big one, in which case inputs < outputs case would catch it as
// an invalid tx. Thus, I didn't write this test.
