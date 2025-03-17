use std::{fs, io::Write};

use approx::assert_relative_eq;

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

    let err = result.expect_err("it is err");

    assert!(
        err.to_string().contains(expected_msg),
        "error message not as expected for test case '{}': expected {}, got {}",
        name,
        expected_msg,
        err,
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
        vec![DataPoint { t: 0., v: 0. }],
        "data needs at least 2 points",
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

#[test]
fn linear_interpolation() {
    let custom_linear_signal = custom::Signal::Poly {
        degree: 1,
        scale: 1.,
        data: vec![
            DataPoint { t: 2., v: 0. },
            DataPoint { t: 3., v: 1. },
            DataPoint { t: 4., v: 0.5 },
        ],
    };

    let linear_signal: Signal = custom_linear_signal
        .try_into()
        .expect("could not convert linear signal");

    assert_eq!(
        linear_signal
            .value_at(2.)
            .expect("could not evaluate signal at 2"),
        0.,
    );
    assert_eq!(
        linear_signal
            .value_at(2.5)
            .expect("could not evaluate signal at 2.5"),
        0.5,
    );
    assert_eq!(
        linear_signal
            .value_at(2.75)
            .expect("could not evaluate signal at 2.75"),
        0.75,
    );
    assert_eq!(
        linear_signal
            .value_at(3.)
            .expect("could not evaluate signal at 3"),
        1.,
    );
    assert_eq!(
        linear_signal
            .value_at(3.5)
            .expect("could not evaluate signal at 3.5"),
        0.75,
    );
}

#[test]
fn cubic_interpolation() {
    let data = vec![
        DataPoint { t: 2., v: 0. },
        DataPoint { t: 3., v: 1. },
        DataPoint { t: 4., v: 0.5 },
        DataPoint { t: 5., v: 0. },
        DataPoint { t: 6., v: 2. },
    ];

    let custom_cubic_signal = custom::Signal::Poly {
        degree: 3,
        scale: 1.,
        data: data.clone(),
    };

    let cubic_signal: Signal = custom_cubic_signal
        .try_into()
        .expect("could not convert cubic signal");

    for DataPoint { t, v } in &data {
        let y = cubic_signal
            .value_at(*t)
            .unwrap_or_else(|_| panic!("could not evaluate signal at {}", t));

        assert_relative_eq!(y, v);
    }

    let mut file = fs::File::create("/tmp/cubic").unwrap();
    let n = 1_000;
    for i in 0..n {
        let a = data.first().expect("data has at least one element").t;
        let b = data.last().expect("data has at least one element").t;
        let t = i as f64 * (b - a) / n as f64 + a;

        let v = cubic_signal
            .value_at(t)
            .unwrap_or_else(|_| panic!("could not evaluate signal at {}", t));
        file.write(format!("{}\n", v).as_bytes())
            .expect("could not write data to temporary file");
    }
}
