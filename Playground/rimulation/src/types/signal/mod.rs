use super::formats::custom::{self, DataPoint};

use anyhow::{anyhow, Error};
use approx::AbsDiffEq;
use nalgebra::{DMatrix, DVector};

#[cfg(test)]
mod test;

#[derive(Debug, PartialEq, Clone)]
pub enum Signal {
    Const {
        value: f64,
    },
    Linear {
        h: f64,
        a: f64,
        b: f64,
        y: Vec<f64>,
        dy: Vec<f64>,
    },
    Cubic {
        h: f64,
        a: f64,
        b: f64,
        y: Vec<f64>,
        m: Vec<f64>,
    },
}

impl TryFrom<custom::Signal> for Signal {
    type Error = Error;

    fn try_from(value: custom::Signal) -> Result<Self, Self::Error> {
        match value {
            custom::Signal::Const { scale, data } => Ok(Signal::Const {
                value: scale * data,
            }),
            custom::Signal::Poly {
                degree,
                scale,
                data,
            } => {
                if ![1, 3].contains(&degree) {
                    return Err(anyhow!("polynomial of degree {} not supported", degree));
                }

                if data.len() < 2 {
                    return Err(anyhow!("data needs at least 2 points"));
                }

                let h = data[1].t - data[0].t;
                for i in 1..data.len() - 1 {
                    if data[i + 1].t - data[i].t != h {
                        return Err(anyhow!("data has inconsistent dt at index {}", i));
                    }
                }

                let a = data.first().expect("at least 2 points in data").t;
                let b = data.last().expect("at least 2 points in data").t;

                let data = data
                    .into_iter()
                    .map(|DataPoint { t: _, v }| scale * v)
                    .collect();

                match degree {
                    1 => Ok(interpolate_linear(h, a, b, data)),
                    3 => interpolate_cubic(h, a, b, data),
                    _ => unreachable!("all other degrees are not allowed"),
                }
            }
        }
    }
}

fn interpolate_linear(h: f64, a: f64, b: f64, data: Vec<f64>) -> Signal {
    let n = data.len() - 1;
    let mut y = vec![0.; n];
    let mut dy = vec![0.; n];

    for i in 0..n {
        dy[i] = (data[i + 1] - data[i]) / h;
        y[i] = data[i] - (a + i as f64 * h) * dy[i];
    }

    Signal::Linear { h, a, b, y, dy }
}

fn divided_difference(h: &f64, y_l: &f64, y: &f64, y_r: &f64) -> f64 {
    (y_r - 2. * y + y_l) / (2. * h * h)
}

fn interpolate_cubic(h: f64, a: f64, b: f64, data: Vec<f64>) -> Result<Signal, Error> {
    let n = data.len() - 1;

    // boundary conditions
    let dl = 0.;
    let dr = 0.;

    let mut d = vec![0.; n + 1];
    d[0] = 6. * ((data[1] - data[0]) / h - dl) / h;
    d[n] = 6. * (dr - (data[n] - data[n - 1]) / h) / h;
    for i in 1..n {
        d[i] = 6. * divided_difference(&h, &data[i - 1], &data[i], &data[i + 1]);
    }
    let d = DVector::from_vec(d);

    let mat = DMatrix::from_fn(n + 1, n + 1, |i, j| {
        if i == j {
            2.
        } else if (i == 0 && j == 1) || (i == n && j == n - 1) {
            1.
        } else if i.abs_diff_eq(&j, 1) {
            0.5
        } else {
            0.
        }
    });

    let m = mat
        .lu()
        .solve(&d)
        .ok_or(anyhow!("could not solve system of equations"))?
        .data
        .as_vec()
        .to_vec();

    Ok(Signal::Cubic {
        h,
        a,
        b,
        y: data,
        m,
    })
}

fn get_index(h: &f64, a: &f64, b: &f64, x: &f64) -> Result<usize, Error> {
    if x < a || x > b {
        return Err(anyhow!("{} out of bounds ([{}, {}])", x, a, b));
    }

    Ok(((x - a) / h).floor() as usize)
}

impl Signal {
    pub fn value_at(&self, x: f64) -> Result<f64, Error> {
        Ok(match self {
            Signal::Const { value } => *value,
            Signal::Linear { h, a, b, y, dy } => {
                let i = get_index(h, a, b, &x)?;
                y[i] + x * dy[i]
            }
            Signal::Cubic { h, a, b, y, m } => {
                if &x == b {
                    return Ok(y[y.len() - 1]);
                }

                let i = get_index(h, a, b, &x)? + 1;

                let dx_l = x - (a + (i - 1) as f64 * h);
                let dx_r = (a + (i) as f64 * h) - x;

                (m[i - 1] * dx_r * dx_r * dx_r
                    + m[i] * dx_l * dx_l * dx_l
                    + (6. * y[i - 1] - m[i - 1] * h * h) * dx_r
                    + (6. * y[i] - m[i] * h * h) * dx_l)
                    / (6. * h)
            }
        })
    }
}
