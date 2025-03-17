mod hydraulic;
mod matrices;

use std::collections::HashMap;

use anyhow::{anyhow, Error};
use matrices::Matrices;
use nalgebra::{DMatrix, DVector};

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
) -> Result<Vec<(usize, DVector<f64>)>, Error> {
    let delays = DVector::from_iterator(
        network.num_edges(),
        network
            .edge_parameters()
            .map(|FixedVelocityPipeParameters { length, velocity }| length / velocity),
    );

    let dt = settings.time_step * 60.;
    let n = settings.num_steps();

    let mut result = network
        .demand_nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| matches!(node, Node::Demand { .. }))
        .map(|(i, _)| (i, DVector::from_element(n, 0 as f64)))
        .collect::<Vec<_>>();

    for (i, result) in result.iter_mut() {
        for t in 0..n {
            let visited_counter = DVector::from_element(network.num_nodes(), 0);

            result[t] = compute_temperature_rec(
                network,
                settings,
                &delays,
                *i,
                t as f64 * settings.time_step,
                visited_counter,
            )?;
        }
    }

    Ok(result)
}

const VISITED_COUNT_THRESHOLD: usize = 3;

fn compute_temperature_rec(
    network: &Network<FixedVelocityPipeParameters>,
    settings: &Settings,
    delays: &DVector<f64>,
    current_node_index: usize,
    time: f64,
    mut visited_counter: DVector<usize>,
) -> Result<f64, Error> {
    if let Node::Pressure { temperature, .. } = network.get_node(current_node_index)? {
        return temperature.value_at(time);
    }

    let count = visited_counter.get_mut(current_node_index).ok_or(anyhow!(
        "could not get visited count for node {}",
        current_node_index
    ))?;
    if *count > VISITED_COUNT_THRESHOLD {
        return Err(anyhow!(
            "simulation visited node {} more than {} times",
            current_node_index,
            VISITED_COUNT_THRESHOLD
        ));
    }
    *count += 1;

    let calls = network
        .adjacent_edges
        .get(&current_node_index)
        .ok_or(anyhow!(
            "could not get adjacent edges to node {}",
            current_node_index
        ))?
        .iter()
        .map(|edge_index| -> Result<_, Error> {
            let next_node_index = network
                .get_edge(*edge_index)?
                .get_other_node(current_node_index)?;

            let (_, reverse) = *network
                .edge_indices_by_connected_nodes
                .get(&(next_node_index, current_node_index))
                .ok_or(anyhow!(
                    "no edge connecting nodes {} and {}",
                    current_node_index,
                    next_node_index
                ))?;

            let FixedVelocityPipeParameters { velocity, .. } =
                network.get_edge_parameters(*edge_index)?;

            Ok((*edge_index, next_node_index, *velocity, reverse))
        })
        .collect::<Result<Vec<_>, Error>>()?
        .iter()
        .filter_map(|(edge_index, next_node_index, velocity, reverse)| {
            ((*velocity < 0.) == *reverse).then_some((
                *next_node_index,
                delays[*edge_index],
                velocity.abs(),
            ))
        })
        .collect::<Vec<_>>();

    let total_weight: f64 = calls.iter().map(|(_, _, weight)| weight).sum();

    let mut temperature = 0.;

    for (next_node_index, time_delay, weight) in calls.into_iter() {
        temperature += weight / total_weight * {
            compute_temperature_rec(
                network,
                settings,
                delays,
                next_node_index,
                time - time_delay,
                visited_counter.clone(),
            )?
        };
    }

    Ok(temperature)
}
