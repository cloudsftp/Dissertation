use csv::{ReaderBuilder, Writer};
use std::fs::File;

use anyhow::{anyhow, Error};
use nalgebra::DVector;

use crate::types::{
    formats::{custom::Settings, NamedComponent},
    network::Network,
};

pub fn write_temperatures<EdgeParameters>(
    network: &Network<EdgeParameters>,
    settings: &Settings,
    result: Vec<(usize, DVector<f64>)>,
    output_file_name: &str,
) -> Result<(), Error> {
    let num_nodes = network.nodes().count();

    if num_nodes == 0 {
        return Err(anyhow!("no nodes in input"));
    } else if result.len() > num_nodes {
        return Err(anyhow!("more result vectors than nodes in network"));
    }

    for (i, temperatures) in &result {
        if temperatures.len() < settings.num_steps() {
            return Err(anyhow!(
                "temperature vector for node {} has {} elements, but simulation steps {} times",
                i,
                temperatures.len(),
                settings.num_steps()
            ));
        }
    }

    let mut writer = Writer::from_writer(File::create(output_file_name)?);

    writer.write_record(
        result
            .iter()
            .map(|(i, _)| {
                network
                    .nodes()
                    .nth(*i)
                    .ok_or(anyhow!("could not get node {}", i))
                    .map(|node| node.get_name())
            })
            .collect::<Result<Vec<_>, Error>>()?,
    )?;

    for record in (0..settings.num_steps()).map(|t| {
        result
            .iter()
            .map(move |(_, temperatures)| temperatures[t])
            .collect::<Vec<_>>()
    }) {
        writer.serialize(record)?;
    }

    writer.flush()?;

    Ok(())
}

pub fn read_temperatures<EdgeParameters>(
    network: &Network<EdgeParameters>,
    settings: &Settings,
    input_file_name: &str,
) -> Result<Vec<(usize, DVector<f64>)>, Error> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(File::open(input_file_name)?);

    let headers = reader.headers()?;
    let node_indices = headers
        .iter()
        .map(|name| {
            network
                .nodes()
                .enumerate()
                .find(|(_, node)| node.get_name() == name)
                .ok_or(anyhow!(
                    "could not find a node with the name '{}' in the network",
                    name
                ))
                .map(|(i, _)| i)
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let mut data: Vec<Vec<f64>> = Vec::with_capacity(settings.num_steps());
    for values in reader.deserialize() {
        data.push(values.map_err(|err| anyhow!("could not deserialize data row: {}", err))?)
    }

    let num_measured_nodes = node_indices.len();
    if !data
        .iter()
        .all(|row_values| row_values.len() == num_measured_nodes)
    {
        return Err(anyhow!(
            "not all data rows have {} values",
            num_measured_nodes
        ));
    }

    let mut results =
        vec![(0usize, DVector::from_element(settings.num_steps(), 0.)); num_measured_nodes];
    for t in 0..settings.num_steps() {
        for (i, (_, temperatures)) in results.iter_mut().enumerate() {
            temperatures[t] = data[t][i]
        }
    }

    Ok(results)
}
