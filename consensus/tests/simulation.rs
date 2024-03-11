use std::{
    collections::{HashMap, HashSet},
    usize,
};

use consensus::{
    node::{ByzantineBehaviour, ByzantineNode, Node, TrustedNode},
    tx::{Candidate, Tx},
};
use rand::{rngs::StdRng, Rng, SeedableRng};

const ROUNDS: u64 = 10;
const NODES: usize = 100;
const TXS: usize = 500;

#[test]
fn simulation() {
    env_logger::init();

    let p_graph: f64 = 0.1;
    let p_byzantine: f64 = 0.45;
    let p_tx_dist: f64 = 0.01;

    let (mut nodes, valid_tx_ids, followees) = init(p_graph, p_byzantine, p_tx_dist);

    for _ in 0..ROUNDS {
        // key is the index of a Node and value is vec of candidate txs from
        // other nodes
        let mut all_proposals: HashMap<usize, Vec<Candidate>> = HashMap::new();

        for i in 0..NODES {
            let proposals = nodes.get(i).unwrap().followers_send();
            for tx in proposals.iter() {
                if !valid_tx_ids.contains(&tx) {
                    continue; // controls that each tx is valid
                }

                // for each of nodes followers, add tx to their proposals for this turn
                for j in 0..NODES {
                    if !followees[j][i] {
                        continue; // tx is only proposed if `j` follows `i`
                    }

                    let candidate = Candidate::new(*tx, i.try_into().unwrap());
                    all_proposals.entry(j).or_insert(vec![]).push(candidate);
                }
            }
        }

        // distributes proposals made in this turn to followers
        for i in 0..NODES {
            if !all_proposals.contains_key(&i) {
                continue;
            }
            nodes
                .get_mut(i)
                .unwrap()
                .followees_receive(all_proposals.get(&i).unwrap());
        }
    }

    results(&nodes);
}

/// Returns initialized Nodes, set of valid tx ids and a followers/followee matrix.
///
/// # Arguments
///
/// * `p_graph` - probability that an edge will exist, can be .1, .2, .3
/// * `p_byzantine` - probability that a Node is byzantine, can be .15, .3, .45
/// * `p_tx_dist` - probability that a tx will be distrubed to a Node, can be .01, .05, .1
fn init(
    p_graph: f64,
    p_byzantine: f64,
    p_tx_dist: f64,
) -> (
    Vec<Box<dyn Node<NODES>>>,
    HashSet<Tx>,
    [[bool; NODES]; NODES],
) {
    let mut nodes: Vec<Box<dyn Node<NODES>>> = Vec::with_capacity(NODES);
    // Repeatable randomness, either change seed or use `rand::thread_rng()`
    // for pseudo randomness
    let mut rng = StdRng::seed_from_u64(23195820964131346);
    let byzantine_rng = StdRng::seed_from_u64(889237982352315235);
    let mut byzantine = 0;
    for i in 0..NODES {
        let node: Box<dyn Node<NODES>>;
        if rng.gen_bool(p_byzantine) {
            let behaviour = match byzantine % 3 {
                0 => ByzantineBehaviour::Dead,
                1 => ByzantineBehaviour::Selfish,
                _ => ByzantineBehaviour::Mix(0.5),
            };
            node = Box::new(ByzantineNode::new(behaviour, ROUNDS, byzantine_rng.clone()));
            byzantine += 1;
        } else {
            node = Box::new(TrustedNode::new(p_graph, p_byzantine, p_tx_dist, ROUNDS));
        }

        nodes.insert(i, node);
    }
    log::info!(
        "There are {} trusted nodes and {} byzantine",
        NODES - byzantine,
        byzantine
    );

    let mut followees: [[bool; NODES]; NODES] = [[false; NODES]; NODES];
    for i in 0..NODES {
        for j in 0..NODES {
            if i == j {
                continue;
            }

            if rng.gen_bool(p_graph) {
                followees[i][j] = true;
            }
        }
    }

    for i in 0..NODES {
        nodes.get_mut(i).unwrap().followees_set(followees[i]);
    }

    let mut valid_tx_ids: HashSet<Tx> = HashSet::new();
    for _ in 0..TXS {
        valid_tx_ids.insert(rng.gen::<Tx>());
    }

    for i in 0..NODES {
        let mut pending_txs: HashSet<Tx> = HashSet::new();
        for id in valid_tx_ids.iter() {
            if rng.gen_bool(p_tx_dist) {
                pending_txs.insert(*id);
            }
        }
        nodes.get_mut(i).unwrap().pending_txs_set(pending_txs);
    }

    (nodes, valid_tx_ids, followees)
}

fn results(nodes: &Vec<Box<dyn Node<NODES>>>) {
    let mut consensus: HashSet<Vec<Tx>> = HashSet::new();
    for i in 0..NODES {
        let node = nodes.get(i).unwrap();
        if node.is_byzantine() {
            continue;
        }
        let mut txs: Vec<Tx> = node.followers_send().iter().map(|tx| *tx).collect();
        txs.sort();
        consensus.insert(txs.clone());

        log::trace!(
            "Transaction ids that Node {} believes consensus on:\n\t{:?}",
            i,
            txs
        );
    }

    log::info!(
        "There are {} different consensuses reached",
        consensus.len()
    );
    assert_eq!(consensus.len(), 1);
    log::info!(
        "count of tx upon which consensus was reached on {}",
        consensus.iter().collect::<Vec<_>>().get(0).unwrap().len()
    );
}
