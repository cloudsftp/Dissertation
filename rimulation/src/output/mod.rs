use csv::Writer;
use std::{collections::HashSet, fs::File};

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
