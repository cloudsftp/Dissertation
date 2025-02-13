use super::formats::custom;

use anyhow::{anyhow, Error};

#[cfg(test)]
mod test;

#[derive(Debug, PartialEq, Clone)]
pub enum Signal {
    Const { value: f64 },
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

                let min_points = degree + 1;
                if data.len() < min_points {
                    return Err(anyhow!("data needs at least {} points", min_points));
                }

                let dt = data[1].t - data[0].t;
                for i in 1..data.len() - 1 {
                    if data[i + 1].t - data[i].t != dt {
                        return Err(anyhow!("data has inconsistent dt at index {}", i));
                    }
                }

                todo!()
            }
        }
    }
}

impl Signal {
    fn value_at(&self, t: f64) -> f64 {
        match self {
            Signal::Const { value } => *value,
            /*
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
        }
    }
}
