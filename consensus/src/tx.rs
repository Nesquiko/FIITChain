#[derive(Eq, PartialEq, Hash)]
pub struct Tx {
    pub id: u64,
}

impl Tx {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

pub struct Candidate {
    tx: u64,
    sender: u64,
}

impl Candidate {
    pub fn new(tx: u64, sender: u64) -> Self {
        Self { tx, sender }
    }
}
