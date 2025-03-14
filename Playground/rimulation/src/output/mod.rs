use std::{
    collections::HashSet,
    fmt::format,
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::{anyhow, Error};
use nalgebra::DVector;

use crate::types::{
    formats::{custom::Settings, NamedComponent},
    network::Network,
};

const CHARACTERS_PER_VALUE: usize = 12;
const CHARACTERS_PER_TIME: usize = 10;

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

    let mut writer = BufWriter::new(File::create(output_file_name)?);

    let demand_node_indices: HashSet<&usize> = result.iter().map(|(i, _)| i).collect();

    let header = network
        .nodes()
        .enumerate()
        .filter_map(|(i, node)| demand_node_indices.contains(&i).then(|| node.get_name()))
        .fold(String::new(), |mut acc, name| {
            if acc.is_empty() {
                for _ in 0..(CHARACTERS_PER_TIME - 1) {
                    acc.push(' ');
                }
                acc.push('t');
            }
            for _ in 0..(CHARACTERS_PER_VALUE - name.len() + 1) {
                acc.push(' ');
            }
            acc.push_str(&name);
            acc
        });

    writer.write(header.as_bytes())?;
    writer.write("\n\n".as_bytes())?;

    for t in 0..settings.num_steps() {
        let values = result.iter().map(|(_, temperatures)| temperatures[t]).fold(
            String::new(),
            |mut acc, value| {
                let value = format!("{:.5}", value);
                if acc.is_empty() {
                    let time = (t as f64 * settings.time_step).to_string();
                    for _ in 0..(CHARACTERS_PER_TIME - time.len()) {
                        acc.push(' ');
                    }
                    acc.push_str(time.as_str());
                }
                for _ in 0..(CHARACTERS_PER_VALUE - value.len() + 1) {
                    acc.push(' ');
                }
                acc.push_str(&value);
                acc
            },
        );

        writer.write(values.as_bytes())?;
        writer.write("\n".as_bytes())?;
    }

    Ok(())
}
