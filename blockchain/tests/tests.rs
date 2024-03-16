use blockchain::{
    block::{IncompleteBlock, COINBASE},
    blockchain::CUT_OFF_AGE,
};
use common::{new_tx, setup_handler, NewTxParams, Participant};
use rsa::signature::{SignatureEncoding, Signer};

mod common;

// Phase 3 test 1 and test 15
#[test]
fn empty_block() {
    common::initialize();

    let bob = Participant::new();
    let (mut handler, _tx) = setup_handler(&bob);

    let block = handler.create_block(&bob.vk);
    assert!(handler.process_block(block));
}

// Phase 3 test 2 and test 16
#[test]
fn block_with_one_tx() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let (mut handler, genesis_tx) = setup_handler(&bob);

    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&alice, COINBASE - 500)],
        return_to_sender: Some(500),
    });
    handler.process_tx(tx1);
    let block = handler.create_block(&bob.vk);
    assert!(handler.process_block(block));
}

// Phase 3 test 3
#[test]
fn block_with_many_tx() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();
    let (mut handler, genesis_tx) = setup_handler(&bob);

    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&alice, 100), (&alice, 100), (&charlie, 200)],
        return_to_sender: None,
    });
    let tx2 = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&tx1, 0), (&tx1, 1)],
        outputs: &[(&charlie, 150)],
        return_to_sender: Some(50),
    });
    let tx3 = new_tx(NewTxParams {
        sender: &charlie,
        inputs: &[(&tx2, 0), (&tx1, 2)],
        outputs: &[(&bob, 340)],
        return_to_sender: Some(10),
    });

    handler.process_tx(tx1);
    handler.process_tx(tx2);
    handler.process_tx(tx3);

    let block = handler.create_block(&charlie.vk);
    assert!(handler.process_block(block));
}

// Phase 3 test 4
#[test]
fn block_with_many_doublespends() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();
    let (mut handler, genesis_tx) = setup_handler(&bob);

    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&alice, 100), (&alice, 100), (&charlie, 200)],
        return_to_sender: None,
    });
    let tx2 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&charlie, 150)],
        return_to_sender: Some(50),
    });

    let tx3_from_tx1 = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&tx1, 0)],
        outputs: &[(&bob, 100)],
        return_to_sender: None,
    });
    let tx4_from_tx1 = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&tx1, 0)],
        outputs: &[(&charlie, 100)],
        return_to_sender: None,
    });

    let tx5_from_tx2 = new_tx(NewTxParams {
        sender: &charlie,
        inputs: &[(&tx2, 0)],
        outputs: &[(&alice, 150)],
        return_to_sender: None,
    });
    let tx6_from_tx2 = new_tx(NewTxParams {
        sender: &charlie,
        inputs: &[(&tx2, 0)],
        outputs: &[(&bob, 150)],
        return_to_sender: None,
    });

    handler.process_tx(tx1);
    handler.process_tx(tx2);
    handler.process_tx(tx3_from_tx1);
    handler.process_tx(tx4_from_tx1);
    handler.process_tx(tx5_from_tx2);
    handler.process_tx(tx6_from_tx2);

    let block = handler.create_block(&charlie.vk);
    assert_eq!(block.txs().len(), 2);
    assert!(handler.process_block(block));
}

// Phase 3 test 5
#[test]
fn reject_new_genesis_block() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let (mut handler, _genesis_tx) = setup_handler(&bob);
    let block = handler.create_block(&bob.vk);
    assert!(handler.process_block(block));

    let new_genesis = IncompleteBlock::new([0; 32], &alice.vk).finalize();
    assert!(!handler.process_block(new_genesis));
}

// Phase 3 test 6
#[test]
fn block_refences_invalid_prev() {
    common::initialize();

    let bob = Participant::new();
    let (mut handler, _tx) = setup_handler(&bob);

    let mut block = handler.create_block(&bob.vk);
    block.set_prev([1; 32]);
    assert!(!handler.process_block(block));
}

// Phase 3 test 7 and test 21
#[test]
fn reject_block_with_invalid_txs() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let (mut handler, genesis_tx) = setup_handler(&bob);

    let tx1_invalid_spender = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&alice, 100)],
        return_to_sender: None,
    });
    let tx2_outputs_more_than_inputs = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&alice, COINBASE)],
        return_to_sender: Some(50),
    });
    let mut tx3_invalid_sig = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&bob, 100)],
        return_to_sender: None,
    });
    let random_signature = bob.sk.sign(b"random data").to_bytes();
    tx3_invalid_sig.force_signature_on_input(0, random_signature);

    handler.process_tx(tx1_invalid_spender);
    handler.process_tx(tx2_outputs_more_than_inputs);
    handler.process_tx(tx3_invalid_sig);

    let block = handler.create_block(&bob.vk);
    assert_eq!(0, block.txs().len());
    assert!(handler.process_block(block)); // block is valid, only txs not
}

// Phase 3 test 8, test 17, test 22, test 23
#[test]
fn multiple_blocks() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();
    let derek = Participant::new();
    let (mut handler, genesis_tx) = setup_handler(&bob);

    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[
            (&alice, 100),
            (&alice, 100),
            (&charlie, 100),
            (&charlie, 100),
            (&derek, 100),
            (&derek, 100),
        ],
        return_to_sender: Some(25),
    });
    handler.process_tx(tx1.clone());
    let block = handler.create_block(&bob.vk);
    assert!(handler.process_block(block));

    let tx2 = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&tx1, 0), (&tx1, 1)],
        outputs: &[(&derek, 75), (&derek, 75)],
        return_to_sender: Some(50),
    });
    let tx3 = new_tx(NewTxParams {
        sender: &derek,
        inputs: &[(&tx1, 4), (&tx1, 5)],
        outputs: &[(&charlie, 75), (&bob, 75)],
        return_to_sender: Some(50),
    });
    handler.process_tx(tx2.clone());
    handler.process_tx(tx3.clone());
    let block = handler.create_block(&alice.vk);
    assert!(handler.process_block(block));

    let tx4 = new_tx(NewTxParams {
        sender: &derek,
        inputs: &[(&tx2, 1)],
        outputs: &[(&bob, 75)],
        return_to_sender: None,
    });
    let tx5 = new_tx(NewTxParams {
        sender: &derek,
        inputs: &[(&tx3, 0)],
        outputs: &[(&bob, 25)],
        return_to_sender: Some(50),
    });
    handler.process_tx(tx4);
    handler.process_tx(tx5);
    let block = handler.create_block(&derek.vk);
    assert!(handler.process_block(block));
}

// Phase 3 test 9 and test 19
#[test]
fn utxo_spent_by_parent() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let (mut handler, genesis_tx) = setup_handler(&bob);

    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&alice, 200), (&alice, 200)],
        return_to_sender: None,
    });
    handler.process_tx(tx1);
    let block = handler.create_block(&bob.vk);
    assert_eq!(1, block.txs().len());
    assert!(handler.process_block(block));

    let tx2 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&alice, 200), (&alice, 200)],
        return_to_sender: None,
    });
    handler.process_tx(tx2);
    let block = handler.create_block(&bob.vk);
    assert_eq!(0, block.txs().len());
    assert!(handler.process_block(block));
}

// Phase 3 test 10 and test 18
#[test]
fn utxo_from_fork() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let (mut handler, genesis_tx) = setup_handler(&bob);
    let genesis_block_hash = handler.hash_at_max_height();

    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&alice, 200), (&alice, 200)],
        return_to_sender: None,
    });
    handler.process_tx(tx1.clone());
    let block = handler.create_block(&bob.vk);
    assert_eq!(1, block.txs().len());
    assert!(handler.process_block(block));

    let tx1_fork = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&alice, 400)],
        return_to_sender: None,
    });
    handler.process_tx(tx1_fork.clone());
    let block = handler.create_fork(genesis_block_hash, &alice.vk).unwrap();
    assert_eq!(1, block.txs().len());
    assert!(handler.process_block(block));

    let tx2_depends_on_tx1 = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&tx1, 1)],
        outputs: &[(&bob, 200)],
        return_to_sender: None,
    });
    handler.process_tx(tx2_depends_on_tx1);
    let block = handler.create_block(&alice.vk);
    assert_eq!(0, block.txs().len());
    assert!(handler.process_block(block));
}

// Phase 3 test 11 and test 20
#[test]
fn spent_old_utxo() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();
    let (mut handler, genesis_tx) = setup_handler(&bob);

    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[
            (&alice, 150),
            (&alice, 150),
            (&charlie, 150),
            (&charlie, 150),
        ],
        return_to_sender: Some(25),
    });
    handler.process_tx(tx1.clone());
    let block = handler.create_block(&bob.vk);
    assert_eq!(1, block.txs().len());
    assert!(handler.process_block(block));

    for _ in 0..6 {
        let block = handler.create_block(&alice.vk);
        assert!(handler.process_block(block));
    }

    let tx_with_old_utxo = new_tx(NewTxParams {
        sender: &charlie,
        inputs: &[(&tx1, 2)],
        outputs: &[(&bob, 150)],
        return_to_sender: None,
    });
    handler.process_tx(tx_with_old_utxo);
    let block = handler.create_block(&bob.vk);
    assert_eq!(1, block.txs().len());
    assert!(handler.process_block(block));
}

// Phase 3 test 12
#[test]
fn linear_blocks() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();
    let (mut handler, _genesis_tx) = setup_handler(&bob);

    for i in 0..24 {
        let miner = match i % 3 {
            0 => &bob.vk,
            1 => &alice.vk,
            _ => &charlie.vk,
        };
        let block = handler.create_block(miner);
        assert!(handler.process_block(block));
    }
}

// Phase 3 test 13
#[test]
fn accept_block_before_cut_off_age() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();
    let (mut handler, _genesis_tx) = setup_handler(&bob);
    let genesis_block_hash = handler.hash_at_max_height();

    for i in 0..(CUT_OFF_AGE - 1) {
        let miner = match i % 3 {
            0 => &bob.vk,
            1 => &alice.vk,
            _ => &charlie.vk,
        };
        let block = handler.create_block(miner);
        assert!(handler.process_block(block));
    }

    let new_b = handler.create_fork(genesis_block_hash, &alice.vk);
    assert!(new_b.is_some());
    assert!(handler.process_block(new_b.unwrap()));
}

// Phase 3 test 14
#[test]
fn reject_block_after_cut_off_age() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();
    let (mut handler, _genesis_tx) = setup_handler(&bob);
    let genesis_block_hash = handler.hash_at_max_height();

    for i in 0..CUT_OFF_AGE {
        let miner = match i % 3 {
            0 => &bob.vk,
            1 => &alice.vk,
            _ => &charlie.vk,
        };
        let block = handler.create_block(miner);
        assert!(handler.process_block(block));
    }

    let new_b = handler.create_fork(genesis_block_hash, &alice.vk);
    assert!(new_b.is_none());
}

// Phase 3 test 24
#[test]
fn utxo_from_sibling() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let (mut handler, genesis_tx) = setup_handler(&bob);
    let genesis_block_hash = handler.hash_at_max_height();

    let tx1 = new_tx(NewTxParams {
        sender: &bob,
        inputs: &[(&genesis_tx, 0)],
        outputs: &[(&alice, 200), (&alice, 200)],
        return_to_sender: None,
    });
    handler.process_tx(tx1.clone());
    let block = handler.create_block(&bob.vk);
    assert_eq!(1, block.txs().len());
    assert!(handler.process_block(block));

    let tx_depends_on_sibling = new_tx(NewTxParams {
        sender: &alice,
        inputs: &[(&tx1, 0)],
        outputs: &[(&bob, 200)],
        return_to_sender: None,
    });
    handler.process_tx(tx_depends_on_sibling);
    let block = handler.create_fork(genesis_block_hash, &alice.vk).unwrap();
    assert_eq!(0, block.txs().len());
    assert!(handler.process_block(block));
}

// Phase 3 test 25
#[test]
fn oldest_fork_is_max_height() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let (mut handler, _genesis_tx) = setup_handler(&bob);
    let genesis_block_hash = handler.hash_at_max_height();

    let mut last_hash = [0; 32];
    for i in 0..8 {
        let miner = match i % 2 {
            0 => &bob.vk,
            _ => &alice.vk,
        };
        let block = handler.create_fork(genesis_block_hash, miner).unwrap();
        last_hash = block.hash();
        assert!(handler.process_block(block));
    }

    assert_eq!(handler.hash_at_max_height(), last_hash);
}

// Phase 3 test 26
#[test]
fn new_blocks_on_oldest_fork() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();
    let (mut handler, _genesis_tx) = setup_handler(&bob);
    let genesis_block_hash = handler.hash_at_max_height();

    let mut last_hash = [0; 32];
    for _ in 0..3 {
        let block = handler
            .create_fork(genesis_block_hash, &charlie.vk)
            .unwrap();
        let mut previous_block = block.hash();
        assert!(handler.process_block(block));

        for j in 1..3 {
            let miner = if j % 2 == 0 { &bob.vk } else { &alice.vk };

            let block = handler.create_block(miner);
            assert_eq!(block.prev(), previous_block);
            previous_block = block.hash();
            last_hash = block.hash();
            assert!(handler.process_block(block));
        }
    }

    assert_eq!(handler.hash_at_max_height(), last_hash);
    let new_block = handler.create_block(&charlie.vk);
    assert_eq!(last_hash, new_block.prev());
    assert!(handler.process_block(new_block));
}

// Phase 3 test 27
#[test]
fn reject_block_with_cut_off_parent() {
    common::initialize();

    let bob = Participant::new();
    let alice = Participant::new();
    let charlie = Participant::new();
    let (mut handler, _genesis_tx) = setup_handler(&bob);
    let genesis_block_hash = handler.hash_at_max_height();

    // 1st fork
    let block = handler
        .create_fork(genesis_block_hash, &charlie.vk)
        .unwrap();
    let mut first_last_block = block.hash();
    assert!(handler.process_block(block));
    let block = handler.create_block(&bob.vk);
    assert_eq!(block.prev(), first_last_block);
    first_last_block = block.hash();
    assert!(handler.process_block(block));

    // 2nd longer fork
    let block = handler
        .create_fork(genesis_block_hash, &charlie.vk)
        .unwrap();
    let mut previous_block = block.hash();
    assert!(handler.process_block(block));
    for j in 0..=CUT_OFF_AGE {
        let miner = if j % 2 == 0 { &bob.vk } else { &alice.vk };

        let block = handler.create_block(miner);
        assert_eq!(block.prev(), previous_block);
        previous_block = block.hash();
        assert!(handler.process_block(block));
    }

    let new_block = handler.create_fork(first_last_block, &charlie.vk);
    assert!(new_block.is_none());
}
