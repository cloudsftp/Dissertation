mod hydraulic;
mod matrices;

use anyhow::Error;
use matrices::Matrices;
use nalgebra::DVector;

use crate::{
    types::{
        formats::custom::Settings,
        network::{Edge, FixedVelocityPipeParameters, Network, Node},
    },
    water,
};

fn initial_energy_densities<T>(
    network: &Network<T>,
    settings: &Settings,
) -> Result<Vec<f64>, Error> {
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

pub fn simulate<PipeParameters>(
    network: Network<PipeParameters>,
    settings: Settings,
) -> Result<(), Error>
where
    PipeParameters: std::fmt::Debug,
{
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

    let v = hydraulic::get_velocities(q, e, &matrices);

    dbg!(v);

    todo!();
}

pub fn simulate_delay(
    network: Network<FixedVelocityPipeParameters>,
    settings: Settings,
) -> Result<(), Error> {
    let t = DVector::from_iterator(
        network.num_edges(),
        network
            .edge_parameters()
            .map(|FixedVelocityPipeParameters { length, velocity }| length / velocity),
    );

    let dt = settings.time_step * 60.;
    let n =
        ((settings.time_end - settings.time_start) * (24 * 60 * 60) as f64 / dt).ceil() as usize;

    let mut result = (0..network.num_nodes())
        .map(|_| DVector::from_element(n, 0 as f64))
        .collect::<Vec<_>>();

    let paths = network.paths;

    for (i, result) in result.iter_mut().enumerate() {
        for t in 0..n {
            let paths = &paths[i];
            result[t] = todo!();
        }
    }

    dbg!(result);

    Ok(())
}
