use std::hash::Hash;

use custom::test_util::DUMMY_CUSTOM_POSITION;

use super::*;

use super::super::formats::custom::test_util::{DUMMY_CONST_CUSTOM_SIGNAL, DUMMY_CONSUMER_FACTORS};

pub const DUMMY_CONST_SIGNAL: Signal = Signal::Const { value: 1. };

pub const DUMMY_PIPE_PARAMETERS: EmptyPipeParameters = EmptyPipeParameters {};

pub const DUMMY_PARSED_PIPE_PARAMETERS: PipeParameters = PipeParameters::Full {
    length: 1.,
    diameter: 2.,
    transmittance: 3.,
    roughness: 4.,
    zeta: 5.,
};

// TODO: move to some utils module
fn set_of<T: Clone + Eq + Hash>(values: &[T]) -> HashSet<T> {
    HashSet::from_iter(values.iter().cloned())
}

fn create_test_nodes_and_edges(
    num_nodes: usize,
    edges: &[(usize, usize)],
) -> (Vec<Node>, Vec<Edge>, Vec<EmptyPipeParameters>) {
    let nodes = (0..num_nodes)
        .map(|i| Node::Zero {
            name: format!("N{}", i),
            position: DUMMY_CUSTOM_POSITION,
        })
        .collect();

    let edges: Vec<_> = edges
        .iter()
        .cloned()
        .map(|(src, tgt)| Edge { src, tgt })
        .collect();

    let edge_parameters = edges.iter().map(|_| DUMMY_PIPE_PARAMETERS).collect();

    (nodes, edges, edge_parameters)
}

#[test]
fn from_custom_network() {
    let feed_edges = [
        (0, 1),
        (1, 2),
        (1, 4),
        (2, 5),
        (3, 4),
        (3, 6),
        (4, 5),
        (5, 6),
        (7, 8),
    ];

    let num_feed_nodes = 10;
    let mut edges = Vec::with_capacity(20);

    for (src, tgt) in feed_edges {
        edges.push((src, tgt));
        edges.push((src + num_feed_nodes, tgt + num_feed_nodes));
    }

    let custom_network =
        custom::test_util::create_test_net(20, num_feed_nodes, &edges, &[5, 6], &[0]);

    let network: Network<FullPipeParameters> = custom_network
        .try_into()
        .expect("could not convert custom network into internal network type");

    let scaled_dummy_const_signal: Signal = DUMMY_CONST_CUSTOM_SIGNAL
        .scale_data(DUMMY_CONSUMER_FACTORS.yearly_demand / HOURS_PER_YEAR)
        .try_into()
        .expect("could not convert custom signal");

    assert_eq!(
        network.nodes().cloned().collect::<Vec<_>>(),
        [
            Node::Zero {
                name: String::from("N1"),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Zero {
                name: String::from("N2"),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Zero {
                name: String::from("N3"),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Zero {
                name: String::from("N4"),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Demand {
                name: String::from("N5"),
                demand: scaled_dummy_const_signal.clone(),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Demand {
                name: String::from("N6"),
                demand: scaled_dummy_const_signal.clone(),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Pressure {
                name: String::from("N0"),
                pressure: DUMMY_CONST_SIGNAL,
                temperature: DUMMY_CONST_SIGNAL,
                position: DUMMY_CUSTOM_POSITION,
            },
        ]
        .to_vec()
    );
    assert_eq!(network.demand_nodes.len(), 6);

    assert_eq!(
        network.edges().cloned().collect::<Vec<_>>(),
        [
            (6, 0), // spanning tree edges
            (0, 1),
            (0, 3),
            (1, 4),
            (2, 3),
            (4, 5),
            (2, 5), // cycle edges
            (3, 4),
        ]
        .map(|(src, tgt)| Edge { src, tgt })
        .into_iter()
        .collect::<Vec<_>>()
    );
    assert_eq!(network.spanning_tree_edges.len(), 6);
}

#[test]
fn extract_nodes_of_custom_net() {
    let custom_net = custom::test_util::create_test_net(10, 5, &[(0, 1), (1, 2)], &[3, 4], &[0]);

    let nodes = extract_nodes(&custom_net).expect("could not extract nodes from custom net");
    assert_eq!(nodes.len(), 10);

    let scaled_dummy_const_signal: Signal = DUMMY_CONST_CUSTOM_SIGNAL
        .scale_data(DUMMY_CONSUMER_FACTORS.yearly_demand / HOURS_PER_YEAR)
        .try_into()
        .expect("could not convert suctom signal");
    assert_eq!(
        nodes.into_iter().take(5).collect::<Vec<_>>(),
        vec![
            Node::Pressure {
                name: String::from("N0"),
                pressure: DUMMY_CONST_SIGNAL,
                temperature: DUMMY_CONST_SIGNAL,
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Zero {
                name: String::from("N1"),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Zero {
                name: String::from("N2"),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Demand {
                name: String::from("N3"),
                demand: scaled_dummy_const_signal.clone(),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Demand {
                name: String::from("N4"),
                demand: scaled_dummy_const_signal.clone(),
                position: DUMMY_CUSTOM_POSITION,
            },
        ]
    );
}

#[test]
fn extract_edges_of_custom_net() {
    let edge_tuples = [(0, 1), (1, 2), (2, 3), (2, 4), (3, 4)];
    let custom_net = custom::test_util::create_test_net(10, 5, &edge_tuples, &[3, 4], &[0]);

    let nodes = extract_nodes(&custom_net).expect("could not extract nodes from custom net");
    let (edges, _) = extract_edges::<FullPipeParameters>(&custom_net, &nodes)
        .expect("could not extract edges from cutsom net");

    assert_eq!(
        edges,
        edge_tuples.map(|(src, tgt)| Edge { src, tgt }).to_vec()
    )
}

fn assert_find_feed(
    name: &str,
    num_nodes: usize,
    edges: &[(usize, usize)],
    start_node: usize,
    expected_nodes: &[usize],
    expected_edges: &[usize],
) {
    let (nodes, edges, _) = create_test_nodes_and_edges(num_nodes, edges);

    let (nodes, edges) =
        find_feed(&nodes, &edges, start_node).expect("could not find feed of network");

    let expected_nodes = set_of(expected_nodes);
    let expected_edges = set_of(expected_edges);

    assert_eq!(
        nodes, expected_nodes,
        "feed nodes not as expected in test case '{}'",
        name
    );
    assert_eq!(
        edges, expected_edges,
        "feed edges not as expected in test case '{}'",
        name
    );
}

#[test]
fn find_feed_of_network() {
    assert_find_feed("one edge", 2, &[(0, 1)], 0, &[0, 1], &[0]);
    assert_find_feed(
        "small loop",
        3,
        &[(0, 1), (0, 2), (1, 2)],
        0,
        &[0, 1, 2],
        &[0, 1, 2],
    );
    assert_find_feed(
        "disconnected loops",
        6,
        &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5)],
        0,
        &[0, 1, 2],
        &[0, 1, 2],
    );
}

fn assert_filter_network(
    name: &str,
    num_nodes: usize,
    edges: &[(usize, usize)],
    nodes_to_keep: &[usize],
    edges_to_keep: &[usize],
    expected_nodes: &[usize],
    expected_edges: &[(usize, usize)],
) {
    let (nodes, edges, edge_parameters) = create_test_nodes_and_edges(num_nodes, edges);

    let nodes_to_keep = nodes_to_keep.iter().cloned().collect();
    let edges_to_keep = edges_to_keep.iter().cloned().collect();

    let (filtered_nodes, filtered_edges, _) =
        filter_network(nodes, edges, edge_parameters, nodes_to_keep, edges_to_keep)
            .expect("filtering did not work");

    assert_eq!(
        filtered_nodes,
        expected_nodes
            .iter()
            .map(|i| Node::Zero {
                name: format!("N{}", i),
                position: DUMMY_CUSTOM_POSITION,
            })
            .collect::<Vec<_>>(),
        "filtered nodes not as expected in test case '{}'",
        name
    );
    assert_eq!(
        filtered_edges,
        expected_edges
            .iter()
            .cloned()
            .map(|(src, tgt)| Edge { src, tgt })
            .collect::<Vec<_>>(),
        "filtered nodes not as expected in test case '{}'",
        name
    );
}

#[test]
fn test_filter_network() {
    assert_filter_network(
        "keep all",
        5,
        &[(0, 1), (1, 2), (2, 3), (3, 4)],
        &[0, 1, 2, 3, 4],
        &[0, 1, 2, 3],
        &[0, 1, 2, 3, 4],
        &[(0, 1), (1, 2), (2, 3), (3, 4)],
    );
    assert_filter_network(
        "remove one node at the end",
        5,
        &[(0, 1), (1, 2), (2, 3), (3, 4)],
        &[0, 1, 2, 3],
        &[0, 1, 2],
        &[0, 1, 2, 3],
        &[(0, 1), (1, 2), (2, 3)],
    );
    assert_filter_network(
        "remove all but one node",
        5,
        &[(0, 1), (1, 2), (2, 3), (3, 4)],
        &[0],
        &[],
        &[0],
        &[],
    );
    assert_filter_network(
        "remove first node",
        5,
        &[(0, 1), (1, 2), (2, 3), (3, 4)],
        &[1, 2, 3, 4],
        &[1, 2, 3],
        &[1, 2, 3, 4],
        &[(0, 1), (1, 2), (2, 3)],
    );
    assert_filter_network(
        "remove first node and middle",
        5,
        &[(0, 1), (1, 2), (2, 3), (3, 4)],
        &[1, 3, 4],
        &[3],
        &[1, 3, 4],
        &[(1, 2)],
    );
}

fn edges_from_pairs(pairs: &[(usize, usize)]) -> Vec<Edge> {
    pairs
        .iter()
        .cloned()
        .map(|(src, tgt)| Edge { src, tgt })
        .collect()
}

#[test]
fn from_feed() {
    let pressure_nodes = vec![Node::Pressure {
        name: String::from("N0"),
        pressure: DUMMY_CONST_SIGNAL,
        temperature: DUMMY_CONST_SIGNAL,
        position: DUMMY_CUSTOM_POSITION,
    }];
    let demand_nodes: Vec<Node> = (1..5)
        .map(|i| Node::Demand {
            name: format!("N{}", i),
            demand: DUMMY_CONST_SIGNAL,
            position: DUMMY_CUSTOM_POSITION,
        })
        .collect();
    let nodes = [pressure_nodes.clone(), demand_nodes.clone()].concat();

    let edges = [(0, 1), (0, 2), (1, 3), (1, 4), (4, 2)]
        .map(|(src, tgt)| Edge { src, tgt })
        .to_vec();

    let edge_parameters: Vec<EmptyPipeParameters> =
        (0..edges.len()).map(|_| DUMMY_PIPE_PARAMETERS).collect();

    let network = Network::try_from_feed(nodes, edges, edge_parameters.clone())
        .expect("could not compute the network");

    let expected_spanning_tree_edges = edges_from_pairs(&[(4, 0), (4, 1), (0, 2), (0, 3)]);
    let expected_cycle_edges = edges_from_pairs(&[(3, 1)]);
    let expected_pred_nodes = [(0, 4), (1, 4), (2, 0), (3, 0)].into_iter().collect();
    let expected_adjacent_edges = [
        (0, vec![0, 2, 3]),
        (1, vec![1, 4]),
        (2, vec![2]),
        (3, vec![3, 4]),
        (4, vec![0, 1]),
    ]
    .iter()
    .cloned()
    .collect();

    assert_eq!(
        network,
        Network {
            demand_nodes,
            pressure_nodes,
            root_node_index: 4,
            spanning_tree_edges: expected_spanning_tree_edges.clone(),
            cycle_edges: expected_cycle_edges.clone(),
            pred_nodes: expected_pred_nodes,
            edge_indices_by_connected_nodes: expected_spanning_tree_edges
                .iter()
                .chain(expected_cycle_edges.iter())
                .enumerate()
                .flat_map(|(i, Edge { src, tgt })| [
                    ((*src, *tgt), (i, false)),
                    ((*tgt, *src), (i, true)),
                ]
                .into_iter())
                .collect(),
            adjacent_edges: expected_adjacent_edges,
            edge_parameters,
        }
    );
}

fn assert_find_spanning_tree(
    name: &str,
    num_nodes: usize,
    edges: &[(usize, usize)],
    expected_spanning_tree: &[usize],
    expected_pred_nodes: &[(usize, usize)],
) {
    let (nodes, edges, _) = create_test_nodes_and_edges(num_nodes, edges);

    let (root_node_index, spanning_tree, cycle_edges, pred_nodes) =
        find_spanning_tree(&nodes, &edges).expect("could not compute spanning tree");

    let expected_spanning_tree = set_of(expected_spanning_tree);
    let expected_cycle_edges =
        HashSet::from_iter((0..edges.len()).filter(|i| !expected_spanning_tree.contains(i)));

    assert_eq!(root_node_index, 0);

    assert_eq!(
        spanning_tree, expected_spanning_tree,
        "spanning tree unexpected for the test case '{}'",
        name,
    );

    assert_eq!(
        cycle_edges, expected_cycle_edges,
        "cycle edges unexpected for the test case '{}'",
        name,
    );

    assert_eq!(pred_nodes, expected_pred_nodes.iter().cloned().collect());
}

#[test]
fn find_spanning_tree_of_net() {
    assert_find_spanning_tree("single edge", 2, &[(0, 1)], &[0], &[(1, 0)]);
    assert_find_spanning_tree(
        "two edges",
        3,
        &[(0, 1), (0, 2)],
        &[0, 1],
        &[(1, 0), (2, 0)],
    );
    assert_find_spanning_tree(
        "small cycle",
        3,
        &[(0, 1), (0, 2), (1, 2)],
        &[0, 1],
        &[(2, 1), (2, 0), (1, 0)],
    );
    assert_find_spanning_tree(
        "two cycles",
        8,
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (2, 4),
            (3, 5),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 1),
        ],
        &[0, 1, 2, 3, 4, 7, 8],
        &[(1, 0), (2, 1), (3, 2), (4, 2), (5, 3), (7, 1), (6, 7)],
    );
    assert_find_spanning_tree(
        "two cycles",
        8,
        &[(0, 2), (0, 1), (1, 2), (1, 3), (2, 3), (2, 4)],
        &[0, 1, 4, 5],
        &[(1, 0), (2, 0), (3, 2), (4, 2)],
    );
}

#[test]
fn reordering_demand_nodes() {
    let zero = Signal::Const { value: 0. };

    let nodes = vec![
        Node::Zero {
            name: String::from("N0"),
            position: DUMMY_CUSTOM_POSITION,
        },
        Node::Pressure {
            name: String::from("N1"),
            pressure: zero.clone(),
            temperature: zero.clone(),
            position: DUMMY_CUSTOM_POSITION,
        },
        Node::Demand {
            name: String::from("N2"),
            demand: zero.clone(),
            position: DUMMY_CUSTOM_POSITION,
        },
        Node::Zero {
            name: String::from("N3"),
            position: DUMMY_CUSTOM_POSITION,
        },
        Node::Zero {
            name: String::from("N4"),
            position: DUMMY_CUSTOM_POSITION,
        },
        Node::Pressure {
            name: String::from("N5"),
            pressure: zero.clone(),
            temperature: zero.clone(),
            position: DUMMY_CUSTOM_POSITION,
        },
    ];

    let edges = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)];

    let edges: Vec<Edge> = edges
        .into_iter()
        .map(|(src, tgt)| Edge { src, tgt })
        .collect();

    let (demand_nodes, pressure_nodes, edges) = split_nodes(nodes, edges);

    assert_eq!(
        demand_nodes,
        vec![
            Node::Zero {
                name: String::from("N0"),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Demand {
                name: String::from("N2"),
                demand: zero.clone(),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Zero {
                name: String::from("N3"),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Zero {
                name: String::from("N4"),
                position: DUMMY_CUSTOM_POSITION,
            },
        ],
    );
    assert_eq!(
        pressure_nodes,
        vec![
            Node::Pressure {
                name: String::from("N1"),
                pressure: zero.clone(),
                temperature: zero.clone(),
                position: DUMMY_CUSTOM_POSITION,
            },
            Node::Pressure {
                name: String::from("N5"),
                pressure: zero.clone(),
                temperature: zero.clone(),
                position: DUMMY_CUSTOM_POSITION,
            },
        ],
    );

    assert_eq!(
        edges,
        [(0, 4), (4, 1), (1, 2), (2, 3), (3, 5), (5, 0),]
            .map(|(src, tgt)| Edge { src, tgt })
            .into_iter()
            .collect::<Vec<_>>()
    );
}
