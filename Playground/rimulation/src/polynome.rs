/// Evaluates a polynomial using Horner's method.
///
/// # Arguments
/// * `x` - The value at which to evaluate the polynomial
/// * `coefficients` - Coefficients of the polynomial in ascending order of degree
///   (i.e., [a, b, c, ...] for polynomial a + b x + c x^2 + ...)
///
/// # Example
/// ```
/// use rimulation::polynome::poly;
///
/// let x = 2.0;
/// let result = poly(x, &[1.0, 2.0, 3.0]);
/// assert_eq!(result, 1. + 2. * x + 3. * x.powi(2))
/// ```
pub fn poly(x: f64, coefficients: &[f64]) -> f64 {
    let mut y = 0.;

    for f in coefficients.iter().rev() {
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
