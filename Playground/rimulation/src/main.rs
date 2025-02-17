use anyhow::Error;
use rimulation::{
    simulation::simulate,
    types::{formats::custom::load, network::Network},
};

fn main() -> Result<(), Error> {
    let network = load("data/custom_format")?;
    let settings = network.scenario.settings.clone();
    let network: Network = network.try_into()?;

    simulate(network, settings)?;

    Ok(())
}
