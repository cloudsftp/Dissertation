use super::*;

use crate::types::formats::custom::{self, DataPoint};

#[test]
fn convert_constant_signal() {
    let custom_signal = custom::Signal::Const {
        scale: 2.,
        data: 100.,
    };

    let signal: Signal = custom_signal
        .try_into()
        .expect("could not convert custom signal");

    assert_eq!(signal, Signal::Const { value: 200. });
}

fn assert_convert_polynomial_errors(
    name: &str,
    degree: usize,
    data: Vec<DataPoint>,
    expected_msg: &str,
) {
    let custom_signal = custom::Signal::Poly {
        degree,
        scale: 1.,
        data,
    };

    let result: Result<Signal, Error> = custom_signal.try_into();

    assert!(result.is_err(), "test case '{}' did not error", name,);

    let err = result.err().expect("it is err");

    assert!(
        err.to_string().contains(expected_msg),
        "error message not as expected for test case '{}': expected {}, got {}",
        name,
        expected_msg,
        err.to_string(),
    );
}

#[test]
fn convert_polynomial_validate_data() {
    assert_convert_polynomial_errors(
        "degree 0",
        0,
        vec![],
        "polynomial of degree 0 not supported",
    );
    assert_convert_polynomial_errors(
        "degree 4",
        4,
        vec![],
        "polynomial of degree 4 not supported",
    );
    assert_convert_polynomial_errors(
        "too few points, degree 1",
        1,
        vec![DataPoint { t: 0., v: 0. }],
        "data needs at least 2 points",
    );
    assert_convert_polynomial_errors(
        "too few points, degree 3",
        3,
        vec![
            DataPoint { t: 0., v: 0. },
            DataPoint { t: 0., v: 0. },
            DataPoint { t: 0., v: 0. },
        ],
        "data needs at least 4 points",
    );
    assert_convert_polynomial_errors(
        "inconsistent dt",
        1,
        vec![
            DataPoint { t: 0., v: 0. },
            DataPoint { t: 1., v: 0. },
            DataPoint { t: 0., v: 0. },
        ],
        "data has inconsistent dt at index 1",
    );
}
