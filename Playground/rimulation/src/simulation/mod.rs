mod hydraulic;
mod matrices;

use anyhow::Error;
use matrices::Matrices;
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
        .nodes()
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

    let matrices = Matrices::try_from(&network)?;

    let e = DVector::from_vec(initial_energy_densities(&network, &settings)?);

    let matrices = Matrices::try_from(&network)?;

    let q = DVector::from_vec(
        network
            .demand_nodes
            .iter()
            .map(|node| match node {
                Node::Pressure { .. } => {
                    unreachable!("there should be no pressure node included here")
                }
                Node::Demand { demand, .. } => demand.value_at(0.), // TODO: transform to velocity
                Node::Zero { .. } => Ok(0.),
            })
            .collect::<Result<Vec<f64>, Error>>()?,
    );

    let m1 = (matrices.ar.transpose() * matrices.at.transpose())
        .lu()
        .solve(&q)
        .expect("could not solve system of equations for m1");

    dbg!(m1);

    let v = DVector::from_fn(network.num_nodes(), |i, _| todo!());

    todo!();
}
