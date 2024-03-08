pub type Tx = u64;

pub struct Candidate {
    tx: Tx,
    sender: u64,
}

impl Candidate {
    pub fn new(tx: Tx, sender: u64) -> Self {
        Self { tx, sender }
    }
}
