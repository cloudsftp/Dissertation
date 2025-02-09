use super::super::Signal;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Position {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    name: String,
    position: Position,
    feed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pipe {
    name: String,
    length: f64,
    diameter: f64,
    transmittance: f64,
    roughness: f64,
    zeta: f64,
    src: String,
    tgt: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Consumer {
    name: String,
    src: String,
    tgt: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Source {
    name: String,
    src: String,
    tgt: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Topology {
    nodes: Vec<Node>,
    pipes: Vec<Pipe>,
    consumers: Vec<Consumer>,
    sources: Vec<Source>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    feed_temperature: f64,
    return_temperature: f64,
    ground_temperature: f64,
    time_start: f64,
    time_end: f64,
    time_step: f64,
    ramp_time: f64,
    num_iterations: usize,
    tolerance: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Input {
    Consumer {
        name: String,
        demand: String,
        return_temperature: String,
    },
    Source {
        name: String,
        base_pressure: String,
        pressure_lift: String,
        temperature: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct ConsumerSignalFactors {
    yearly_demand: f64,
    normal_return_temperature: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConsumerInput {
    input: String,
    factors: ConsumerSignalFactors,
}

#[derive(Debug, Serialize, Deserialize)]
struct Scenario {
    settings: Settings,
    signals: Vec<Signal>,
    inputs: Vec<Input>,
    consumer_inputs: HashMap<String, ConsumerInput>,
    source_inputs: HashMap<String, String>,
}

#[cfg(test)]
mod test {
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
