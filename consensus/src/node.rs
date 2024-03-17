use std::collections::{HashMap, HashSet};

use rand::{rngs::StdRng, Rng};

use crate::tx::{Candidate, Tx};

pub trait Node<const N: usize> {
    /// If `ith` entry is `true` then this Node follows the `ith` Node
    fn followees_set(&mut self, followees: [bool; N]);

    /// Initializes proposed set of txs
    fn pending_txs_set(&mut self, pending_txs: HashSet<Tx>);

    /// Returns proposed txs, which shall be send to this Node's followers.
    /// After final round, the behaviour changes and it will return txs,
    /// on which consensus was reached.
    fn followers_send(&self) -> &HashSet<Tx>;

    /// Candites from different Nodes
    fn followees_receive(&mut self, candidates: &Vec<Candidate>);

    fn is_byzantine(&self) -> bool;
}

pub struct TrustedNode<const N: usize> {
    /// Probability of an edge existing in graph
    p_graph: f64,
    /// Probability of a Node will be set as byzantine
    p_byzantine: f64,
    /// Probability of assigning a tx to a Node
    p_tx_dist: f64,
    /// Number of rounds in simulation
    num_rounds: u64,
    /// This node's followers, if `i` is true, then this node follows `ith` node
    followees: [bool; N],
    /// The initial set of txs given to this Node
    pending_txs: HashSet<Tx>,
    /// Map of txs to set of its proposers
    received_txs: HashMap<Tx, HashSet<u64>>,
    consensus_reached: HashSet<Tx>,
    /// how many of this nodes followees must confirm a tx in order to reach a
    /// consensus on it, minimum 1
    consensus_threshold: usize,
}

impl<const N: usize> TrustedNode<N> {
    pub fn new(p_graph: f64, p_byzantine: f64, p_tx_dist: f64, num_rounds: u64) -> Self {
        Self {
            p_graph,
            p_byzantine,
            p_tx_dist,
            num_rounds,
            followees: [false; N],
            pending_txs: HashSet::new(),
            received_txs: HashMap::new(),
            consensus_reached: HashSet::new(),
            consensus_threshold: 0,
        }
    }
}

impl<const N: usize> Node<N> for TrustedNode<N> {
    fn followees_set(&mut self, followees: [bool; N]) {
        self.followees = followees;
        let probable_followers = (N as f64 * self.p_graph).ceil();
        let probable_byzantines = (N as f64 * self.p_byzantine).ceil();
        self.consensus_threshold = f64::min(probable_followers - probable_byzantines, 1.) as usize;
    }

    fn pending_txs_set(&mut self, pending_txs: HashSet<Tx>) {
        self.pending_txs = pending_txs;
    }

    fn followers_send(&self) -> &HashSet<Tx> {
        if self.num_rounds == 0 {
            &self.consensus_reached
        } else {
            &self.pending_txs
        }
    }

    fn followees_receive(&mut self, candidates: &Vec<Candidate>) {
        self.num_rounds -= 1;

        for candidate in candidates.iter() {
            self.received_txs
                .entry(candidate.tx)
                .or_insert(HashSet::new())
                .insert(candidate.sender);

            if self.received_txs.get(&candidate.tx).unwrap().len() >= self.consensus_threshold {
                self.consensus_reached.insert(candidate.tx);
            }

            self.pending_txs.insert(candidate.tx);
        }
    }

    fn is_byzantine(&self) -> bool {
        false
    }
}

pub enum ByzantineBehaviour {
    /// Doesn't send any txs
    Dead,
    /// Only sends its own set of txs and never resends received ones
    Selfish,
    /// Mixes all other behaviour with given f64 probability for any of them
    Mix(f64),
}

pub struct ByzantineNode<const N: usize> {
    behaviour: ByzantineBehaviour,
    /// Number of rounds in simulation
    num_rounds: u64,
    /// This node's followers, if `i` is true, then this node follows `ith` node
    followees: [bool; N],
    /// The initial set of txs given to this Node
    pending_txs: HashSet<Tx>,
    choosen_txs: HashSet<Tx>,
    rng: StdRng,
}

impl<const N: usize> ByzantineNode<N> {
    pub fn new(behaviour: ByzantineBehaviour, num_rounds: u64, rng: StdRng) -> Self {
        Self {
            behaviour,
            num_rounds,
            followees: [false; N],
            pending_txs: HashSet::new(),
            choosen_txs: HashSet::new(),
            rng,
        }
    }
}

impl<const N: usize> Node<N> for ByzantineNode<N> {
    fn followees_set(&mut self, followees: [bool; N]) {
        self.followees = followees;
    }

    fn pending_txs_set(&mut self, pending_txs: HashSet<Tx>) {
        self.choosen_txs = pending_txs.clone();
        self.pending_txs = pending_txs;
    }

    fn followers_send(&self) -> &HashSet<Tx> {
        &self.choosen_txs
    }

    fn followees_receive(&mut self, _candidates: &Vec<Candidate>) {
        self.num_rounds -= 1;

        match self.behaviour {
            ByzantineBehaviour::Dead => {
                self.choosen_txs = HashSet::new();
            }
            ByzantineBehaviour::Selfish => {
                self.choosen_txs = self.pending_txs.clone();
            }
            ByzantineBehaviour::Mix(p) => {
                if self.rng.gen_bool(p) {
                    self.choosen_txs = HashSet::new();
                    return;
                }
                self.choosen_txs = self.pending_txs.clone();
            }
        }
    }

    fn is_byzantine(&self) -> bool {
        true
    }
}
