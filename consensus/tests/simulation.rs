use std::{
    collections::{HashMap, HashSet},
    usize,
};

use consensus::{
    node::{ByzantineNode, Node, TrustedNode},
    tx::{Candidate, Tx},
};
use rand::Rng;

const ROUNDS: u64 = 1_000;
const NODES: usize = 100;
const TXS: usize = 500;

#[test]
fn simulation() {
    let p_graph: f64 = 0.1; // can be .1, .2, .3
    let p_byzantine: f64 = 0.15; // can be .15, .3, .54
    let p_tx_dist: f64 = 0.01; // can be .01, .05, .1

    let (mut nodes, valid_tx_ids, followees) = init(p_graph, p_byzantine, p_tx_dist);

    for _ in 0..ROUNDS {
        let mut all_proposals: HashMap<usize, Vec<Candidate>> = HashMap::new();

        for i in 0..NODES {
            let proposals = nodes.get(i).unwrap().followers_send();
            for tx in proposals.iter() {
                if !valid_tx_ids.contains(&tx.id) {
                    continue;
                }

                for j in 0..NODES {
                    if !followees[j][i] {
                        continue;
                    }

                    if !all_proposals.contains_key(&j) {
                        all_proposals.insert(j, vec![]);
                    }
                    let candidate = Candidate::new(tx.id, i.try_into().unwrap());
                    all_proposals.get_mut(&j).unwrap().push(candidate);
                }
            }
        }

        for i in 0..NODES {
            if all_proposals.contains_key(&i) {
                nodes
                    .get_mut(i)
                    .unwrap()
                    .followees_receive(all_proposals.get(&i).unwrap());
            }
        }
    }

    results(&nodes);
}

fn init(
    p_graph: f64,
    p_byzantine: f64,
    p_tx_dist: f64,
) -> (
    Vec<Box<dyn Node<NODES>>>,
    HashSet<u64>,
    [[bool; NODES]; NODES],
) {
    let mut nodes: Vec<Box<dyn Node<NODES>>> = Vec::with_capacity(NODES);
    let mut rng = rand::thread_rng();
    for i in 0..NODES {
        let node: Box<dyn Node<NODES>>;
        if rng.gen_bool(p_byzantine) {
            node = Box::new(ByzantineNode::new(p_graph, p_byzantine, p_tx_dist, ROUNDS));
        } else {
            node = Box::new(TrustedNode::new(p_graph, p_byzantine, p_tx_dist, ROUNDS));
        }

        nodes.insert(i, node);
    }

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
        match nodes.get_mut(i) {
            Some(node) => node.followees_set(followees[i]),
            None => todo!(),
        }
    }

    let mut valid_tx_ids: HashSet<u64> = HashSet::new();
    for _ in 0..TXS {
        valid_tx_ids.insert(rng.gen::<u64>());
    }

    for i in 0..NODES {
        let mut pending_txs = HashSet::new();
        for id in valid_tx_ids.iter() {
            if rng.gen_bool(p_tx_dist) {
                pending_txs.insert(Tx::new(*id));
            }
        }
        nodes.get_mut(i).unwrap().pending_txs_set(pending_txs);
    }

    (nodes, valid_tx_ids, followees)
}

fn results(nodes: &Vec<Box<dyn Node<NODES>>>) {
    for i in 0..NODES {
        let txs_ids: Vec<u64> = nodes
            .get(i)
            .unwrap()
            .followers_send()
            .iter()
            .map(|tx| tx.id)
            .collect();

        log::info!(
            "Transaction ids that Node {} believes consensus on:\n\t{:?}",
            i,
            txs_ids
        );
    }
}
