use common::{new_tx, setup_pool, NewTxParams, Participant, OUTPUT_VALUE};
use fiitcoin::handler::{balance_of, MaxFeeHandler, TxHandler};

mod common;

#[test]
// Phase 1 Maxfee test 1
fn valid_txs() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 3);
    let mut handler = MaxFeeHandler::new(utxo_pool);

    // bob: 100 + 20, alice: 80, charlie: 80, fee: 20
    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0), (&root_tx, 1)],
        outputs: &[(&alice, 40), (&alice, 40), (&charlie, 80)],
        return_to_sender: Some(20),
    });

    let mut txs = handler.handle(vec![&tx1]);
    assert_eq!(1, txs.len());
    assert_eq!(120, balance_of(handler.pool(), bob.vk.as_ref()));
    assert_eq!(80, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(80, balance_of(handler.pool(), charlie.vk.as_ref()));

    let tx_alice_combine_outputs = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&tx1, 0), (&tx1, 1)],
        outputs: &[(&alice, 80)],
        return_to_sender: None,
    });
    txs = handler.handle(vec![&tx_alice_combine_outputs]);
    assert_eq!(1, txs.len());
    assert_eq!(120, balance_of(handler.pool(), bob.vk.as_ref()));
    assert_eq!(80, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(80, balance_of(handler.pool(), charlie.vk.as_ref()));
}

#[test]
// Phase 1 Maxfee test 2
fn transactions_with_same_output() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 1);
    let mut handler = MaxFeeHandler::new(utxo_pool);

    let tx_fee_50 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, 10)],
        return_to_sender: Some(40),
    });

    let tx_fee_10 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&charlie, 10)],
        return_to_sender: Some(80),
    });

    let txs = handler.handle(vec![&tx_fee_10, &tx_fee_50]);
    assert_eq!(1, txs.len());
    assert_eq!(tx_fee_50.hash(), txs.get(0).unwrap().hash());
    assert_eq!(40, balance_of(handler.pool(), bob.vk.as_ref()));
    assert_eq!(10, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(0, balance_of(handler.pool(), charlie.vk.as_ref()));
}

#[test]
// Phase 1 Maxfee test 3
fn mix_of_txs() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();

    let (utxo_pool, root_tx) = setup_pool(&bob, OUTPUT_VALUE, 3);
    let mut handler = MaxFeeHandler::new(utxo_pool);

    let tx_fee_50 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 0)],
        outputs: &[(&alice, 50)],
        return_to_sender: None,
    });
    let tx_fee_related_10 = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&tx_fee_50, 0)],
        outputs: &[(&bob, 20)],
        return_to_sender: Some(20),
    });

    let invalid_output_tx = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&root_tx, 69)],
        outputs: &[(&alice, 50)],
        return_to_sender: None,
    });

    let mut txs = handler.handle(vec![&tx_fee_related_10, &invalid_output_tx, &tx_fee_50]);
    assert_eq!(2, txs.len());
    assert_eq!(20, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(
        2 * OUTPUT_VALUE as u64 + 20,
        balance_of(handler.pool(), bob.vk.as_ref())
    );
    assert_eq!(0, balance_of(handler.pool(), charlie.vk.as_ref()));

    let tx_fee_5 = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&tx_fee_related_10, 1)],
        outputs: &[(&charlie, 10), (&charlie, 4), (&charlie, 1)],
        return_to_sender: None,
    });
    let invalid_output_greater_than_input = new_tx(NewTxParams {
        sender: &charlie,
        inputs: &[(&tx_fee_5, 0), (&tx_fee_5, 1), (&tx_fee_5, 2)],
        outputs: &[(&bob, 20)],
        return_to_sender: None,
    });

    txs = handler.handle(vec![&invalid_output_greater_than_input, &tx_fee_5]);
    assert_eq!(1, txs.len());
    assert_eq!(0, balance_of(handler.pool(), alice.vk.as_ref()));
    assert_eq!(
        2 * OUTPUT_VALUE as u64 + 20,
        balance_of(handler.pool(), bob.vk.as_ref())
    );
    assert_eq!(15, balance_of(handler.pool(), charlie.vk.as_ref()));
}
