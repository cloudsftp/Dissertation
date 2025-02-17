use anyhow::{anyhow, Error};
use nalgebra::ComplexField;

const T2: f64 = 59.2453;
const T2_2: f64 = 2. * T2;
const T2_4: f64 = 4. * T2;
const T1: f64 = 220.536;
const T1_SQRED: f64 = T1 * T1;
const T0: f64 = 1.93729;

/// Computes the temperature T [°C] based on the energy density e [GJ/m^3]
pub fn temperature(e: f64) -> f64 {
    (T2 * e + T1) * e + T0
}

/// Computes the energy density e [GJ/m^3] based on the temperature T [°C]
/// Returns an error if t < -203.2947
pub fn energy_density(t: f64) -> Result<f64, Error> {
    // solve t = T2 e^2 + T1 e + T0 with quadratic formula
    let d = T1_SQRED - T2_4 * (T0 - t);
    if d < 0. {
        return Err(anyhow!(
            "temperature {} not allowed: could not solve quadratic formula",
            t
        ));
    }
    Ok((-T1 + d.sqrt()) / T2_2)
}
