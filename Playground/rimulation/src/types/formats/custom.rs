use super::super::Signal;
use super::NamedComponent;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub position: Position,
    pub feed: bool,
}

impl NamedComponent for Node {
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pipe {
    pub name: String,
    pub length: f64,
    pub diameter: f64,
    pub transmittance: f64,
    pub roughness: f64,
    pub zeta: f64,
    pub src: String,
    pub tgt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Consumer {
    pub name: String,
    pub src: String,
    pub tgt: String,
}

impl NamedComponent for Consumer {
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub name: String,
    pub src: String,
    pub tgt: String,
}

impl NamedComponent for Source {
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topology {
    pub nodes: Vec<Node>,
    pub pipes: Vec<Pipe>,
    pub consumers: Vec<Consumer>,
    pub sources: Vec<Source>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub feed_temperature: f64,
    pub return_temperature: f64,
    pub ground_temperature: f64,
    pub time_start: f64,
    pub time_end: f64,
    pub time_step: f64,
    pub ramp_time: f64,
    pub num_iterations: usize,
    pub tolerance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Input {
    Consumer {
        demand: String,
        return_temperature: String,
    },
    Source {
        base_pressure: String,
        pressure_lift: String,
        temperature: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsumerSignalFactors {
    pub yearly_demand: f64,
    pub normal_return_temperature: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsumerInput {
    pub input: String,
    pub factors: ConsumerSignalFactors,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scenario {
    pub settings: Settings,
    pub signals: HashMap<String, Signal>,
    pub inputs: HashMap<String, Input>,
    pub consumer_inputs: HashMap<String, ConsumerInput>,
    pub source_inputs: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Network {
    pub topology: Topology,
    pub scenario: Scenario,
}

#[cfg(test)]
pub mod test {
    use super::*;

    use serde_json::from_reader;
    use std::fs;

    #[test]
    fn no_error_parsing_custom_format_topology() {
        let file = fs::File::open("data/custom_format/topology.json").expect("could not open file");
        let _: Topology = from_reader(file).expect("could not parse topology json");
    }

    #[test]
    fn no_error_parsing_custom_format_scenario() {
        let file = fs::File::open("data/custom_format/scenario.json").expect("could not open file");
        let _: Scenario = from_reader(file).expect("could not parse scenario json");
    }
}

#[cfg(test)]
pub mod test_util {
    use super::*;

    const DUMMY_CUSTOM_POSITION: Position = Position {
        x: 1.,
        y: 2.,
        z: 3.,
    };

    const DUMMY_CUSTOM_SETTINGS: Settings = Settings {
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

    const DUMMY_CONST_SIGNAL: Signal = Signal::Const {
        scale: 1.,
        data: 1.,
    };

    const DUMMY_CONSUMER_FACTORS: ConsumerSignalFactors = ConsumerSignalFactors {
        yearly_demand: 1.,
        normal_return_temperature: 1.,
    };

    fn create_test_topology(
        num_nodes: usize,
        num_feed_nodes: usize,
        edges: &[(usize, usize)],
        consumers: &[usize],
        sources: &[usize],
    ) -> Topology {
        let nodes = (0..num_nodes)
            .map(|i| Node {
                name: format!("N{}", i),
                position: DUMMY_CUSTOM_POSITION,
                feed: i < num_feed_nodes,
            })
            .collect();

        let pipes = edges
            .iter()
            .enumerate()
            .map(|(i, (src, tgt))| Pipe {
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
            .map(|(i, j)| Consumer {
                name: format!("C{}", i),
                src: format!("N{}", j),
                tgt: format!("N{}", j + num_feed_nodes),
            })
            .collect();

        let sources = sources
            .iter()
            .enumerate()
            .map(|(i, j)| Source {
                name: format!("S{}", i),
                src: format!("N{}", j + num_feed_nodes),
                tgt: format!("N{}", j),
            })
            .collect();

        Topology {
            nodes,
            pipes,
            consumers,
            sources,
        }
    }

    fn create_test_scenario(num_consumers: usize, num_sources: usize) -> Scenario {
        let consumer_inputs = (0..num_consumers)
            .map(|i| {
                (
                    format!("C{}", i),
                    ConsumerInput {
                        input: String::from("fake_consumer_input"),
                        factors: DUMMY_CONSUMER_FACTORS,
                    },
                )
            })
            .collect();

        let source_inputs = (0..num_sources)
            .map(|i| (format!("S{}", i), String::from("fake_source_input")))
            .collect();

        Scenario {
            settings: DUMMY_CUSTOM_SETTINGS,
            signals: [(String::from("const"), DUMMY_CONST_SIGNAL)]
                .iter()
                .cloned()
                .collect(),
            inputs: [
                (
                    String::from("fake_consumer_input"),
                    Input::Consumer {
                        demand: String::from("const"),
                        return_temperature: String::from("const"),
                    },
                ),
                (
                    String::from("fake_source_input"),
                    Input::Source {
                        base_pressure: String::from("const"),
                        pressure_lift: String::from("const"),
                        temperature: String::from("const"),
                    },
                ),
            ]
            .iter()
            .cloned()
            .collect(),
            consumer_inputs,
            source_inputs,
        }
    }

    pub fn create_test_net(
        num_nodes: usize,
        num_feed_nodes: usize,
        edges: &[(usize, usize)],
        consumers: &[usize],
        sources: &[usize],
    ) -> Network {
        let topology = create_test_topology(num_nodes, num_feed_nodes, edges, consumers, sources);
        let scenario = create_test_scenario(consumers.len(), sources.len());

        Network { topology, scenario }
    }
}
