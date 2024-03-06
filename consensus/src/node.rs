use std::collections::HashSet;

use crate::tx::{Candidate, Tx};

pub trait Node<const N: usize> {
    /// If `ith` entry is `true` then this Node follows the `ith` Node
    fn followees_set(&mut self, followees: [bool; N]);

    /// Initializes proposed set of txs
    fn pending_txs_set(&mut self, pending_txs: HashSet<Tx>);

    /// Returns proposed txs, which shall be send to this Node's followers.
    /// After final round, the behaviour changes and it will return txs,
    /// on which consensus was reached.
    fn followers_send(&self) -> HashSet<Tx>;

    /// Candites from different Nodes
    fn followees_receive(&mut self, candidates: &Vec<Candidate>);
}

pub struct TrustedNode {
    /// Probability of an edge existing in graph
    p_graph: f64,
    /// Probability of a Node will be set as byzantine
    p_byzantine: f64,
    /// Probability of assigning the first tx to a Node
    p_tx_dist: f64,
    /// Number of rounds in simulation
    num_rounds: u64,
}

impl TrustedNode {
    pub fn new(p_graph: f64, p_byzantine: f64, p_tx_dist: f64, num_rounds: u64) -> Self {
        Self {
            p_graph,
            p_byzantine,
            p_tx_dist,
            num_rounds,
        }
    }
}

impl<const N: usize> Node<N> for TrustedNode {
    fn followees_set(&mut self, followees: [bool; N]) {
        todo!()
    }

    fn pending_txs_set(&mut self, pending_txs: HashSet<Tx>) {
        todo!()
    }

    fn followers_send(&self) -> HashSet<Tx> {
        todo!()
    }

    fn followees_receive(&mut self, candidates: &Vec<Candidate>) {
        todo!()
    }
}

pub struct ByzantineNode {
    /// Probability of an edge existing in graph
    p_graph: f64,
    /// Probability of a Node will be set as byzantine
    p_byzantine: f64,
    /// Probability of assigning the first tx to a Node
    p_tx_dist: f64,
    /// Number of rounds in simulation
    num_rounds: u64,
}

impl ByzantineNode {
    pub fn new(p_graph: f64, p_byzantine: f64, p_tx_dist: f64, num_rounds: u64) -> Self {
        Self {
            p_graph,
            p_byzantine,
            p_tx_dist,
            num_rounds,
        }
    }
}

impl<const N: usize> Node<N> for ByzantineNode {
    fn followees_set(&mut self, followees: [bool; N]) {
        todo!()
    }

    fn pending_txs_set(&mut self, pending_txs: HashSet<Tx>) {
        todo!()
    }

    fn followers_send(&self) -> HashSet<Tx> {
        todo!()
    }

    fn followees_receive(&mut self, candidates: &Vec<Candidate>) {
        todo!()
    }
}
