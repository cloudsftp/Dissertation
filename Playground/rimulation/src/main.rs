use std::{fs, io::Write};

use anyhow::Error;
use rimulation::{
    simulation::simulate,
    transition::{transition, transition_fast},
    types::{formats::custom::load, network::Network},
};

fn sim() -> Result<(), Error> {
    let network = load("data/running_example")?;
    let settings = network.scenario.settings.clone();
    let network: Network = network.try_into()?;

    simulate(network, settings)?;

    Ok(())
}

fn trans() -> Result<(), Error> {
    let mut file = fs::File::create("/tmp/transition_data")?;

    let n = 1000usize;
    let dx = 2.;

    let yl = 0.;
    let dyl = 0.;
    let yr = 1.;
    let dyr = 0.5;

    for i in 0..n {
        let x = -dx + i as f64 * 2. * dx / n as f64;
        let y = transition(x, -dx, dx, yl, dyl, yr, dyr);
        let y_fast = transition_fast(x, -dx, dx, yl, dyl, yr, dyr);

        file.write(format!("{} {} {}\n", x, y, y_fast).as_bytes())?;
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    //sim()?;
    trans()?;

    Ok(())
}
