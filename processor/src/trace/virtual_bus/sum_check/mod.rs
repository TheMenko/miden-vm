use self::domain::EvaluationDomain;
use alloc::vec::Vec;
use vm_core::FieldElement;

mod domain;

mod prover;
pub use prover::SumCheckProver;
mod verifier;
pub use verifier::SumCheckVerifier;

/// A sum-check round proof.
///
/// This represents the polynomial sent by the Prover during one of the rounds of the sum-check
/// protocol. The polynomial is in evaluation form and excludes the zero-th coefficient as
/// the Verifier can recover it from the first coefficient and the current reduced claim.
#[derive(Debug, Clone)]
pub struct RoundProof<E> {
    pub poly_evals: Vec<E>,
}

impl<E: FieldElement> RoundProof<E> {
    /// Completes the evaluations of the round polynomial by computing the zero-th coefficient
    /// using the round claim.
    pub fn to_evals(&self, claim: E) -> Vec<E> {
        let mut result = vec![];

        // s(0) + s(1) = claim
        let c0 = claim - self.poly_evals[0];

        result.push(c0);
        result.extend_from_slice(&self.poly_evals);
        result
    }
}

/// A sum-check proof.
///
/// Composed of the round proofs i.e., the polynomials sent by the Prover at each round as well as
/// the (claimed) openings of the multi-linear oracles at the evaluation point given by the round
/// challenges.
/// Openings is an [Option] as there are situations where we would like to run the sum-check
/// protocol for a certain number of rounds that is less than the number of variables of the
/// multi-linears. This is the case for example when we have a merged polynomial.
#[derive(Debug, Clone)]
pub struct Proof<E> {
    pub openings: Option<Vec<E>>,
    pub round_proofs: Vec<RoundProof<E>>,
}

/// Contains the round challenges sent by the Verifier up to some round as well as the current
/// reduced claim.
#[derive(Debug)]
pub struct RoundClaim<E: FieldElement> {
    pub eval_point: Vec<E>,
    pub claim: E,
}

/// Reduces an old claim to a new claim using the round challenge.
pub fn reduce_claim<E: FieldElement>(
    domain: &EvaluationDomain<E>,
    current_poly: &RoundProof<E>,
    current_round_claim: RoundClaim<E>,
    round_challenge: E,
) -> RoundClaim<E> {
    // construct the round polynomial using the current claim
    let poly_evals = current_poly.to_evals(current_round_claim.claim);
    // evaluate the round polynomial at the round challenge to obtain the new claim
    let new_claim = domain.evaluate(&poly_evals, round_challenge);

    // update the evaluation point using the round challenge
    let mut new_partial_eval_point = current_round_claim.eval_point;
    new_partial_eval_point.push(round_challenge);

    RoundClaim {
        eval_point: new_partial_eval_point,
        claim: new_claim,
    }
}

/// Represents an opening claim at an evaluation point against a batch of oracles.
///
/// After verifying [Proof], the verifier is left with a final question being whether a number
/// of oracles open to some value at some given point. This question is answered either using
/// further interaction with the Prover or using a polynomial commitment opening proof in
/// the compiled protocol.
#[derive(Clone, Debug)]
pub struct FinalOpeningClaim<E: FieldElement> {
    pub evaluation_point: Vec<E>,
    pub openings: Vec<E>,
    // TODO: add a Vec<Oracles> to give more information on which main trace columns we would like
    // to open.
}

#[cfg(test)]
mod test {
    use super::{
        domain::EvaluationDomain,
        prover::SumCheckProver,
        verifier::{FinalQueryBuilder, SumCheckVerifier},
    };
    use crate::trace::virtual_bus::multilinear::{CompositionPolynomial, MultiLinear};
    use alloc::vec::Vec;
    use test_utils::rand::{rand_array, rand_value, rand_vector};
    use vm_core::{crypto::random::RpoRandomCoin, Felt, FieldElement, Word, ONE, ZERO};

    #[test]
    fn test_evaluation_domain() {
        let max_degree = 5;
        let eval_domain = EvaluationDomain::<Felt>::new(max_degree);

        let r = rand_value();
        let coefficients: [Felt; 6] = rand_array();

        let evaluations: Vec<Felt> = (0..=max_degree)
            .into_iter()
            .map(|x| eval(&coefficients, Felt::from(x as u8)))
            .collect();

        assert_eq!(coefficients.len(), evaluations.len());

        let result = eval_domain.evaluate(&evaluations, r);
        let expected = eval(&coefficients, r);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_sum_check_sum() {
        let num_variables = 14;
        let values = rand_vector(1 << num_variables);
        let claim = values.iter().fold(ZERO, |acc, &x| x + acc);

        let ml = MultiLinear::new(values.to_vec()).expect("should not fail");
        let mut mls = vec![ml];
        let virtual_poly = ProjectionComposition::new(0);

        // Prover
        let prover = SumCheckProver::new(virtual_poly);
        let mut coin = RpoRandomCoin::new(Word::default());
        let proof = prover.prove(claim, &mut mls, num_variables, &mut coin).unwrap();

        // Verifier
        let plain_query_builder = PlainQueryBuilder;
        let verifier = SumCheckVerifier::new(virtual_poly, plain_query_builder);
        let mut coin = RpoRandomCoin::new(Word::default());
        let result = verifier.verify(claim, proof, &mut coin);

        assert!(result.is_ok())
    }

    #[test]
    fn test_sum_check_product() {
        let num_variables = 14;
        let values_0 = rand_vector(1 << num_variables);
        let values_1 = rand_vector(1 << num_variables);
        let claim = values_0.iter().zip(values_1.iter()).fold(ZERO, |acc, (x, y)| *x * *y + acc);

        let ml_0 = MultiLinear::new(values_0.to_vec()).expect("should not fail");
        let ml_1 = MultiLinear::new(values_1.to_vec()).expect("should not fail");
        let mut mls = vec![ml_0, ml_1];
        let virtual_poly = ProductComposition;

        // Prover
        let prover = SumCheckProver::new(virtual_poly);
        let mut coin = RpoRandomCoin::new(Word::default());
        let proof = prover.prove(claim, &mut mls, num_variables, &mut coin).unwrap();

        // Verifier
        let plain_query_builder = PlainQueryBuilder;
        let verifier = SumCheckVerifier::new(virtual_poly, plain_query_builder);
        let mut coin = RpoRandomCoin::new(Word::default());
        let result = verifier.verify(claim, proof, &mut coin);

        assert!(result.is_ok())
    }

    #[test]
    fn test_sum_check_product_failure() {
        let num_variables = 14;
        let values_0 = rand_vector(1 << num_variables);
        let values_1 = rand_vector(1 << num_variables);
        let mut claim =
            values_0.iter().zip(values_1.iter()).fold(ZERO, |acc, (x, y)| *x * *y + acc);

        // modifying the claim should make the Verifier reject the proof
        claim += ONE;

        let ml_0 = MultiLinear::new(values_0.to_vec()).expect("should not fail");
        let ml_1 = MultiLinear::new(values_1.to_vec()).expect("should not fail");
        let mut mls = vec![ml_0, ml_1];
        let virtual_poly = ProductComposition;

        // Prover
        let prover = SumCheckProver::new(virtual_poly);
        let mut coin = RpoRandomCoin::new(Word::default());
        let proof = prover.prove(claim, &mut mls, num_variables, &mut coin).unwrap();

        // Verifier
        let plain_query_builder = PlainQueryBuilder;
        let verifier = SumCheckVerifier::new(virtual_poly, plain_query_builder);
        let mut coin = RpoRandomCoin::new(Word::default());
        let result = verifier.verify(claim, proof, &mut coin);

        assert!(result.is_err())
    }

    struct PlainQueryBuilder;

    impl FinalQueryBuilder for PlainQueryBuilder {
        type Field = Felt;

        fn build_query(
            &self,
            openings: &[Self::Field],
            _evaluation_point: &[Self::Field],
        ) -> Vec<Self::Field> {
            openings.to_vec()
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct ProjectionComposition {
        coordinate: usize,
    }

    impl ProjectionComposition {
        pub fn new(coordinate: usize) -> Self {
            Self { coordinate }
        }
    }

    impl<E> CompositionPolynomial<E> for ProjectionComposition
    where
        E: FieldElement,
    {
        fn num_variables(&self) -> usize {
            1
        }

        fn max_degree(&self) -> usize {
            1
        }

        fn evaluate(&self, query: &[E]) -> E {
            query[self.coordinate]
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub struct ProductComposition;

    impl<E> CompositionPolynomial<E> for ProductComposition
    where
        E: FieldElement,
    {
        fn num_variables(&self) -> usize {
            2
        }

        fn max_degree(&self) -> usize {
            2
        }

        fn evaluate(&self, query: &[E]) -> E {
            assert_eq!(query.len(), 2);
            query[0] * query[1]
        }
    }

    pub fn eval<E>(p: &[E], x: E) -> E
    where
        E: FieldElement,
    {
        p.iter().rev().fold(E::ZERO, |acc, &coeff| acc * x + coeff)
    }
}
