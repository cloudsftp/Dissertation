mod hydraulic;
mod matrices;

use anyhow::{anyhow, Error};
use matrices::Matrices;
use nalgebra::DVector;

use crate::{
    types::{
        formats::custom::Settings,
        network::{FixedVelocityPipeParameters, Network, Node},
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
    network: &Network<FixedVelocityPipeParameters>,
    settings: &Settings,
) -> Result<Vec<DVector<f64>>, Error> {
    let dealys = DVector::from_iterator(
        network.num_edges(),
        network
            .edge_parameters()
            .map(|FixedVelocityPipeParameters { length, velocity }| length / velocity),
    );

    let dt = settings.time_step * 60.;
    let n = settings.num_steps();

    let mut result = (0..network.num_nodes())
        .map(|_| DVector::from_element(n, 0 as f64))
        .collect::<Vec<_>>();

    for (i, result) in result.iter_mut().enumerate() {
        let path_delays: Vec<_> = network.paths[i]
            .iter()
            .filter(|(_, path)| {
                path.iter().all(|(edge_index, correct_direction)| {
                    if *correct_direction {
                        network.edge_parameters[*edge_index].velocity > 0.
                    } else {
                        network.edge_parameters[*edge_index].velocity < 0.
                    }
                })
            })
            .map(|(source_index, path)| {
                let mut delay = 0.;

                for (edge_index, _) in path {
                    delay += dealys[*edge_index];
                }

                (source_index, delay)
            })
            .collect();

        for t in 0..n {
            let mut value = 0.;

            for (source_index, delay) in &path_delays {
                let t = t as f64 * settings.time_step - delay;

                let source = &network.pressure_nodes[*source_index - network.demand_nodes.len()];
                if let Node::Pressure { temperature, .. } = source {
                    value += temperature.value_at(t)?;
                } else {
                    return Err(anyhow!("not a source node"));
                }
            }

            result[t] = value;
        }
    }

    Ok(result)
}
