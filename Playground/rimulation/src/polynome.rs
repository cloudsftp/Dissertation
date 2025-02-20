pub fn poly(x: f64, factors: &[f64]) -> f64 {
    let mut y = 0.;

    for f in factors.iter().rev() {
        y *= x;
        y += *f;
    }

    y
}

#[cfg(test)]
mod tests {
    use super::*;

    const X: [f64; 6] = [0., -1., 1., 10., -10., 100.];

    #[test]
    fn constant() {
        for x in X {
            assert_eq!(poly(x, &[1.]), 1.)
        }
    }

    #[test]
    fn linear() {
        for x in X {
            assert_eq!(poly(x, &[2., 1.]), x + 2.)
        }
    }

    #[test]
    fn quadratic() {
        for x in X {
            assert_eq!(poly(x, &[3., 2., 1.]), (x + 2.) * x + 3.)
        }
    }

    #[test]
    fn cubic() {
        for x in X {
            assert_eq!(poly(x, &[4., 3., 2., 1.]), ((x + 2.) * x + 3.) * x + 4.)
        }
    }
}
