use std::sync::Once;

use fiitcoin::tx::{self, Finalized, Tx};
use rand::Rng;
use rsa::{
    pkcs1v15::{SigningKey, VerifyingKey},
    signature::Keypair,
    RsaPrivateKey,
};
use sha2::Sha256;

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

pub struct Participant {
    pub sk: SigningKey<Sha256>,
    pub vk: VerifyingKey<Sha256>,
    pub balance: u64,
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

        Self { sk, vk, balance: 0 }
    }
}

pub fn random_new_tx(
    participants: &Vec<Participant>,
    prev_tx_hash: [u8; 32],
    output_idx: u8,
) -> Tx<Finalized> {
    //sender
    let mut sender_idx = rand::thread_rng().gen_range(0..participants.len());
    let mut sender = participants.get(sender_idx).unwrap();
    while sender.balance < 1 {
        sender_idx = rand::thread_rng().gen_range(0..participants.len());
        sender = participants.get(sender_idx).unwrap();
    }
    let mut receiver_idx = rand::thread_rng().gen_range(0..participants.len());
    let mut receiver = participants.get(receiver_idx).unwrap();
    while sender_idx == receiver_idx {
        receiver_idx = rand::thread_rng().gen_range(0..participants.len());
        receiver = participants.get(receiver_idx).unwrap();
    }

    new_tx(sender, receiver, prev_tx_hash, output_idx)
}

pub fn new_tx(
    sender: &Participant,
    receiver: &Participant,
    prev_tx_hash: [u8; 32],
    output_idx: u8,
) -> Tx<Finalized> {
    let mut tx = tx::Tx::new();
    let max_available_balance = sender.balance - 1; // 1 as fee
    let number_of_outputs = rand::thread_rng().gen_range(0..max_available_balance);
    let value_per_output = max_available_balance / number_of_outputs;
    for _ in 0..number_of_outputs {
        tx.add_output(value_per_output.try_into().unwrap(), &receiver.vk);
    }
    tx.add_input(prev_tx_hash, output_idx);

    tx.sing_input_and_finalize(0, &sender.sk).unwrap()
}
