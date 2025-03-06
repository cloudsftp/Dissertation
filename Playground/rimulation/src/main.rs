use anyhow::Error;
use rimulation::{
    simulation::simulate,
    types::{
        formats::custom::load,
        network::{FullPipeParameters, Network},
    },
};

fn main() -> Result<(), Error> {
    let network = load("data/running_example")?;
    let settings = network.scenario.settings.clone();
    let network: Network<FullPipeParameters> = network.try_into()?;

    simulate(network, settings)?;

    Ok(())
}
