use anyhow::Error;
use rimulation::{
    simulation::{simulate, simulate_delay},
    types::{
        formats::custom::load,
        network::{FixedVelocityPipeParameters, Network},
    },
};

fn main() -> Result<(), Error> {
    let network = load("data/fixed_velocity/single_pipe")?;
    let settings = network.scenario.settings.clone();
    let network: Network<FixedVelocityPipeParameters> = network.try_into()?;

    simulate_delay(network, settings)?;

    Ok(())
}
