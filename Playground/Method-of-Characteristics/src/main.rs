use std::{fs, io};

use nalgebra::{DMatrix, DVector};

mod definition;

const DT: f64 = 1.;

fn main() {
    let file = fs::OpenOptions::new()
        .read(true)
        .open("net.json")
        .expect("could not open file");
    let file = io::BufReader::new(file);

    let network: definition::Network =
        serde_json::from_reader(file).expect("could not decode network");
    dbg!(&network);

    let initial_state = network.nodes.iter().map(|node| node.energy).collect();
    let initial_state = DVector::from_vec(initial_state);

    let mut state = initial_state;

    for t in 0..10 {
        println!("{}", state_string(t, &state));
        state = step(state, &network, DT);
    }
}

fn state_string(t: usize, state: &DVector<f64>) -> String {
    let mut result = format!("{:4} ", t);

    for value in state {
        result.push_str(&format!("{:.6} ", value));
    }

    result
}

fn step(state: DVector<f64>, network: &definition::Network, dt: f64) -> DVector<f64> {
    let n = network.nodes.len();

    let mut a = DMatrix::<f64>::zeros(n, n);
    let mut b = DVector::<f64>::zeros(n);

    for edge in network.edges.iter() {
        // TODO: check that index is ok

        if network.nodes[edge.target].constant {
            panic!("nodes that have constant energy should never be target of an edge")
        }

        a[(edge.target, edge.target)] += edge.mass_flux;

        if edge.length >= dt * edge.mass_flux {
            b[edge.target] +=
                state[edge.target] * edge.mass_flux * (1. - dt * edge.mass_flux / edge.length);

            b[edge.target] += state[edge.source] * dt * edge.mass_flux.powi(2) / edge.length;
        } else {
            a[(edge.target, edge.source)] = -edge.length / dt;
            b[edge.target] +=
                state[edge.source] * edge.mass_flux * (1. - edge.length / (dt * edge.mass_flux));
        }
    }

    for (index, node) in network.nodes.iter().enumerate() {
        if node.constant {
            a[(index, index)] = 1.;
            b[index] = node.energy;
        }
    }

    let a = a;
    let decomp = a.lu();
    let b = b;

    //dbg!(&decomp, &rhs);
    let new_state = decomp.solve(&b).expect("hopefully doesnt crash");
    dbg!(&new_state);

    new_state
}
