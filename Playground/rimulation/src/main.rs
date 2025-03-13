use anyhow::Error;
use rimulation::{
    output::write_temperatures,
    simulation::{simulate, simulate_delay},
    types::{
        formats::custom::load,
        network::{FixedVelocityPipeParameters, Network},
    },
};

fn main() -> Result<(), Error> {
    let network = load("data/fixed_velocity/triangle")?;
    let settings = network.scenario.settings.clone();
    let network: Network<FixedVelocityPipeParameters> = network.try_into()?;

    let result = simulate_delay(&network, &settings)?;

    write_temperatures(&network, &settings, result, "/tmp/output")?;

    Ok(())
}
