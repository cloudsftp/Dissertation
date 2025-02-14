use super::formats::custom::{self, DataPoint};

use anyhow::{anyhow, Error};

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
                if degree < 1 || degree > 3 {
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

                Ok(match degree {
                    1 => interpolate_linear(h, a, b, data),
                    _ => todo!(),
                    _ => unreachable!("all other degrees are not allowed"),
                })
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

fn get_index(h: &f64, a: &f64, b: &f64, x: &f64) -> Result<usize, Error> {
    if x < a || x > b {
        return Err(anyhow!("{} out of bounds ([{}, {}])", x, a, b));
    }

    Ok(((x - a) / h).floor() as usize)
}

impl Signal {
    fn value_at(&self, x: f64) -> Result<f64, Error> {
        Ok(match self {
            Signal::Const { value } => *value,
            Signal::Linear { h, a, b, y, dy } => {
                let i = get_index(h, a, b, &x)?;
                y[i] + x * dy[i]
            } /*
              Signal::Poly {
                  degree,
                  scale,
                  data,
              } => {
                  let first_point = data
                      .first()
                      .ok_or(anyhow!("data vector should have at least one element"))?;
                  if t <= first_point.t {
                      return Ok(scale * first_point.v);
                  }

                  let last_point = data
                      .last()
                      .ok_or(anyhow!("data vector should have at least one element"))?;
                  if t >= last_point.t {
                      return Ok(scale * last_point.v);
                  }

                  let right_index = data
                      .iter()
                      .enumerate()
                      .find(|(_, point)| point.t >= t)
                      .map(|(i, _)| i)
                      .ok_or(anyhow!(
                          "could not find the point to the right of t = {}",
                          t
                      ))?;

                  let right = data
                      .get(right_index)
                      .expect("this index is taken directly from the indexes of the data array");
                  let left = data
                      .get(right_index - 1).expect("left_index cannot be the last index of the data array. Therefore, left_index + 1 is also valid");

                  if right.t <= left.t {
                      return Err(anyhow!("data points in wrong order"));
                  }

                  let l = right.t - left.t;
                  dbg!(left, right, l);
                  Ok(scale * (left.v * (right.t - t) + right.v * (t - left.t)) / l)
              }
               */
        })
    }
}
