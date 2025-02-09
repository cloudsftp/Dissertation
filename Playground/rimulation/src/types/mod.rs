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
    nodes: &[Node],
    edges: &[Edge],
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
        num_nodes: usize,
        edges: &[(usize, usize)],
        expected_spanning_tree: &[usize],
    ) {
        let nodes = (0..num_nodes)
            .map(|i| Node::Node {
                name: format!("N{}", i),
            })
            .collect::<Vec<_>>();
        let edges = edges
            .iter()
            .cloned()
            .map(|(src, tgt)| Edge { src, tgt })
            .collect::<Vec<_>>();

        let (spanning_tree, cycle_edges) =
            find_spanning_tree(&nodes, &edges).expect("could not compute spanning tree");

        let expected_spanning_tree = HashSet::from_iter(expected_spanning_tree.iter().cloned());
        let expected_cycle_edges = HashSet::from_iter(0..edges.len())
            .difference(&expected_spanning_tree)
            .cloned()
            .collect::<HashSet<_>>();
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
