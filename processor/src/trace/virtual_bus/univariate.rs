use alloc::vec::Vec;
use vm_core::{polynom, FieldElement};
use winter_prover::math::batch_inversion;

/// The evaluations of a univariate polynomial of degree n at 0, 1, ..., n with the evaluation at 0 omitted.
#[derive(Clone, Debug)]
pub struct UnivariatePolyEvals<E> {
    pub(crate) partial_evaluations: Vec<E>,
}

impl<E: FieldElement> UnivariatePolyEvals<E> {
    /// Gives the coefficient representation of a polynomial represented in evaluation form.
    ///
    /// Since the evaluation at 0 is omitted, we need to use the round claim to recover
    /// the evaluation at 0 using the identity p(0) + p(1) = claim.
    /// Now, we have that for any polynomial p(x) = c0 + c1 * x + ... + c_{n-1} * x^{n - 1}:
    ///
    /// 1. p(0) = c0.
    /// 2. p(x) = c0 + x * q(x) where q(x) = c1 + ... + c_{n-1} * x^{n - 2}.
    ///
    /// This means that we can compute the evaluations of q at 1, ..., n - 1 using the evaluations
    /// of p and thus reduce by 1 the size of the interpolation problem.
    /// Once the coefficient of q are recovered, the c0 coefficient is appended to these and this
    /// is precisely the coefficient representation of the original polynomial q.
    /// Note that the coefficient of the linear term is removed as this coefficient can be recovered
    /// from the remaining coefficients, again, using the round claim using the relation
    /// 2 * c0 + c1 + ... c_{n - 1} = claim.
    pub fn to_poly(&self, round_claim: E) -> UnivariatePolyCoef<E> {
        // construct the vector of interpolation points 1, ..., n
        let n_minus_1 = self.partial_evaluations.len();
        let mut points = vec![E::BaseField::ZERO; n_minus_1];

        // construct their inverses. These will be needed for computing the evaluations
        // of the q polynomial as well as for doing the interpolation on q
        points
            .iter_mut()
            .enumerate()
            .for_each(|(i, node)| *node = E::BaseField::from(i as u32 + 1));
        let points_inv = batch_inversion(&points);

        // compute the zeroth coefficient
        let c0 = round_claim - self.partial_evaluations[0];

        // compute the evaluations of q
        let mut q_evals: Vec<E> = vec![E::ZERO; n_minus_1];
        q_evals.iter_mut().zip(self.partial_evaluations.iter()).enumerate().for_each(
            |(i, (normalized, evals))| *normalized = (*evals - c0).mul_base(points_inv[i]),
        );

        // interpolate q
        let q_coefs = multiply_by_inverse_vandermonde(&q_evals, &points_inv);

        // append c0 to the coefficients of q to get the coefficients of p. The linear term
        // coefficient is removed as this can be recovered from the other coefficients using
        // the reduced claim.
        let mut coefficients = Vec::with_capacity(self.partial_evaluations.len() + 1);
        coefficients.push(c0);
        coefficients.extend_from_slice(&q_coefs[1..]);

        UnivariatePolyCoef { coefficients }
    }
}

/// The coefficients of a univariate polynomial of degree n with the linear term coefficient omitted.
#[derive(Clone, Debug)]
pub struct UnivariatePolyCoef<E: FieldElement> {
    pub(crate) coefficients: Vec<E>,
}

impl<E: FieldElement> UnivariatePolyCoef<E> {
    /// Evaluates a polynomial at a challenge point using a round claim.
    ///
    /// The round claim is used to recover the coefficient of the linear term using the relation
    /// 2 * c0 + c1 + ... c_{n - 1} = claim. Using the complete list of coefficients, the polynomial
    /// is then evaluated using Horner's method.
    pub fn evaluate_using_claim(&self, claim: &E, challenge: &E) -> E {
        // recover the coefficient of the linear term
        let c1 = *claim
            - self.coefficients.iter().fold(E::ZERO, |acc, term| acc + *term)
            - self.coefficients[0];

        // construct the full coefficient list
        let mut complete_coefficients = vec![self.coefficients[0], c1];
        complete_coefficients.extend_from_slice(&self.coefficients[1..]);

        // evaluate
        polynom::eval(&complete_coefficients, *challenge)
    }
}

/// Given a (row) vector `v`, computes the vector-matrix product `v * V^{-1}` where `V` is
/// the Vandermonde matrix over the points `1, ..., n` where `n` is the length of `v`.
/// The resulting vector will then be the coefficients of the minimal interpolating polynomial
/// through the points `(i+1, v[i])` for `i` in `0, ..., n - 1`
///
/// The naive way would be to invert the matrix `V` and then compute the vector-matrix product
/// this will cost `O(n^3)` operations and `O(n^2)` memory. We can also try Gaussian elimination
/// but this is also worst case `O(n^3)` operations and `O(n^2)` memory.
/// In the following implementation, we use the fact that the points over which we are interpolating
/// is a set of equidistant points and thus both the Vandermonde matrix and its inverse can be
/// described by sparse linear recurrence equations.
/// More specifically, we use the representation given in [1], where `V^{-1}` is represented as
/// `U * M` where:
///
/// 1. `M` is a lower triangular matrix where its entries are given by
///     M(i, j) = M(i - 1, j) - M(i - 1, j - 1) / (i - 1)
/// with boundary conditions M(i, 1) = 1 and M(i, j) = 0 when j > i.
///
/// 2. `U` is an upper triangular (involutory) matrix where its entries are given by
///     U(i, j) = U(i, j - 1) - U(i - 1, j - 1)
/// with boundary condition U(1, j) = 1 and U(i, j) = 0 when i > j.
///
/// Note that the matrix indexing in the formulas above matches the one in the reference and starts
/// from 1.
///
/// The above implies that we can do the vector-matrix multiplication in `O(n^2)` and using only
/// `O(n)` space.
///
/// [1]: https://link.springer.com/article/10.1007/s002110050360
fn multiply_by_inverse_vandermonde<E: FieldElement>(
    vector: &[E],
    nodes_inv: &[E::BaseField],
) -> Vec<E> {
    let res = multiply_by_u(vector);
    multiply_by_m(&res, nodes_inv)
}

/// Multiplies a (row) vector `v` by an upper triangular matrix `U` to compute `v * U`.
///
/// `U` is an upper triangular (involutory) matrix with its entries given by
///     U(i, j) = U(i, j - 1) - U(i - 1, j - 1)
/// with boundary condition U(1, j) = 1 and U(i, j) = 0 when i > j.
fn multiply_by_u<E: FieldElement>(vector: &[E]) -> Vec<E> {
    let n = vector.len();
    let mut previous_u_col = vec![E::BaseField::ZERO; n];
    previous_u_col[0] = E::BaseField::ONE;
    let mut current_u_col = vec![E::BaseField::ZERO; n];
    current_u_col[0] = E::BaseField::ONE;

    let mut result: Vec<E> = vec![E::ZERO; n];
    for (i, res) in result.iter_mut().enumerate() {
        *res = vector[0];

        for (j, v) in vector.iter().enumerate().take(i + 1).skip(1) {
            let u_entry: E::BaseField =
                compute_u_entry::<E>(j, &mut previous_u_col, &mut current_u_col);
            *res += v.mul_base(u_entry);
        }
        previous_u_col.clone_from(&current_u_col);
    }

    result
}

/// Multiplies a (row) vector `v` by a lower triangular matrix `M` to compute `v * M`.
///
/// `M` is a lower triangular matrix with its entries given by
///     M(i, j) = M(i - 1, j) - M(i - 1, j - 1) / (i - 1)
/// with boundary conditions M(i, 1) = 1 and M(i, j) = 0 when j > i.
fn multiply_by_m<E: FieldElement>(vector: &[E], nodes_inv: &[E::BaseField]) -> Vec<E> {
    let n = vector.len();
    let mut previous_m_col = vec![E::BaseField::ONE; n];
    let mut current_m_col = vec![E::BaseField::ZERO; n];
    current_m_col[0] = E::BaseField::ONE;

    let mut result: Vec<E> = vec![E::ZERO; n];
    result[0] = vector.iter().fold(E::ZERO, |acc, term| acc + *term);
    for (i, res) in result.iter_mut().enumerate().skip(1) {
        current_m_col = vec![E::BaseField::ZERO; n];

        for (j, v) in vector.iter().enumerate().skip(i) {
            let m_entry: E::BaseField =
                compute_m_entry::<E>(j, &mut previous_m_col, &mut current_m_col, nodes_inv[j - 1]);
            *res += v.mul_base(m_entry);
        }
        previous_m_col.clone_from(&current_m_col);
    }

    result
}

/// Returns the j-th entry of the i-th column of matrix `U` given the values of the (i - 1)-th
/// column. The i-th column is also updated with the just computed `U(i, j)` entry.
///
/// `U` is an upper triangular (involutory) matrix with its entries given by
///     U(i, j) = U(i, j - 1) - U(i - 1, j - 1)
/// with boundary condition U(1, j) = 1 and U(i, j) = 0 when i > j.
fn compute_u_entry<E: FieldElement>(
    j: usize,
    col_prev: &mut [E::BaseField],
    col_cur: &mut [E::BaseField],
) -> E::BaseField {
    let value = col_prev[j] - col_prev[j - 1];
    col_cur[j] = value;
    value
}

/// Returns the j-th entry of the i-th column of matrix `M` given the values of the (i - 1)-th
/// and the i-th columns. The i-th column is also updated with the just computed `M(i, j)` entry.
///
/// `M` is a lower triangular matrix with its entries given by
///     M(i, j) = M(i - 1, j) - M(i - 1, j - 1) / (i - 1)
/// with boundary conditions M(i, 1) = 1 and M(i, j) = 0 when j > i.
fn compute_m_entry<E: FieldElement>(
    j: usize,
    col_previous: &mut [E::BaseField],
    col_current: &mut [E::BaseField],
    node_inv: E::BaseField,
) -> E::BaseField {
    let value = col_current[j - 1] - node_inv * col_previous[j - 1];
    col_current[j] = value;
    value
}

#[test]
fn test_poly_partial() {
    use vm_core::Felt;

    use test_utils::rand;
    let degree = 1000;
    let mut points: Vec<Felt> = vec![Felt::ZERO; degree];
    points.iter_mut().enumerate().for_each(|(i, node)| *node = Felt::from(i as u32));

    let p: Vec<Felt> = rand::rand_vector(degree);
    let evals = vm_core::polynom::eval_many(&p, &points);

    let mut partial_evals = evals.clone();
    partial_evals.remove(0);

    let partial_poly = UnivariatePolyEvals {
        partial_evaluations: partial_evals,
    };
    let claim = evals[0] + evals[1];
    let poly_coeff = partial_poly.to_poly(claim);

    let r = rand::rand_vector(1);

    assert_eq!(vm_core::polynom::eval(&p, r[0]), poly_coeff.evaluate_using_claim(&claim, &r[0]))
}
