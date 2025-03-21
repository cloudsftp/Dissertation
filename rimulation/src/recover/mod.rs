use std::{f64::INFINITY, usize::MAX};

use anyhow::{anyhow, Error};
use nalgebra::{DVector, Normed};

use crate::types::{
    formats::custom::Settings,
    network::{FixedVelocityPipeParameters, Network, Node},
};

fn filter_signals(
    signals: &Vec<(Node, DVector<f64>)>,
    pred: fn(&Node) -> bool,
) -> Vec<(Node, DVector<f64>)> {
    signals
        .iter()
        .filter(|(node, _)| pred(node))
        .cloned()
        .collect()
}

pub fn recover(
    network: &Network<FixedVelocityPipeParameters>,
    settings: &Settings,
    signals: Vec<(usize, DVector<f64>)>,
) -> Result<(), Error> {
    let signals = signals
        .into_iter()
        .map(|(i, signal)| -> Result<(Node, DVector<f64>), Error> {
            network
                .nodes()
                .nth(i)
                .ok_or(anyhow!("could not get node {}", i))
                .map(|node| (node.clone(), signal))
        })
        .collect::<Result<_, Error>>()?;

    let source_signals = filter_signals(&signals, |node| matches!(node, Node::Pressure { .. }));
    let consumer_signals = filter_signals(&signals, |node| matches!(node, Node::Pressure { .. }));

    for (node, signal) in consumer_signals {
        compute_delays(signal, &source_signals)?;
    }

    Ok(())
}

fn compute_delays(
    signal: DVector<f64>,
    source_signals: &Vec<(Node, DVector<f64>)>,
) -> Result<Vec<(usize, usize)>, Error> {
    let n = signal.len();

    if source_signals.len() != 1 {
        return Err(anyhow!("multiple sources not supported yet"));
    }

    let (source_id, source_signal) = source_signals[0].clone();

    let mut min_dist = INFINITY;
    let mut opt_offset = MAX;
    for offset in 0..n - 2 {
        let dist = delay_distance(&signal, &source_signal, offset);

        dbg!(offset, dist);

        if dist < min_dist {
            min_dist = dist;
            opt_offset = offset;
        }
    }

    Ok(vec![(0, opt_offset)])
}

fn delay_distance(
    signal_a: &DVector<f64>,
    signal_b: &DVector<f64>,
    offset: usize, // f64?
) -> f64 {
    let n = signal_a.len();
    let signal_a = signal_a.rows(0, n - offset);
    let signal_b = signal_b.rows(offset, n - offset);

    (signal_a - signal_b).norm()
}

#[cfg(test)]
mod tests {
    use approx::relative_eq;

    use crate::types::{formats::custom::Position, network::test::DUMMY_CONST_SIGNAL};

    use super::*;

    #[test]
    fn compute_distance_of_signals() {
        struct TestCase<'a> {
            name: &'a str,
            signal_a: &'a [f64],
            signal_b: &'a [f64],
            offset: usize,
            distance: f64,
        }

        let test_cases = [
            TestCase {
                name: "no offset",
                signal_a: &[60., 60., 120., 120., 120., 120.],
                signal_b: &[60., 60., 60., 120., 120., 120.],
                offset: 0,
                distance: 60.,
            },
            TestCase {
                name: "perfect offset",
                signal_a: &[60., 60., 120., 120., 120., 120.],
                signal_b: &[60., 60., 60., 120., 120., 120.],
                offset: 1,
                distance: 0.,
            },
        ];

        for TestCase {
            name,
            signal_a,
            signal_b,
            offset,
            distance,
        } in test_cases
        {
            let signal_a = DVector::from_column_slice(signal_a);
            let signal_b = DVector::from_column_slice(signal_b);

            let measured = delay_distance(&signal_a, &signal_b, offset);

            if !relative_eq!(measured, distance) {
                panic!(
                    "distance not as expected {} != {} in test case '{}'",
                    measured, distance, name
                )
            }
        }
    }

    #[test]
    fn compute_delays_of_signals() {
        struct TestCase<'a> {
            name: &'a str,
            signal_a: &'a [f64],
            signal_b: &'a [f64],
            expected_delay: usize,
        }

        let test_cases = [
            TestCase {
                name: "step function, delay exactly one",
                signal_a: &[60., 60., 120., 120., 120., 120.],
                signal_b: &[60., 60., 60., 120., 120., 120.],
                expected_delay: 1,
            },
            TestCase {
                name: "step function, delay exactly one and one half",
                signal_a: &[60., 60., 120., 120., 120., 120.],
                signal_b: &[60., 60., 60., 90., 120., 120.],
                expected_delay: 1,
            },
            TestCase {
                name: "step function, dealy bigger than one and one half",
                signal_a: &[60., 60., 120., 120., 120., 120.],
                signal_b: &[60., 60., 60., 80., 120., 120.],
                expected_delay: 2,
            },
            TestCase {
                name: "step function, dealy smaller than one and one half",
                signal_a: &[60., 60., 120., 120., 120., 120.],
                signal_b: &[60., 60., 60., 100., 120., 120.],
                expected_delay: 1,
            },
        ];

        for TestCase {
            name,
            signal_a,
            signal_b,
            expected_delay,
        } in test_cases
        {
            let signal_a = DVector::from_row_slice(signal_a);
            let signal_b = DVector::from_row_slice(signal_b);

            let delays = compute_delays(
                signal_a,
                &vec![(
                    Node::Pressure {
                        name: String::from("Source"),
                        pressure: DUMMY_CONST_SIGNAL,
                        temperature: DUMMY_CONST_SIGNAL,
                        position: Position {
                            x: 0.,
                            y: 0.,
                            z: 0.,
                        },
                    },
                    signal_b,
                )],
            )
            .expect("error when computing delays");

            assert_eq!(vec![(0, expected_delay)], delays)
        }
    }
}
