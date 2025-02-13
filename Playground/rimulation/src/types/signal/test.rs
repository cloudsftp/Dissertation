use super::*;

use crate::types::formats::custom;

#[test]
fn convert_constant_signal() {
    let custom_signal = custom::Signal::Const {
        scale: 2.,
        data: 100.,
    };

    let signal: Signal = custom_signal
        .try_into()
        .expect("could not convert custom signal");

    assert_eq!(signal, Signal::Const { value: 200. })
}
