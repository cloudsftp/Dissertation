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
    }

    let mut writer = Writer::from_writer(File::create(output_file_name)?);

    let demand_node_indices: HashSet<&usize> = result.iter().map(|(i, _)| i).collect();

    writer.write_record(
        network
            .nodes()
            .enumerate()
            .filter_map(|(i, node)| demand_node_indices.contains(&i).then_some(node.get_name())),
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
