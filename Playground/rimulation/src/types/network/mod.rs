#[cfg(test)]
pub mod test;

use super::formats::{
    custom::{self, Input, Pipe},
    NamedComponent,
};
use super::signal::Signal;

use anyhow::{anyhow, Error};
use std::collections::{HashMap, HashSet, VecDeque};

const HOURS_PER_YEAR: f64 = 8760.;

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Pressure {
        name: String,
        pressure: Signal,
        temperature: Signal,
    },
    Demand {
        name: String,
        demand: Signal,
    },
    Zero {
        name: String,
    },
}

impl NamedComponent for Node {
    fn get_name(&self) -> String {
        match self {
            Node::Pressure {
                name,
                pressure: _,
                temperature: _,
            } => name.clone(),
            Node::Demand { name, demand: _ } => name.clone(),
            Node::Zero { name } => name.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PipeParameters {
    pub length: f64,
    pub diameter: f64,
    pub transmittance: f64,
    pub roughness: f64,
    pub zeta: f64,
}

impl From<&Pipe> for PipeParameters {
    fn from(value: &Pipe) -> Self {
        Self {
            length: value.length,
            diameter: value.diameter,
            transmittance: value.transmittance,
            roughness: value.roughness,
            zeta: value.zeta,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Edge {
    pub src: usize,
    pub tgt: usize,
    pub parameters: PipeParameters,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Network {
    pub demand_nodes: Vec<Node>,
    pub pressure_nodes: Vec<Node>,
    pub root_node_index: usize,
    pub spanning_tree_edges: Vec<Edge>,
    pub cycle_edges: Vec<Edge>,
    pub pred_nodes: HashMap<usize, usize>,
    pub edge_indices_by_connected_nodes: HashMap<(usize, usize), (usize, bool)>,
    // Future: pressure_edges
}

impl Network {
    pub fn try_from_feed(nodes: Vec<Node>, edges: Vec<Edge>) -> Result<Self, Error> {
        let (demand_nodes, pressure_nodes, edges) = split_nodes(nodes, edges);

        let (root_node_index, spanning_tree_edges, cycle_edges, pred_nodes) =
            split_edges(demand_nodes.iter().chain(pressure_nodes.iter()), edges)?;

        let edge_indices_by_connected_nodes: HashMap<(usize, usize), (usize, bool)> =
            spanning_tree_edges
                .iter()
                .enumerate()
                .map(|(i, edge)| {
                    [
                        ((edge.src, edge.tgt), (i, false)),
                        ((edge.tgt, edge.src), (i, true)),
                    ]
                    .into_iter()
                })
                .flatten()
                .collect();

        Ok(Network {
            demand_nodes,
            pressure_nodes,
            root_node_index,
            spanning_tree_edges,
            cycle_edges,
            pred_nodes,
            edge_indices_by_connected_nodes,
        })
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.demand_nodes.iter().chain(self.pressure_nodes.iter())
    }

    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.spanning_tree_edges
            .iter()
            .chain(self.cycle_edges.iter())
    }

    pub fn get_node(&self, i: usize) -> Result<&Node, Error> {
        let num_demand_nodes = self.demand_nodes.len();
        let num_pressure_nodes = self.pressure_nodes.len();
        if i < num_demand_nodes {
            Ok(&self.demand_nodes[i])
        } else if i - num_demand_nodes < num_pressure_nodes {
            Ok(&self.pressure_nodes[i - num_demand_nodes])
        } else {
            Err(anyhow!(
                "index {} out of range, only have {} nodes",
                i,
                num_demand_nodes + num_pressure_nodes,
            ))
        }
    }

    pub fn get_edge(&self, i: usize) -> Result<&Edge, Error> {
        let num_spanning_tree_edges = self.spanning_tree_edges.len();
        let num_cycle_edges = self.cycle_edges.len();
        if i < num_spanning_tree_edges {
            Ok(&self.spanning_tree_edges[i])
        } else if i - num_spanning_tree_edges < num_cycle_edges {
            Ok(&self.cycle_edges[i - num_spanning_tree_edges])
        } else {
            Err(anyhow!(
                "index {} out of range, only have {} edges",
                i,
                num_spanning_tree_edges + num_cycle_edges,
            ))
        }
    }

    pub fn num_cycles(&self) -> usize {
        self.cycle_edges.len()
    }

    pub fn num_edges(&self) -> usize {
        self.spanning_tree_edges.len() + self.cycle_edges.len()
    }

    pub fn num_nodes(&self) -> usize {
        self.demand_nodes.len() + self.pressure_nodes.len()
    }
}

impl TryFrom<custom::Network> for Network {
    type Error = Error;

    fn try_from(value: custom::Network) -> Result<Self, Self::Error> {
        let nodes = extract_nodes(&value)?;
        let edges = extract_edges(&value, &nodes)?;

        let num_sources = nodes
            .iter()
            .filter(|node| matches!(node, &Node::Pressure { .. }))
            .count();
        if num_sources == 0 {
            return Err(anyhow!("network does not have any sources"));
        } else if num_sources > 1 {
            return Err(anyhow!(
                "network with multiple sources not supported as of yet"
            ));
        }

        let (nodes, edges) = extract_feed(nodes, edges)?;
        Network::try_from_feed(nodes, edges)
    }
}

fn split_nodes(nodes: Vec<Node>, edges: Vec<Edge>) -> (Vec<Node>, Vec<Node>, Vec<Edge>) {
    let demand_indices: HashSet<usize> = nodes
        .iter()
        .enumerate()
        .filter_map(|(i, node)| match node {
            Node::Pressure { .. } => None,
            Node::Demand { .. } => Some(i),
            Node::Zero { .. } => Some(i),
        })
        .collect();

    let mut index_mapping: HashMap<usize, usize> = HashMap::new();
    let mut j = 0usize;

    let mut insert_node = |(i, node): &(usize, &Node)| {
        index_mapping.insert(*i, j);
        j += 1;
    };

    let demand_nodes = nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| demand_indices.contains(i))
        .inspect(&mut insert_node)
        .map(|(_, node)| node.clone())
        .collect();

    let pressure_nodes = nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| !demand_indices.contains(i))
        .inspect(&mut insert_node)
        .map(|(_, node)| node.clone())
        .collect();

    let edges: Vec<Edge> = edges
        .into_iter()
        .map(
            |Edge {
                 src,
                 tgt,
                 parameters,
             }| {
                Edge {
                    src: index_mapping[&src],
                    tgt: index_mapping[&tgt],
                    parameters,
                }
            },
        )
        .collect();

    (demand_nodes, pressure_nodes, edges)
}

fn split_edges<'a>(
    nodes: impl Iterator<Item = &'a Node>,
    edges: Vec<Edge>,
) -> Result<(usize, Vec<Edge>, Vec<Edge>, HashMap<usize, usize>), Error> {
    let (root_node_index, spanning_tree_edge_indices, cycle_edge_indices, pred_nodes) =
        find_spanning_tree(nodes, &edges)?;

    let spanning_tree_edges = edges
        .iter()
        .enumerate()
        .filter(move |(i, _)| spanning_tree_edge_indices.contains(i))
        .map(|(_, edge)| edge.clone())
        .collect();

    let cycle_edges = edges
        .iter()
        .enumerate()
        .filter(move |(i, _)| cycle_edge_indices.contains(i))
        .map(|(_, edge)| edge.clone())
        .collect();

    Ok((
        root_node_index,
        spanning_tree_edges,
        cycle_edges,
        pred_nodes,
    ))
}

fn node_mapping<T: NamedComponent>(
    elements: &[T],
    extract_node_name: fn(&T) -> String,
) -> HashMap<String, String> {
    elements
        .iter()
        .map(|element| (extract_node_name(element), element.get_name()))
        .collect()
}

fn extract_nodes(value: &custom::Network) -> Result<Vec<Node>, Error> {
    let consumers_by_node =
        node_mapping(&value.topology.consumers, |consumer| consumer.src.clone());
    let sources_by_node = node_mapping(&value.topology.sources, |source| source.tgt.clone());

    let get_signal = |name: &String| -> Result<custom::Signal, Error> {
        Ok(value
            .scenario
            .signals
            .get(name)
            .ok_or(anyhow!("signal with name '{}' does not exist", name))?
            .clone())
    };

    let create_consumer_node = |consumer_name: &String,
                                node_name: &String|
     -> Result<Node, Error> {
        dbg!(&value.scenario.consumer_inputs);
        let consumer_input = &value
            .scenario
            .consumer_inputs
            .get(consumer_name)
            .ok_or(anyhow!(
                "no inputs defined for consumer '{}'",
                consumer_name
            ))?;

        let demand_signal_name =
            match value
                .scenario
                .inputs
                .get(&consumer_input.input)
                .ok_or(anyhow!(
                    "input with name '{}' does not exist",
                    consumer_input.input
                ))? {
                Input::Consumer {
                    demand,
                    return_temperature: _,
                } => Ok(demand),
                _ => Err(anyhow!(
                    "input with name '{}' has the wrong type, expected to be Input::Consumer",
                    consumer_input.input
                )),
            }?;

        // TODO: why scaled by hours per year, not seconds?
        let demand = get_signal(demand_signal_name)?
            .scale_data(consumer_input.factors.yearly_demand / HOURS_PER_YEAR)
            .try_into()?;
        Ok(Node::Demand {
            name: node_name.clone(),
            demand,
        })
    };

    let create_source_node = |source_name: &String, node_name: &String| {
        let source_input_name = value
            .scenario
            .source_inputs
            .get(source_name)
            .ok_or(anyhow!("no inputs defined for source '{}'", source_name))?;
        let (pressure_signal_name, temperature_signal_name) =
            match value.scenario.inputs.get(source_input_name).ok_or(anyhow!(
                "input with name '{}' does not exist",
                source_input_name
            ))? {
                Input::Source {
                    base_pressure: _,
                    pressure_lift,
                    temperature,
                } => Ok((pressure_lift, temperature)),
                _ => Err(anyhow!(
                    "input with name '{}' has the wrong type, expected to be Input::Source",
                    source_input_name
                )),
            }?;

        let pressure = get_signal(pressure_signal_name)?.try_into()?;
        let temperature = get_signal(temperature_signal_name)?.try_into()?;

        Ok(Node::Pressure {
            name: node_name.clone(),
            pressure,
            temperature,
        })
    };

    value
        .topology
        .nodes
        .iter()
        .map(|node| {
            let node_name = &node.name;
            if let Some(consumer_name) = consumers_by_node.get(node_name) {
                create_consumer_node(consumer_name, node_name)
            } else if let Some(source_name) = sources_by_node.get(&node.name) {
                create_source_node(source_name, node_name)
            } else {
                Ok(Node::Zero {
                    name: node.name.clone(),
                })
            }
        })
        .collect()
}

fn extract_edges(value: &custom::Network, nodes: &[Node]) -> Result<Vec<Edge>, Error> {
    let node_indices: HashMap<String, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.get_name(), i))
        .collect();

    let get_node_index = |node_name| {
        node_indices
            .get(node_name)
            .ok_or(anyhow!("node '{}' does not exist", node_name))
    };

    value
        .topology
        .pipes
        .iter()
        .map(|pipe| {
            let src = *get_node_index(&pipe.src)?;
            let tgt = *get_node_index(&pipe.tgt)?;

            let parameters = PipeParameters::from(pipe);

            Ok(Edge {
                src,
                tgt,
                parameters,
            })
        })
        .collect()
}

fn get_adjacent_edges<'a>(
    nodes: impl Iterator<Item = &'a Node>,
    edges: &[Edge],
) -> HashMap<usize, Vec<usize>> {
    nodes
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
    let adjacent_edges = get_adjacent_edges(nodes.iter(), edges);

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

fn filter_network(
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    nodes_to_keep: HashSet<usize>,
    edges_to_keep: HashSet<usize>,
) -> Result<(Vec<Node>, Vec<Edge>), Error> {
    let (nodes, node_index_mapping): (Vec<Node>, HashMap<usize, usize>) = nodes
        .into_iter()
        .enumerate()
        .filter(|(i, _)| nodes_to_keep.contains(i))
        .enumerate()
        .map(|(new_index, (old_index, node))| (node, (old_index, new_index)))
        .collect();

    let get_new_node_index = |old_index: usize| -> Result<usize, Error> {
        node_index_mapping
            .get(&old_index)
            .ok_or(anyhow!("no index mapping exists for node {}", old_index))
            .copied()
    };

    let edges = edges
        .into_iter()
        .enumerate()
        .filter(|(i, _)| edges_to_keep.contains(i))
        .map(
            |(
                _,
                Edge {
                    src,
                    tgt,
                    parameters,
                },
            )|
             -> Result<Edge, Error> {
                let src = get_new_node_index(src)?;
                let tgt = get_new_node_index(tgt)?;

                Ok(Edge {
                    src,
                    tgt,
                    parameters,
                })
            },
        )
        .collect::<Result<Vec<Edge>, Error>>()?;

    Ok((nodes, edges))
}

fn extract_feed(nodes: Vec<Node>, edges: Vec<Edge>) -> Result<(Vec<Node>, Vec<Edge>), Error> {
    let start_node = nodes
        .iter()
        .enumerate()
        .find_map(|(i, node)| match node {
            Node::Pressure { .. } => Some(i),
            _ => None,
        })
        .ok_or(anyhow!("no pressure (source) node in the network"))?;

    let (nodes_to_keep, edges_to_keep) = find_feed(&nodes, &edges, start_node)?;

    filter_network(nodes, edges, nodes_to_keep, edges_to_keep)
}

fn find_spanning_tree<'a>(
    nodes: impl Iterator<Item = &'a Node>,
    edges: &[Edge],
) -> Result<(usize, HashSet<usize>, HashSet<usize>, HashMap<usize, usize>), Error> {
    let adjacent_edges = get_adjacent_edges(nodes, edges);

    let mut spanning_tree = HashSet::new();
    let mut cycle_edges = HashSet::new();
    let mut pred_nodes = HashMap::new();

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
        pred_nodes.insert(next_node_idx, current_node_idx);
        enqueue_work_items(&mut work, &spanning_tree, next_node_idx);
    }

    Ok((start_node, spanning_tree, cycle_edges, pred_nodes))
}
