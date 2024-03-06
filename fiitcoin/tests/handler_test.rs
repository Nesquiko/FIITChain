use crate::common::{new_tx, setup_pool, NewTxParams, Participant, OUTPUT_VALUE};
use fiitcoin::handler::{balance_of, Handler, TxHandler};
use fiitcoin::tx::UnsignedTx;
use fiitcoin::utxo::{UTXOPool, UTXO};
use rsa::signature::{SignatureEncoding, Signer};
use sha2::{Digest, Sha256};

mod common;

#[test]
// Phase 1 test 8
fn transaction_is_valid_and_possible() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();

    let mut hasher = Sha256::new();
    hasher.update("genesis-hash");
    let genesis_hash: [u8; 32] = hasher.finalize().into();

    let mut bob_tx = UnsignedTx::new();
    bob_tx.add_output(10, &bob.vk);
    bob_tx.add_input(genesis_hash, 0);
    let root_bob_tx = bob_tx.sing_inputs_and_finalize(&bob.sk).unwrap();

    let mut utxo_pool = UTXOPool::new();
    let root_utxo = UTXO::new(root_bob_tx.hash(), 0);
    let output = root_bob_tx.output(0).unwrap();
    utxo_pool.add_utxo(root_utxo, &output);

    let mut tx_to_alice = UnsignedTx::new();
    tx_to_alice.add_input(root_bob_tx.hash(), 0);
    // split the 10 into outputs of 4, 3, 2 and 1 as fee
    tx_to_alice.add_output(4, &alice.vk);
    tx_to_alice.add_output(3, &alice.vk);
    tx_to_alice.add_output(2, &alice.vk);
    let alice_tx = tx_to_alice.sing_inputs_and_finalize(&bob.sk).unwrap();

    let mut handler = Handler::new(utxo_pool);
    let alice_tx_valid = handler.is_tx_valid(&alice_tx);
    assert!(alice_tx_valid);

    let possible_txs = handler.handle(vec![&alice_tx]);
    println!("possible_txs: {:?}", possible_txs);
    assert_eq!(1, possible_txs.len());

    assert_eq!(9, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(0, balance_of(handler.pool(), bob.vk.as_ref()));
}

#[test]
// Phase 1 test 9
fn invalid_signature() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 1);
    let mut handler = Handler::new(utxo_pool);

    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, 10)],
        return_to_sender: Some(13),
    });
    let mut invalid_tx = tx1.clone();
    let random_signature = bob.sk.sign(b"random data").to_bytes();
    invalid_tx.force_signature_on_input(0, random_signature);

    let txs = handler.handle(vec![&tx1, &invalid_tx]);
    assert_eq!(1, txs.len());

    assert_eq!(10, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(13, balance_of(handler.pool(), bob.vk.as_ref()));
}

#[test]
// Phase 1 test 10
fn outputs_greater_than_inputs() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 1);
    let mut handler = Handler::new(utxo_pool);

    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, OUTPUT_VALUE + 1)],
        return_to_sender: None,
    });

    let txs = handler.handle(vec![&tx1]);
    assert_eq!(0, txs.len());

    assert_eq!(0, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(
        OUTPUT_VALUE as u64,
        balance_of(handler.pool(), bob.vk.as_ref())
    );
}

#[test]
// Phase 1 test 11
fn output_double_spend() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 1);
    let mut handler = Handler::new(utxo_pool);

    let to_alice_tx = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, 10)],
        return_to_sender: Some(13),
    });
    let to_alice_doublespend = to_alice_tx.clone();

    let txs = handler.handle(vec![&to_alice_tx, &to_alice_doublespend]);
    assert_eq!(1, txs.len());

    assert_eq!(10, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(13, balance_of(handler.pool(), bob.vk.as_ref()));
}

#[test]
// Phase 1 test 12
fn related_transactions_chronological_order() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 1);
    let mut handler = Handler::new(utxo_pool);

    let to_alice_tx = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, 10)],
        return_to_sender: None,
    });
    assert!(handler.is_tx_valid(&to_alice_tx));

    let to_john_from_alice = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&to_alice_tx, 0)],
        outputs: &[(&charlie, 10)],
        return_to_sender: None,
    });
    assert!(!handler.is_tx_valid(&to_john_from_alice));

    let txs = handler.handle(vec![&to_alice_tx, &to_john_from_alice]);
    assert_eq!(2, txs.len());

    assert_eq!(0, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(0, balance_of(handler.pool(), bob.vk.as_ref()));
    assert_eq!(10, balance_of(handler.pool(), charlie.vk.as_ref()));
}

#[test]
// Phase 1 test 13
fn non_existent_utxo_as_input() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 1);
    let mut handler = Handler::new(utxo_pool);

    let with_invalid_utxo = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 123)],
        outputs: &[(&alice, 10)],
        return_to_sender: None,
    });

    let txs = handler.handle(vec![&with_invalid_utxo]);
    assert_eq!(0, txs.len());

    assert_eq!(0, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(
        OUTPUT_VALUE as u64,
        balance_of(handler.pool(), bob.vk.as_ref())
    );
}

#[test]
// Phase 1 test 14
fn related_transactions_reverse_order() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();
    let derek = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 1);
    let mut handler = Handler::new(utxo_pool);

    // alice: 3 x 10
    // bob: 50
    // fee: 100 - 50 - 3 x 10 = 20
    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, 10), (&alice, 10), (&alice, 10)],
        return_to_sender: Some(50),
    });

    // input: tx1 idx 0 and 2
    // alice: 10 from tx1 idx 1 + 5
    // charlie: 15
    // fee: 0
    let tx2 = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&tx1, 0), (&tx1, 2)],
        outputs: &[(&charlie, 15)],
        return_to_sender: Some(5),
    });

    // input: tx1 idx 3
    // bob: 5
    // charlie 15 from tx2 idx 0 + 40
    // fee: 5
    let tx3 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&tx1, 3)],
        outputs: &[(&charlie, 40)],
        return_to_sender: Some(5),
    });

    // derek tries to send himself alice's output from tx1 at idx 1
    let tx_invalid = new_tx(NewTxParams {
        sender: &derek,
        inputs: &[(&tx1, 1)],
        outputs: &[],
        return_to_sender: Some(10),
    });

    let txs = handler.handle(vec![&tx2, &tx_invalid, &tx3, &tx1]);
    assert_eq!(3, txs.len());

    assert_eq!(15, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(5, balance_of(handler.pool(), bob.vk.as_ref()));
    assert_eq!(55, balance_of(handler.pool(), charlie.vk.as_ref()));
    assert_eq!(0, balance_of(handler.pool(), derek.vk.as_ref()));
}

#[test]
// Phase 1 test 15
fn multiple_rounds() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 2);
    let mut handler = Handler::new(utxo_pool);

    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, 25), (&alice, 50), (&alice, 25)],
        return_to_sender: None,
    });
    let mut txs = handler.handle(vec![&tx1]);
    assert_eq!(1, txs.len());
    assert_eq!(
        OUTPUT_VALUE as u64,
        balance_of(handler.pool(), alice.vk.as_ref())
    );
    assert_eq!(
        OUTPUT_VALUE as u64,
        balance_of(handler.pool(), bob.vk.as_ref())
    );

    let tx2 = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&tx1, 1)],
        outputs: &[(&bob, 30)],
        return_to_sender: Some(20),
    });
    txs = handler.handle(vec![&tx2]);
    assert_eq!(1, txs.len());
    assert_eq!(25 + 25 + 20, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(
        OUTPUT_VALUE as u64 + 30,
        balance_of(handler.pool(), bob.vk.as_ref())
    );
}
