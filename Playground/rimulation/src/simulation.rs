use anyhow::Error;
use nalgebra::DVector;

use crate::{
    types::{
        formats::custom::Settings,
        network::{Network, Node},
    },
    water,
};

fn initial_energy_densities(network: &Network, settings: &Settings) -> Result<Vec<f64>, Error> {
    network
        .nodes
        .iter()
        .map(|node| -> Result<f64, anyhow::Error> {
            water::energy_density(match node {
                Node::Pressure { temperature, .. } => temperature.value_at(0.)?,
                _ => settings.feed_temperature,
            })
        })
        .collect()
}

pub fn simulate(network: Network, settings: Settings) -> Result<(), Error> {
    dbg!(&network);

    let e = DVector::from_vec(initial_energy_densities(&network, &settings)?);

    dbg!(&e);

    let v = DVector::from_fn(network.nodes.len(), |i, _| todo!());

    todo!();
}
