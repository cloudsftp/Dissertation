mod formats;

use formats::{
    custom::{self, Input},
    NamedComponent,
};

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

const HOURS_PER_YEAR: f64 = 8760.;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataPoint {
    t: f64,
    v: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Signal {
    Const {
        scale: f64,
        data: f64,
    },
    Poly {
        degree: usize,
        scale: f64,
        data: Vec<DataPoint>,
    },
}

impl Signal {
    fn scale_data(self, factor: f64) -> Self {
        match self {
            Signal::Const { scale, data } => Signal::Const {
                scale,
                data: data * factor,
            },
            Signal::Poly {
                degree,
                scale,
                data,
            } => Signal::Poly {
                degree,
                scale,
                data: data
                    .into_iter()
                    .map(|DataPoint { t, v }| DataPoint { t, v: v * factor })
                    .collect(),
            },
        }
    }
}

#[derive(Debug)]
enum Node {
    Pressure {
        name: String,
        pressure: Signal,
        temperature: Signal,
    },
    Demand {
        name: String,
        demand: Signal,
    },
    Node {
        name: String,
    },
}

#[derive(Debug)]
struct Edge {
    src: usize,
    tgt: usize,
}

struct Network {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

fn set_of_names<T: NamedComponent>(
    elements: &[T],
    extract_node_name: fn(&T) -> String,
) -> HashMap<String, String> {
    elements
        .iter()
        .map(|element| (element.get_name(), extract_node_name(element)))
        .collect()
}

fn extract_nodes(value: &custom::Network) -> Result<Vec<Node>, Error> {
    let consumers_by_node =
        set_of_names(&value.topology.consumers, |consumer| consumer.src.clone());
    let sources_by_node = set_of_names(&value.topology.sources, |source| source.tgt.clone());

    let get_signal = |name: &String| -> Result<Signal, Error> {
        Ok(value
            .scenario
            .signals
            .get(name)
            .ok_or(anyhow!("signal with name '{}' does not exist", name))?
            .clone())
    };

    value
        .topology
        .nodes
        .iter()
        .map(|node| {
            Ok(
                if let Some(consumer_name) = consumers_by_node.get(&node.name) {
                    let consumer_input = &value
                        .scenario
                        .consumer_inputs
                        .get(consumer_name)
                        .ok_or(anyhow!("no inputs defined for consumer '{}'", node.name))?;

                    let demand_signal_name = match value.scenario.inputs.get(&consumer_input.input).ok_or(
                        anyhow!("input with name '{}' does not exist", consumer_input.input),
                    )? {
                        Input::Consumer{demand, return_temperature: _} => Ok(demand),
                        _ => Err(anyhow!("input with name '{}' has the wrong type, expected to be Input::Consumer", consumer_input.input)),
                    }?;

                    // TODO: why scaled by hours per year, not seconds?
                    let demand = get_signal(demand_signal_name)?.scale_data(consumer_input.factors.yearly_demand / HOURS_PER_YEAR);

                    Node::Demand {
                        name: node.name.clone(),
                        demand,
                    }
                } else if let Some(source_name) = sources_by_node.get(&node.name) {
                    let source_input_name = value.scenario.source_inputs.get(source_name).ok_or(anyhow!("no inputs defined for source '{}'", source_name))?;
                    let (pressure_signal_name, temperature_signal_name) = match value.scenario.inputs.get(source_input_name).ok_or(
                        anyhow!("input with name '{}' does not exist", source_input_name
                    ))? {
                        Input::Source{ base_pressure: _, pressure_lift, temperature } => Ok((pressure_lift, temperature)),
                        _ => Err(anyhow!("input with name '{}' has the wrong type, expected to be Input::Source", source_input_name)),
                    }?;

                    let pressure = get_signal(pressure_signal_name)?;
                    let temperature = get_signal(temperature_signal_name)?;

                    Node::Pressure {
                        name: node.name.clone(),
                        pressure: pressure,
                        temperature: temperature,
                    }
                } else {
                    Node::Node { name: node.name.clone() }
                },
            )
        })
        .collect()
}

impl TryFrom<custom::Network> for Network {
    type Error = Error;

    fn try_from(value: custom::Network) -> Result<Self, Self::Error> {
        let nodes = extract_nodes(&value)?;

        // prepare [node], [edge]

        // only keep the ones in feed
        // find spanning tree, (find pressure paths), reorder
        todo!();
    }
}

fn get_adjacent_edges(nodes: &[Node], edges: &[Edge]) -> HashMap<usize, Vec<usize>> {
    nodes
        .iter()
        .enumerate()
        .map(|(node_idx, _)| {
            (
                node_idx,
                edges
                    .iter()
                    .enumerate()
                    .filter(|(_, edge)| edge.src == node_idx || edge.tgt == node_idx)
                    .map(|(edge_idx, _)| edge_idx)
                    .collect(),
            )
        })
        .collect()
}

fn collect_feed(
    feed_nodes: &mut HashSet<usize>,
    feed_edges: &mut HashSet<usize>,
    current_node: usize,
    adjacent_edges: &HashMap<usize, Vec<usize>>,
    edges: &[Edge],
) -> Result<(), Error> {
    if feed_nodes.contains(&current_node) {
        return Ok(());
    }
    feed_nodes.insert(current_node);

    let walkable_edges = adjacent_edges.get(&current_node).ok_or(anyhow!(
        "node '{}' does not have any adjacent edges",
        current_node
    ))?;
    for edge_idx in walkable_edges {
        let edge = &edges
            .get(*edge_idx)
            .ok_or(anyhow!("edge '{}' does not exist", edge_idx))?;
        let next_node = edge.get_other_node(current_node)?;

        feed_edges.insert(*edge_idx);
        collect_feed(feed_nodes, feed_edges, next_node, adjacent_edges, edges)?;
    }

    Ok(())
}

fn find_feed(
    nodes: &[Node],
    edges: &[Edge],
    start_node: usize,
) -> Result<(HashSet<usize>, HashSet<usize>), Error> {
    let adjacent_edges = get_adjacent_edges(nodes, edges);

    let mut feed_nodes = HashSet::new();
    let mut feed_edges = HashSet::new();

    collect_feed(
        &mut feed_nodes,
        &mut feed_edges,
        start_node,
        &adjacent_edges,
        edges,
    )?;

    Ok((feed_nodes, feed_edges))
}

fn find_spanning_tree(
    nodes: &[Node],
    edges: &[Edge],
) -> Result<(HashSet<usize>, HashSet<usize>), Error> {
    let adjacent_edges = get_adjacent_edges(nodes, edges);

    let mut spanning_tree = HashSet::new();
    let mut cycle_edges = HashSet::new();

    let start_node = 0;
    let mut work = VecDeque::new();

    let enqueue_work_items =
        |work: &mut VecDeque<(usize, usize)>, spanning_tree: &HashSet<usize>, node_idx| {
            for edge_index in &adjacent_edges[&node_idx] {
                if spanning_tree.contains(edge_index) {
                    continue;
                }

                work.push_back((node_idx, *edge_index));
            }
        };

    enqueue_work_items(&mut work, &spanning_tree, start_node);
    let mut visited_nodes = HashSet::from([start_node]);

    while let Some((current_node_idx, edge_idx)) = work.pop_front() {
        let edge = &edges[edge_idx];
        let next_node_idx = edge.get_other_node(current_node_idx)?;
        if visited_nodes.contains(&next_node_idx) {
            cycle_edges.insert(edge_idx);
            continue;
        }

        spanning_tree.insert(edge_idx);
        visited_nodes.insert(next_node_idx);
        enqueue_work_items(&mut work, &spanning_tree, next_node_idx);
    }

    Ok((spanning_tree, cycle_edges))
}

impl Edge {
    fn get_other_node(&self, some_node: usize) -> Result<usize, Error> {
        if self.src == some_node {
            Ok(self.tgt)
        } else if self.tgt == some_node {
            Ok(self.src)
        } else {
            Err(anyhow!("edge does not connect to node"))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;

    use super::*;

    const DUMMY_CUSTOM_POSITION: custom::Position = custom::Position {
        x: 1.,
        y: 2.,
        z: 3.,
    };

    const DUMMY_CUSTOM_SETTINGS: custom::Settings = custom::Settings {
        feed_temperature: 1.,
        return_temperature: 2.,
        ground_temperature: 3.,
        time_start: 4.,
        time_end: 5.,
        time_step: 6.,
        ramp_time: 7.,
        num_iterations: 8,
        tolerance: 9.,
    };

    fn create_test_custom_topology(
        num_nodes: usize,
        num_feed_nodes: usize,
        edges: &[(usize, usize)],
        consumers: &[usize],
        sources: &[usize],
    ) -> custom::Topology {
        let nodes = (0..num_nodes)
            .map(|i| custom::Node {
                name: format!("N{}", i),
                position: DUMMY_CUSTOM_POSITION,
                feed: i < num_feed_nodes,
            })
            .collect();

        let pipes = edges
            .iter()
            .enumerate()
            .map(|(i, (src, tgt))| custom::Pipe {
                name: format!("P{}", i),
                length: 1.,
                diameter: 2.,
                transmittance: 3.,
                roughness: 4.,
                zeta: 5.,
                src: format!("N{}", src),
                tgt: format!("N{}", tgt),
            })
            .collect();

        let consumers = consumers
            .iter()
            .enumerate()
            .map(|(i, j)| custom::Consumer {
                name: format!("C{}", i),
                src: format!("N{}", j),
                tgt: format!("N{}", j + num_feed_nodes),
            })
            .collect();

        let sources = sources
            .iter()
            .enumerate()
            .map(|(i, j)| custom::Source {
                name: format!("S{}", i),
                src: format!("N{}", j),
                tgt: format!("N{}", j + num_feed_nodes),
            })
            .collect();

        custom::Topology {
            nodes,
            pipes,
            consumers,
            sources,
        }
    }

    fn create_test_custom_net(
        num_nodes: usize,
        num_feed_nodes: usize,
        edges: &[(usize, usize)],
        consumers: &[usize],
        sources: &[usize],
    ) -> custom::Network {
        let topology =
            create_test_custom_topology(num_nodes, num_feed_nodes, edges, consumers, sources);

        custom::Network {
            topology,
            scenario: custom::Scenario {
                settings: DUMMY_CUSTOM_SETTINGS,
                signals: HashMap::new(),
                inputs: HashMap::new(),
                consumer_inputs: HashMap::new(),
                source_inputs: HashMap::new(),
            },
        }
    }

    // TODO: move to some utils module
    fn set_of<T: Clone + Eq + Hash>(values: &[T]) -> HashSet<T> {
        HashSet::from_iter(values.iter().cloned())
    }

    fn create_test_nodes_and_edges(
        num_nodes: usize,
        edges: &[(usize, usize)],
    ) -> (Vec<Node>, Vec<Edge>) {
        let nodes = (0..num_nodes)
            .map(|i| Node::Node {
                name: format!("N{}", i),
            })
            .collect();
        let edges = edges
            .iter()
            .cloned()
            .map(|(src, tgt)| Edge { src, tgt })
            .collect();

        (nodes, edges)
    }

    #[test]
    fn test_extract_nodes() {
        let custom_net = create_test_custom_net(8, 4, &[(0, 1), (1, 2)], &[3, 4], &[0]);

        let nodes = extract_nodes(&custom_net).expect("could not extract nodes from custom net");

        // TODO: test for nodes (length, types, names, signals)
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
}
