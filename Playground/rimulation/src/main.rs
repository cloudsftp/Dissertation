use anyhow::Error;
use rimulation::{
    simulation::simulate,
    transition::transition_cubic,
    types::{formats::custom::load, network::Network},
};

fn main() -> Result<(), Error> {
    let network = load("data/running_example")?;
    let settings = network.scenario.settings.clone();
    let network: Network = network.try_into()?;

    simulate(network, settings)?;

    Ok(())
}
