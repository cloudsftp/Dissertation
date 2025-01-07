use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Network {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Node {
    #[serde(rename = "producer")]
    Producer {
        name: String,
        energy: f64,
        pressure: f64,
        #[serde(default = "bool::default")]
        constant: bool,
    },
    #[serde(rename = "consumer")]
    Consumer { name: String, energy: f64 },
    #[serde(rename = "join")]
    Join {
        name: String,
        energy: f64,
        demand: f64,
    },
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub source: usize,
    pub target: usize,
    pub length: f64,
    pub diameter: f64,
}
