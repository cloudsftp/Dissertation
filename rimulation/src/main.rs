use anyhow::Error;
use clap::{Parser, Subcommand};
use rimulation::{
    output::{read_temperatures, write_temperatures},
    simulation::simulate_delay,
    types::{
        formats::custom::load,
        network::{FixedVelocityPipeParameters, Network},
    },
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Simulate { directory: String },
    Recover { directory: String },
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Simulate { directory } => {
            let network = load(directory)?;
            let settings = network.scenario.settings.clone();
            let network: Network<FixedVelocityPipeParameters> = network.try_into()?;

            let result = simulate_delay(&network, &settings)?;

            write_temperatures(
                &network,
                &settings,
                result,
                format!("{}/result", directory).as_str(),
            )?;
        }
        Commands::Recover { directory } => {
            let network = load(directory)?;
            let settings = network.scenario.settings.clone();
            let network: Network<FixedVelocityPipeParameters> = network.try_into()?;

            let results = read_temperatures(
                &network,
                &settings,
                format!("{}/result", directory).as_str(),
            )?;
        }
    }

    Ok(())
}
