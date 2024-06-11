use core::ops::Add;

use crate::trace::virtual_bus::multilinear::CompositionPolynomial;
use crate::trace::virtual_bus::multilinear::EqFunction;
use alloc::vec::Vec;
use miden_air::trace::chiplets::{MEMORY_D0_COL_IDX, MEMORY_D1_COL_IDX};
use miden_air::trace::decoder::{DECODER_OP_BITS_OFFSET, DECODER_USER_OP_HELPERS_OFFSET};
use miden_air::trace::range::{M_COL_IDX, V_COL_IDX};
use miden_air::trace::{CHIPLETS_OFFSET, TRACE_WIDTH};
use prover::CircuitLayerPolys;
use static_assertions::const_assert;
use vm_core::{Felt, FieldElement};

mod error;
mod prover;
pub use prover::prove;

mod verifier;
pub use verifier::verify;

use super::multilinear::MultiLinearPoly;
use super::sum_check::Proof as SumCheckProof;

/// Defines the number of wires in the input layer that are generated from a single main trace row.
const NUM_WIRES_PER_TRACE_ROW: usize = 8;
const_assert!(NUM_WIRES_PER_TRACE_ROW.is_power_of_two());

/// Represents a fraction `numerator / denominator` as a pair `(numerator, denominator)`. This is
/// the type for the gates' inputs in [`prover::EvaluatedCircuit`].
///
/// Hence, addition is defined in the natural way fractions are added together: `a/b + c/d = (ad +
/// bc) / bd`.
#[derive(Debug, Clone, Copy)]
pub struct CircuitWire<E: FieldElement> {
    numerator: E,
    denominator: E,
}

impl<E> CircuitWire<E>
where
    E: FieldElement,
{
    /// Creates new projective coordinates from a numerator and a denominator.
    pub fn new(numerator: E, denominator: E) -> Self {
        assert_ne!(denominator, E::ZERO);

        Self {
            numerator,
            denominator,
        }
    }
}

impl<E> Add for CircuitWire<E>
where
    E: FieldElement,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let numerator = self.numerator * other.denominator + other.numerator * self.denominator;
        let denominator = self.denominator * other.denominator;

        Self::new(numerator, denominator)
    }
}

/// Converts a main trace row (or more generally "query") to numerators and denominators of the
/// input layer.
fn evaluate_fractions_at_main_trace_query<E>(
    query: &[E],
    log_up_randomness: &[E],
) -> [[E; NUM_WIRES_PER_TRACE_ROW]; 2]
where
    E: FieldElement,
{
    // numerators
    let multiplicity = query[M_COL_IDX];
    let f_m = {
        let mem_selec0 = query[CHIPLETS_OFFSET];
        let mem_selec1 = query[CHIPLETS_OFFSET + 1];
        let mem_selec2 = query[CHIPLETS_OFFSET + 2];
        mem_selec0 * mem_selec1 * (E::ONE - mem_selec2)
    };

    let f_rc = {
        let op_bit_4 = query[DECODER_OP_BITS_OFFSET + 4];
        let op_bit_5 = query[DECODER_OP_BITS_OFFSET + 5];
        let op_bit_6 = query[DECODER_OP_BITS_OFFSET + 6];

        (E::ONE - op_bit_4) * (E::ONE - op_bit_5) * op_bit_6
    };

    // denominators
    let alphas = log_up_randomness;

    let table_denom = alphas[0] - query[V_COL_IDX];
    let memory_denom_0 = -(alphas[0] - query[MEMORY_D0_COL_IDX]);
    let memory_denom_1 = -(alphas[0] - query[MEMORY_D1_COL_IDX]);
    let stack_value_denom_0 = -(alphas[0] - query[DECODER_USER_OP_HELPERS_OFFSET]);
    let stack_value_denom_1 = -(alphas[0] - query[DECODER_USER_OP_HELPERS_OFFSET + 1]);
    let stack_value_denom_2 = -(alphas[0] - query[DECODER_USER_OP_HELPERS_OFFSET + 2]);
    let stack_value_denom_3 = -(alphas[0] - query[DECODER_USER_OP_HELPERS_OFFSET + 3]);

    [
        [multiplicity, f_m, f_m, f_rc, f_rc, f_rc, f_rc, E::ZERO],
        [
            table_denom,
            memory_denom_0,
            memory_denom_1,
            stack_value_denom_0,
            stack_value_denom_1,
            stack_value_denom_2,
            stack_value_denom_3,
            E::ONE,
        ],
    ]
}

/// Computes the wires added to the input layer that come from a given main trace row (or more
/// generally, "query").
fn compute_input_layer_wires_at_main_trace_query<E>(
    query: &[E],
    log_up_randomness: &[E],
) -> [CircuitWire<E>; NUM_WIRES_PER_TRACE_ROW]
where
    E: FieldElement,
{
    let [numerators, denominators] =
        evaluate_fractions_at_main_trace_query(query, log_up_randomness);
    let input_gates_values: Vec<CircuitWire<E>> = numerators
        .into_iter()
        .zip(denominators)
        .map(|(numerator, denominator)| CircuitWire::new(numerator, denominator))
        .collect();
    input_gates_values.try_into().unwrap()
}

/// A GKR proof for the correct evaluation of the sum of fractions circuit.
#[derive(Debug)]
pub struct GkrCircuitProof<E: FieldElement> {
    circuit_outputs: CircuitLayerPolys<E>,
    before_final_layer_proofs: BeforeFinalLayerProof<E>,
}

/// A set of sum-check proofs for all GKR layers but for the input circuit layer.
#[derive(Debug)]
pub struct BeforeFinalLayerProof<E: FieldElement> {
    pub proof: Vec<SumCheckProof<E>>,
}

/// Represents a claim to be proven by a subsequent call to the sum-check protocol.
#[derive(Debug)]
pub struct GkrClaim<E: FieldElement> {
    pub evaluation_point: Vec<E>,
    pub claimed_evaluation: (E, E),
}

/// A composition polynomial used in the GKR protocol for all of its sum-checks except the final
/// one.
#[derive(Clone)]
pub struct GkrComposition<E>
where
    E: FieldElement<BaseField = Felt>,
{
    pub combining_randomness: E,
}

impl<E> GkrComposition<E>
where
    E: FieldElement<BaseField = Felt>,
{
    pub fn new(combining_randomness: E) -> Self {
        Self {
            combining_randomness,
        }
    }
}

impl<E> CompositionPolynomial<E> for GkrComposition<E>
where
    E: FieldElement<BaseField = Felt>,
{
    fn max_degree(&self) -> u32 {
        3
    }

    fn evaluate(&self, query: &[E]) -> E {
        let eval_left_numerator = query[0];
        let eval_right_numerator = query[1];
        let eval_left_denominator = query[2];
        let eval_right_denominator = query[3];
        let eq_eval = query[4];
        eq_eval
            * ((eval_left_numerator * eval_right_denominator
                + eval_right_numerator * eval_left_denominator)
                + eval_left_denominator * eval_right_denominator * self.combining_randomness)
    }
}

/// A composition polynomial used in the GKR protocol for its final sum-check.
#[derive(Clone)]
pub struct GkrCompositionMerge<E>
where
    E: FieldElement<BaseField = Felt>,
{
    pub sum_check_combining_randomness: E,
    pub tensored_merge_randomness: Vec<E>,
    pub log_up_randomness: Vec<E>,
}

impl<E> GkrCompositionMerge<E>
where
    E: FieldElement<BaseField = Felt>,
{
    pub fn new(
        combining_randomness: E,
        merge_randomness: Vec<E>,
        log_up_randomness: Vec<E>,
    ) -> Self {
        let tensored_merge_randomness =
            EqFunction::ml_at(merge_randomness.clone()).evaluations().to_vec();

        Self {
            sum_check_combining_randomness: combining_randomness,
            tensored_merge_randomness,
            log_up_randomness,
        }
    }
}

impl<E> CompositionPolynomial<E> for GkrCompositionMerge<E>
where
    E: FieldElement<BaseField = Felt>,
{
    fn max_degree(&self) -> u32 {
        // Computed as:
        // 1 + max(left_numerator_degree + right_denom_degree, right_numerator_degree +
        // left_denom_degree)
        5
    }

    fn evaluate(&self, query: &[E]) -> E {
        let [numerators, denominators] =
            evaluate_fractions_at_main_trace_query(query, &self.log_up_randomness);

        let numerators = MultiLinearPoly::from_evaluations(numerators.to_vec()).unwrap();
        let denominators = MultiLinearPoly::from_evaluations(denominators.to_vec()).unwrap();

        let (left_numerators, right_numerators) = numerators.project_least_significant_variable();
        let (left_denominators, right_denominators) =
            denominators.project_least_significant_variable();

        let eval_left_numerators =
            left_numerators.evaluate_with_lagrange_kernel(&self.tensored_merge_randomness);
        let eval_right_numerators =
            right_numerators.evaluate_with_lagrange_kernel(&self.tensored_merge_randomness);

        let eval_left_denominators =
            left_denominators.evaluate_with_lagrange_kernel(&self.tensored_merge_randomness);
        let eval_right_denominators =
            right_denominators.evaluate_with_lagrange_kernel(&self.tensored_merge_randomness);

        let eq_eval = query[TRACE_WIDTH];

        eq_eval
            * ((eval_left_numerators * eval_right_denominators
                + eval_right_numerators * eval_left_denominators)
                + eval_left_denominators
                    * eval_right_denominators
                    * self.sum_check_combining_randomness)
    }
}
