use anyhow::Error;
use nalgebra::{DMatrix, DVector};

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

    let num_edges = network.edges.len();

    let e = DVector::from_vec(initial_energy_densities(&network, &settings)?);

    let ar_T = DMatrix::from_row_iterator(
        network.num_demand,
        num_edges,
        (0..network.num_demand)
            .map(|i| {
                network.edges.iter().map(move |edge| {
                    if edge.src == i {
                        -1.
                    } else if edge.tgt == i {
                        1.
                    } else {
                        0.
                    }
                })
            })
            .flatten(),
    );

    let at_T = DMatrix::from_row_iterator(
        num_edges,
        network.num_demand,
        (0..num_edges)
            .map(|i| (0..network.num_demand).map(move |j| if i == j { 1. } else { 0. }))
            .flatten(),
    );

    let q = DVector::from_vec(
        network
            .nodes
            .iter()
            .take(network.num_demand)
            .map(|node| match node {
                Node::Pressure { .. } => {
                    unreachable!("there should be no pressure node included here")
                }
                Node::Demand { demand, .. } => demand.value_at(0.), // TODO: transform to velocity
                Node::Zero { .. } => Ok(0.),
            })
            .collect::<Result<Vec<f64>, Error>>()?,
    );

    dbg!(&ar_T, &at_T, &q);

    let m1 = (ar_T * at_T)
        .lu()
        .solve(&q)
        .expect("could not solve system of equations for m1");

    dbg!(m1);

    let v = DVector::from_fn(network.nodes.len(), |i, _| todo!());

    todo!();
}
