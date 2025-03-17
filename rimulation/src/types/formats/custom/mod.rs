use super::NamedComponent;

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{collections::HashMap, fs};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PipeParameters {
    Full {
        length: f64,
        diameter: f64,
        transmittance: f64,
        roughness: f64,
        zeta: f64,
    },
    FixedVelocity {
        length: f64,
        velocity: f64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topology {
    pub nodes: Vec<Node>,
    pub pipes: Vec<Pipe>,
    pub consumers: Vec<Consumer>,
    pub sources: Vec<Source>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Settings {
    pub fn num_steps(&self) -> usize {
        ((self.time_end - self.time_start) * (24 * 60) as f64 / self.time_step).ceil() as usize
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub t: f64,
    pub v: f64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Signal {
    #[serde(rename = "const")]
    Const { scale: f64, data: f64 },
    #[serde(rename = "poly")]
    Poly {
        // TODO: make lookup cheaper (hashmap of interpolated times?)
        degree: usize,
        scale: f64,
        data: Vec<DataPoint>,
    },
    #[serde(rename = "step")]
    Step { low: f64, high: f64, time: f64 },
}

impl Signal {
    pub fn scale_data(&self, factor: f64) -> Self {
        match self {
            Signal::Const { scale, data } => Signal::Const {
                scale: *scale,
                data: data * factor,
            },
            Signal::Poly {
                degree,
                scale,
                data,
            } => Signal::Poly {
                degree: *degree,
                scale: *scale,
                data: data
                    .iter()
                    .map(|DataPoint { t, v }| DataPoint {
                        t: *t,
                        v: v * factor,
                    })
                    .collect(),
            },
            Signal::Step { low, high, time } => Signal::Step {
                low: low * factor,
                high: high * factor,
                time: *time,
            },
        }
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameters {
    pub parameters: HashMap<String, PipeParameters>,
    pub pipes: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Network {
    pub topology: Topology,
    pub scenario: Scenario,
    pub parameters: Parameters,
}

pub fn load(path: &str) -> Result<Network, Error> {
    let topology_file = fs::File::open(format!("{}/topology.json", path))
        .map_err(|err| anyhow!("could not open topology file: {}", err))?;
    let topology: Topology =
        from_reader(topology_file).map_err(|err| anyhow!("could not decode topology: {}", err))?;

    let scenario_file = fs::File::open(format!("{}/scenario.json", path))
        .map_err(|err| anyhow!("could not open scenario file: {}", err))?;
    let scenario: Scenario =
        from_reader(scenario_file).map_err(|err| anyhow!("could not decode scenario: {}", err))?;

    let parameters_file = fs::File::open(format!("{}/parameters.json", path))
        .map_err(|err| anyhow!("could not open parameters file: {}", err))?;
    let parameters: Parameters = from_reader(parameters_file)
        .map_err(|err| anyhow!("could not decode parameters: {}", err))?;

    Ok(Network {
        topology,
        scenario,
        parameters,
    })
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

    #[test]
    fn no_error_parsing_custom_format_parameters() {
        let file =
            fs::File::open("data/custom_format/parameters.json").expect("could not open file");
        let _: Parameters = from_reader(file).expect("could not parse parameters json");
    }
}

#[cfg(test)]
pub mod test_util;
