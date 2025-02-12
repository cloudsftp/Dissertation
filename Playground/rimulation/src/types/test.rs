use std::hash::Hash;

use super::*;

use formats::custom::test_util::{DUMMY_CONST_SIGNAL, DUMMY_CONSUMER_FACTORS};

const DUMMY_PIPE_PARAMETERS: PipeParameters = PipeParameters {
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
) -> (Vec<Node>, Vec<Edge>) {
    let nodes = (0..num_nodes)
        .map(|i| Node::Zero {
            name: format!("N{}", i),
        })
        .collect();
    let edges = edges
        .iter()
        .cloned()
        .map(|(src, tgt)| Edge {
            src,
            tgt,
            parameters: DUMMY_PIPE_PARAMETERS,
        })
        .collect();

    (nodes, edges)
}

#[test]
fn test_from_custom_network() {
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

    for (i, j) in feed_edges {
        edges.push((i, j));
        edges.push((i + num_feed_nodes, j + num_feed_nodes));
    }

    let custom_network =
        custom::test_util::create_test_net(20, num_feed_nodes, &edges, &[5, 6], &[0]);

    let network: Network = custom_network
        .try_into()
        .expect("could not convert custom network into internal network type");

    let scaled_dummy_const_signal = custom::test_util::DUMMY_CONST_SIGNAL
        .scale_data(DUMMY_CONSUMER_FACTORS.yearly_demand / HOURS_PER_YEAR);
    assert_eq!(
        network.nodes,
        [
            Node::Pressure {
                name: String::from("N0"),
                pressure: DUMMY_CONST_SIGNAL,
                temperature: DUMMY_CONST_SIGNAL
            },
            Node::Zero {
                name: String::from("N1")
            },
            Node::Zero {
                name: String::from("N2")
            },
            Node::Zero {
                name: String::from("N3")
            },
            Node::Zero {
                name: String::from("N4")
            },
            Node::Demand {
                name: String::from("N5"),
                demand: scaled_dummy_const_signal.clone(),
            },
            Node::Demand {
                name: String::from("N6"),
                demand: scaled_dummy_const_signal.clone(),
            },
        ]
        .to_vec()
    );

    assert_eq!(
        network.edges,
        [
            (0, 1), // spanning tree edges
            (1, 2),
            (1, 4),
            (2, 5),
            (3, 4),
            (5, 6),
            (3, 6), // cycle edges
            (4, 5),
        ]
        .map(|(i, j)| Edge {
            src: i,
            tgt: j,
            parameters: DUMMY_PIPE_PARAMETERS
        })
        .into_iter()
        .collect::<Vec<_>>()
    );
}

#[test]
fn test_extract_nodes() {
    let custom_net = custom::test_util::create_test_net(10, 5, &[(0, 1), (1, 2)], &[3, 4], &[0]);

    let nodes = extract_nodes(&custom_net).expect("could not extract nodes from custom net");
    assert_eq!(nodes.len(), 10);

    let scaled_dummy_const_signal = custom::test_util::DUMMY_CONST_SIGNAL
        .scale_data(DUMMY_CONSUMER_FACTORS.yearly_demand / HOURS_PER_YEAR);
    assert_eq!(
        nodes.into_iter().take(5).collect::<Vec<_>>(),
        vec![
            Node::Pressure {
                name: String::from("N0"),
                pressure: custom::test_util::DUMMY_CONST_SIGNAL,
                temperature: custom::test_util::DUMMY_CONST_SIGNAL
            },
            Node::Zero {
                name: String::from("N1")
            },
            Node::Zero {
                name: String::from("N2")
            },
            Node::Demand {
                name: String::from("N3"),
                demand: scaled_dummy_const_signal.clone(),
            },
            Node::Demand {
                name: String::from("N4"),
                demand: scaled_dummy_const_signal.clone(),
            },
        ]
    );
}

#[test]
fn test_extract_edges() {
    let edge_tuples = [(0, 1), (1, 2), (2, 3), (2, 4), (3, 4)];
    let custom_net = custom::test_util::create_test_net(10, 5, &edge_tuples, &[3, 4], &[0]);

    let nodes = extract_nodes(&custom_net).expect("could not extract nodes from custom net");
    let edges =
        extract_edges(&custom_net, &nodes).expect("could not extract edges from cutsom net");

    assert_eq!(
        edges,
        edge_tuples
            .map(|(i, j)| Edge {
                src: i,
                tgt: j,
                parameters: DUMMY_PIPE_PARAMETERS
            })
            .to_vec()
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
    let (nodes, edges) = create_test_nodes_and_edges(num_nodes, edges);

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
fn test_find_feed() {
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
    let (nodes, edges) = create_test_nodes_and_edges(num_nodes, edges);

    let nodes_to_keep = nodes_to_keep.into_iter().cloned().collect();
    let edges_to_keep = edges_to_keep.into_iter().cloned().collect();

    let (filtered_nodes, filtered_edges) =
        filter_network(nodes, edges, nodes_to_keep, edges_to_keep).expect("filtering did not work");

    assert_eq!(
        filtered_nodes,
        expected_nodes
            .iter()
            .map(|i| Node::Zero {
                name: format!("N{}", i)
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
            .map(|(i, j)| Edge {
                src: i,
                tgt: j,
                parameters: DUMMY_PIPE_PARAMETERS
            })
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

fn assert_find_spanning_tree(
    name: &str,
    num_nodes: usize,
    edges: &[(usize, usize)],
    expected_spanning_tree: &[usize],
) {
    let (nodes, edges) = create_test_nodes_and_edges(num_nodes, edges);

    let (spanning_tree, cycle_edges) =
        find_spanning_tree(&nodes, &edges).expect("could not compute spanning tree");

    let expected_spanning_tree = set_of(expected_spanning_tree);
    let expected_cycle_edges =
        HashSet::from_iter((0..edges.len()).filter(|i| !expected_spanning_tree.contains(i)));
    assert_eq!(
        spanning_tree, expected_spanning_tree,
        "spanning tree unexpected for the test case '{}'",
        name,
    );

    assert_eq!(
        cycle_edges, expected_cycle_edges,
        "cycle edges unexpected for the test case '{}'",
        name,
    )
}

#[test]
fn test_find_spanning_tree() {
    assert_find_spanning_tree("single edge", 2, &[(0, 1)], &[0]);
    assert_find_spanning_tree("two edges", 3, &[(0, 1), (0, 2)], &[0, 1]);
    assert_find_spanning_tree("small cycle", 3, &[(0, 1), (0, 2), (1, 2)], &[0, 1]);
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
    );
    assert_find_spanning_tree(
        "two cycles",
        8,
        &[(0, 2), (0, 1), (1, 2), (1, 3), (2, 3), (2, 4)],
        &[0, 1, 4, 5],
    );
}
