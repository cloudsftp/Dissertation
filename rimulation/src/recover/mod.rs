use anyhow::{anyhow, Error};
use nalgebra::DVector;

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
    let signal_length = signal.len();

    todo!()
}

fn delay_distance(
    signal_a: DVector<f64>,
    signal_b: DVector<f64>,
    offset: usize, // f64?
) -> f64 {
    // Compute distance of signals when a offset is applied
    // this should scale with the overlap of the signals
    // distance([1, 1]) == distance([1])
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::types::{formats::custom::Position, network::test::DUMMY_CONST_SIGNAL};

    use super::*;

    #[test]
    fn compute_delays_of_two_signals() {
        struct TestCase<'a> {
            name: &'a str,
            signal_a: &'a [f64],
            signal_b: &'a [f64],
            delay: usize,
        }

        let test_cases = [TestCase {
            name: "simple",
            signal_a: &[0., 0., 0.],
            signal_b: &[0., 0., 0.],
            delay: 0,
        }];

        for TestCase {
            name,
            signal_a,
            signal_b,
            delay,
        } in test_cases
        {
            let signal_a = DVector::from_row_slice(signal_a);
            let signal_b = DVector::from_row_slice(signal_b);

            let delays = compute_delays(
                signal_b,
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
                    signal_a,
                )],
            )
            .expect("error when computing delays");

            dbg!(delays);

            panic!()
        }
    }
}
