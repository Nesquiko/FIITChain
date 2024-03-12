use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    sync::mpsc,
    time::Instant,
    u64, usize,
};

use consensus::{
    node::{ByzantineBehaviour, ByzantineNode, Node, TrustedNode},
    tx::{Candidate, Tx},
};
use rand::{rngs::StdRng, Rng, SeedableRng};

const NODES: usize = 100;

#[test]
fn simulations() {
    env_logger::init();

    let rounds_params = [10, 20, 30];
    let txs_params = [500, 1000, 1500];
    let p_graph_params = [0.1, 0.2, 0.3];
    let p_byzantine_params = [0.15, 0.3, 0.45];
    let p_tx_dist_params = [0.01, 0.05, 0.1];
    let total = rounds_params.len()
        * txs_params.len()
        * p_graph_params.len()
        * p_byzantine_params.len()
        * p_tx_dist_params.len();
    let mut permutations: Vec<(u64, u64, f64, f64, f64)> = Vec::with_capacity(total);
    let mut current = 0;

    for &rounds in rounds_params.iter() {
        for &txs in txs_params.iter() {
            for &p_graph in p_graph_params.iter() {
                for &p_byzantine in p_byzantine_params.iter() {
                    for &p_tx_dist in p_tx_dist_params.iter() {
                        permutations.push((rounds, txs, p_graph, p_byzantine, p_tx_dist));
                    }
                }
            }
        }
    }

    let (tx, rx) = mpsc::channel::<String>();
    for permutation in permutations {
        current += 1;
        let tx = tx.clone();
        rayon::spawn(move || {
            let mut tries = 0;
            let rounds = permutation.0;
            let txs = permutation.1;
            let p_graph = permutation.2;
            let p_byzantine = permutation.3;
            let p_tx_dist = permutation.4;

            let (mut result, mut passed) = simulation(rounds, txs, p_graph, p_byzantine, p_tx_dist);

            while tries < 3 && !passed {
                tries += 1;
                log::info!("Retrying {}", current);
                (result, passed) = simulation(rounds, txs, p_graph, p_byzantine, p_tx_dist);
            }

            tx.send(result).unwrap();
            log::info!("Finished {}/{}", current, total);
        })
    }

    let mut file = File::create("/tmp/sim-result.txt").unwrap();
    for received in rx {
        write!(file, "{}\n", received).unwrap();
    }
}

fn simulation(
    rounds: u64,
    txs: u64,
    p_graph: f64,
    p_byzantine: f64,
    p_tx_dist: f64,
) -> (String, bool) {
    log::debug!(
        "========= starting simulation with {} rounds with {} txs =========
        - probability that an edge will exist = {}
        - probability that a Node is byzantine = {}
        - probability that a tx will be distrubed to a Node = {}",
        rounds,
        txs,
        p_graph,
        p_byzantine,
        p_tx_dist
    );
    let mut result = format!(
        "rounds: {} | txs: {} | p_graph: {} | p_byzantine: {} | p_tx_dist: {}",
        rounds, txs, p_graph, p_byzantine, p_tx_dist
    );

    let mut before = Instant::now();
    let (mut nodes, valid_tx_ids, followees, seeds) =
        init(rounds, txs, p_graph, p_byzantine, p_tx_dist);
    result.push_str(&format!(
        " | initialized in {:.3?} | seeds: {:?}",
        before.elapsed(),
        seeds
    ));
    log::debug!("initialized in {:.3?}", before.elapsed());

    before = Instant::now();
    for _ in 0..rounds {
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
    result.push_str(&format!(" | simulation done in {:.3?}", before.elapsed()));
    log::debug!("simulation done in {:.3?}", before.elapsed());

    let (res, passed) = results(&nodes);
    result.push_str(&res);

    (result, passed)
}

/// Returns initialized Nodes, set of valid tx ids and a followers/followee matrix,
/// and tuple containing seed for the rng used in simulation and seed used in
/// byzantine nodes.
///
/// # Arguments
///
/// * `p_graph` - probability that an edge will exist, can be .1, .2, .3
/// * `p_byzantine` - probability that a Node is byzantine, can be .15, .3, .45
/// * `p_tx_dist` - probability that a tx will be distrubed to a Node, can be .01, .05, .1
fn init(
    rounds: u64,
    txs: u64,
    p_graph: f64,
    p_byzantine: f64,
    p_tx_dist: f64,
) -> (
    Vec<Box<dyn Node<NODES>>>,
    HashSet<Tx>,
    [[bool; NODES]; NODES],
    (u64, u64),
) {
    let mut nodes: Vec<Box<dyn Node<NODES>>> = Vec::with_capacity(NODES);
    let seed: u64 = rand::thread_rng().gen();
    log::debug!("rng seed {}", seed);
    let byzantine_seed: u64 = rand::thread_rng().gen();
    log::debug!("byzantine rng seed {}", byzantine_seed);

    let mut rng = StdRng::seed_from_u64(seed);
    let byzantine_rng = StdRng::seed_from_u64(byzantine_seed);

    let mut byzantine = 0;
    for i in 0..NODES {
        let node: Box<dyn Node<NODES>>;
        if rng.gen_bool(p_byzantine) {
            let behaviour = match byzantine % 3 {
                0 => ByzantineBehaviour::Dead,
                1 => ByzantineBehaviour::Selfish,
                _ => ByzantineBehaviour::Mix(0.5),
            };
            node = Box::new(ByzantineNode::new(behaviour, rounds, byzantine_rng.clone()));
            byzantine += 1;
        } else {
            node = Box::new(TrustedNode::new(p_graph, p_byzantine, p_tx_dist, rounds));
        }

        nodes.insert(i, node);
    }
    log::debug!(
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
    for _ in 0..txs {
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

    (nodes, valid_tx_ids, followees, (seed, byzantine_seed))
}

fn results(nodes: &Vec<Box<dyn Node<NODES>>>) -> (String, bool) {
    let mut consensuses: HashSet<Vec<Tx>> = HashSet::new();
    for i in 0..NODES {
        let node = nodes.get(i).unwrap();
        if node.is_byzantine() {
            continue;
        }
        let mut txs: Vec<Tx> = node.followers_send().iter().map(|tx| *tx).collect();
        txs.sort();
        consensuses.insert(txs.clone());

        log::trace!(
            "Transaction ids that Node {} believes consensus on:\n\t{:?}",
            i,
            txs
        );
    }

    let mut result = String::new();
    let mut passed = true;
    if consensuses.len() != 1 {
        passed = false;
        result.push_str(&format!(
            " | {} different consensuses reached!",
            consensuses.len()
        ));
        log::debug!(
            "There are {} different consensuses reached",
            consensuses.len()
        );
    }
    result.push_str(&format!(
        " | count of tx upon which consensus was reached {}",
        consensuses.iter().collect::<Vec<_>>().get(0).unwrap().len()
    ));
    log::debug!(
        "count of tx upon which consensus was reached on {}",
        consensuses.iter().collect::<Vec<_>>().get(0).unwrap().len()
    );

    (result, passed)
}
