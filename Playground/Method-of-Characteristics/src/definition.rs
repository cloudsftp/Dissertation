use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Network {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub name: String,
    pub energy: f64,
    #[serde(default = "bool::default")]
    pub constant: bool,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub source: usize,
    pub target: usize,
    pub length: f64,
    pub mass_flux: f64,
}
