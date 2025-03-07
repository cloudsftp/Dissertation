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
    let mut e = DVector::from_vec(initial_energy_densities(&network, &settings)?);

    let t = DVector::from_iterator(
        network.num_edges(),
        network
            .edge_parameters()
            .map(|FixedVelocityPipeParameters { length, velocity }| length / velocity),
    );

    let mut result = vec![e.clone()];

    let num_steps = (settings.time_end * 24. * 60. / settings.time_step) as i32;
    for i in 0..num_steps {
        for (j, &Edge { src, tgt }) in network.edges().enumerate() {
            let t = t[j] / settings.time_step;
            e[tgt] = t * e[src] + (1. - t) * e[tgt];
        }

        for (j, node) in network.nodes().enumerate() {
            match node {
                Node::Pressure { temperature, .. } => {
                    e[j] =
                        water::energy_density(temperature.value_at(i as f64 * settings.time_step)?)?
                }
                _ => (),
            }
        }

        result.push(e.clone())
    }

    dbg!(result);

    Ok(())
}
