use std::collections::{HashMap, HashSet};

use rand::Rng;

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
        }
    }
}

impl<const N: usize> Node<N> for TrustedNode<N> {
    fn followees_set(&mut self, followees: [bool; N]) {
        self.followees = followees;
    }

    fn pending_txs_set(&mut self, pending_txs: HashSet<Tx>) {
        self.pending_txs = pending_txs;
    }

    fn followers_send(&self) -> &HashSet<Tx> {
        &self.pending_txs
    }

    fn followees_receive(&mut self, candidates: &Vec<Candidate>) {
        self.num_rounds -= 1;
        // consensus can be reached on a tx if all my followees propose it back to
        // me (minus the estimated count of byzantine nodes)
        // example: if I have 10 followees, and p of a node being a byzantine one is 10%
        // then I need to receive it from 9 of my followees, and then I can
        // add it to txs on which consensus was reached
        todo!()
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
    choosen_txs: HashSet<Tx>,
}

impl<const N: usize> ByzantineNode<N> {
    pub fn new(
        behaviour: ByzantineBehaviour,
        p_graph: f64,
        p_byzantine: f64,
        p_tx_dist: f64,
        num_rounds: u64,
    ) -> Self {
        Self {
            behaviour,
            p_graph,
            p_byzantine,
            p_tx_dist,
            num_rounds,
            followees: [false; N],
            pending_txs: HashSet::new(),
            choosen_txs: HashSet::new(),
        }
    }

    fn dead(&mut self) {
        self.choosen_txs = HashSet::new();
    }

    fn selfish(&mut self) {
        self.choosen_txs = self.pending_txs.clone();
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

    fn followees_receive(&mut self, candidates: &Vec<Candidate>) {
        self.num_rounds -= 1;

        match self.behaviour {
            ByzantineBehaviour::Dead => self.dead(),
            ByzantineBehaviour::Selfish => self.selfish(),
            ByzantineBehaviour::Mix(p) => {
                if rand::thread_rng().gen_bool(p) {
                    self.dead();
                    return;
                }
                self.selfish();
            }
        }

        todo!()
    }
}
