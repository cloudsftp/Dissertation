mod formats;

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct DataPoint {
    t: f64,
    v: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Signal {
    Const {
        name: String,
        scale: f64,
        data: f64,
    },
    Poly {
        name: String,
        degree: usize,
        scale: f64,
        data: Vec<DataPoint>,
    },
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

fn find_spanning_tree(
    nodes: Vec<Node>,
    edges: Vec<Edge>,
) -> Result<(HashSet<usize>, HashSet<usize>), Error> {
    let pipes_by_node: HashMap<usize, Vec<usize>> = nodes
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
        .collect();

    let mut spanning_tree = HashSet::new();
    let mut cycle_edges = HashSet::new();

    let start_node = 0;
    let mut work = VecDeque::new();

    let enqueue_work_items =
        |work: &mut VecDeque<(usize, usize)>, spanning_tree: &HashSet<usize>, node_idx| {
            for edge_index in &pipes_by_node[&node_idx] {
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
    use super::*;

    fn assert_find_spanning_tree(
        name: &str,
        nodes: Vec<Node>,
        edges: Vec<Edge>,
        expected_spanning_tree: &[usize],
        expected_cycle_edges: &[usize],
    ) {
        let (spanning_tree, cycle_edges) =
            find_spanning_tree(nodes, edges).expect("could not compute spanning tree");

        assert_eq!(
            spanning_tree,
            HashSet::from_iter(expected_spanning_tree.iter().cloned()),
            "spanning tree unexpected for the test case '{}'",
            name,
        );
        assert_eq!(
            cycle_edges,
            HashSet::from_iter(expected_cycle_edges.iter().cloned()),
            "cycle edges unexpected for the test case '{}'",
            name,
        )
    }

    #[test]
    fn test_find_spanning_tree() {
        assert_find_spanning_tree(
            "single edge",
            vec![
                Node::Node {
                    name: String::from("N1"),
                },
                Node::Node {
                    name: String::from("N2"),
                },
            ],
            vec![Edge { src: 0, tgt: 1 }],
            &[0],
            &[],
        );
        assert_find_spanning_tree(
            "two edges",
            vec![
                Node::Node {
                    name: String::from("N1"),
                },
                Node::Node {
                    name: String::from("N2"),
                },
                Node::Node {
                    name: String::from("N3"),
                },
            ],
            vec![Edge { src: 0, tgt: 1 }, Edge { src: 0, tgt: 2 }],
            &[0, 1],
            &[],
        );
        assert_find_spanning_tree(
            "small cycle",
            vec![
                Node::Node {
                    name: String::from("N1"),
                },
                Node::Node {
                    name: String::from("N2"),
                },
                Node::Node {
                    name: String::from("N3"),
                },
            ],
            vec![
                Edge { src: 0, tgt: 1 },
                Edge { src: 0, tgt: 2 },
                Edge { src: 1, tgt: 2 },
            ],
            &[0, 1],
            &[2],
        );
    }
}
