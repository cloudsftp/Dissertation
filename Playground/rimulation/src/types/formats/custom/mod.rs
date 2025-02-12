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
pub mod test_util;
