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

#[cfg(test)]
mod test {
    use super::*;

    use serde_json::from_reader;
    use std::{fs, io::BufReader};

    #[test]
    fn no_error_parsing_custom_format() {
        let file = fs::File::open("data/custom_format/topology.json").expect("could not open file");
        let reader = BufReader::new(file);
        let topology: Topology = from_reader(reader).expect("could not parse topology json");
    }
}
