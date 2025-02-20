use nalgebra::{matrix, vector};

use crate::polynome::poly;

pub fn transition(x: f64, l: f64, r: f64, yl: f64, dyl: f64, yr: f64, dyr: f64) -> f64 {
    let a = matrix![
        1.,  l,  l * l,  l * l * l;
        0., 1., 2. * l, 3. * l * l;
        1.,  r,  r * r,  r * r * r;
        0., 1., 2. * r, 3. * r * r;
    ];

    let d = vector![yl, dyl, yr, dyr];

    let p = a.lu().solve(&d).expect("should be able to solve");
    poly(x, p.data.as_slice())
}

// WIP: faster
pub fn transition_fast(x: f64, l: f64, r: f64, yl: f64, dyl: f64, yr: f64, dyr: f64) -> f64 {
    let x = if x < l {
        0.
    } else if x > r {
        1.
    } else {
        (x - l) / (r - l)
    };

    let a = yl;
    let b = dyl;
    let c = -3. * yl - 2. * dyl + 3. * yr - dyr;
    let d = 2. * yl + dyl - 2. * yr + dyr;

    poly(x, &[a, b, c, d])
}

/*
#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn transition_values() {
        let n = 1000usize;
        let dx = 2.;

        let yl = 0.;
        let dyl = 0.5;
        let yr = 1.;
        let dyr = 2.;

        for i in 0..n {
            let x = -dx + i as f64 * 2. * dx / n as f64;
            let y_og = transition(x, -dx, dx, yl, dyl, yr, dyr);
            let y = transition_fast(x, -dx, dx, yl, dyl, yr, dyr);

            dbg!(i);

            assert_relative_eq!(y_og, y);
        }
    }
}
*/
