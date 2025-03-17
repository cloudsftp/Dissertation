
use crate::polynome::poly;

pub fn transition_cubic(
    x: f64,
    left: f64,
    right: f64,
    p_0: f64,
    m_0: f64,
    p_1: f64,
    m_1: f64,
) -> f64 {
    let width = right - left;

    let t = ((x - left) / width).clamp(0., 1.);

    let m_0 = m_0 * width;
    let m_1 = m_1 * width;

    let cubic_term = 2. * p_0 + m_0 - 2. * p_1 + m_1;
    let quadratic_term = -3. * p_0 - 2. * m_0 + 3. * p_1 - m_1;

    poly(t, &[p_0, m_0, quadratic_term, cubic_term])
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;

    use super::*;

    #[test]
    fn generate_transition_data() {
        let left = -5.;
        let right = 5.;

        let (p_0, m_0) = (0., 0.2);
        let (p_1, m_1) = (1., 0.3);

        let mut file = fs::File::create("/tmp/transition_data").expect("could not open file");

        let n = 1000usize;

        for i in 0..n {
            let x = left + i as f64 * (right - left) / n as f64;
            let y = transition_cubic(x, left, right, p_0, m_0, p_1, m_1);

            file.write(format!("{} {}\n", x, y).as_bytes())
                .expect("could not write to file");
        }
    }

    #[test]
    fn cubic_boundary_values() {
        let left = -5.;
        let right = 5.;

        let (p_0, m_0) = (-1., 0.2);
        let (p_1, m_1) = (5., 0.3);

        assert_eq!(transition_cubic(-5., left, right, p_0, m_0, p_1, m_1), -1.);
        assert_eq!(transition_cubic(5., left, right, p_0, m_0, p_1, m_1), 5.);
    }

    #[test]
    fn cubic_boundary_slopes() {
        let left = -5.;
        let right = 5.;

        let (p_0, m_0) = (-1., 0.2);
        let (p_1, m_1) = (5., 0.3);

        let assert_slope = |x, expected: f64| {
            let mut dx = 1e-6;
            if x == right {
                dx = -dx;
            }

            let fx = transition_cubic(x, left, right, p_0, m_0, p_1, m_1);
            let fdx = transition_cubic(x + dx, left, right, p_0, m_0, p_1, m_1);

            let slope = (fdx - fx) / dx;
            let difference = (slope - expected).abs();
            assert!(
                difference < dx.abs(),
                "expected difference of slopes to be smaller than {}: was {} and expected {}",
                dx,
                slope,
                expected,
            );
        };

        assert_slope(-5., 0.2);
        assert_slope(5., 0.3);
    }
}
