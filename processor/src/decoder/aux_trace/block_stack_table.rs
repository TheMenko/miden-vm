use super::{
    AuxColumnBuilder, Felt, FieldElement, MainTrace, CALL, DYN, END, JOIN, LOOP, ONE, RESPAN, SPAN,
    SPLIT, SYSCALL, ZERO,
};

// BLOCK STACK TABLE COLUMN BUILDER
// ================================================================================================

/// Builds the execution trace of the decoder's `p1` column which describes the state of the block
/// stack table via multiset checks.
#[derive(Default)]
pub struct BlockStackColumnBuilder {}

impl<E: FieldElement<BaseField = Felt>> AuxColumnBuilder<E> for BlockStackColumnBuilder {
    /// Removes a row from the block stack table.
    fn get_requests_at(&self, main_trace: &MainTrace, alphas: &[E], i: usize) -> E {
        let op_code_felt = main_trace.get_op_code(i);
        let op_code = op_code_felt.as_int() as u8;

        match op_code {
            RESPAN => get_block_stack_table_removal_multiplicand(main_trace, i, true, alphas),
            END => get_block_stack_table_removal_multiplicand(main_trace, i, false, alphas),
            _ => E::ONE,
        }
    }

    /// Adds a row to the block stack table.
    fn get_responses_at(&self, main_trace: &MainTrace, alphas: &[E], i: usize) -> E {
        let op_code_felt = main_trace.get_op_code(i);
        let op_code = op_code_felt.as_int() as u8;

        match op_code {
            JOIN | SPLIT | SPAN | DYN | LOOP | RESPAN | CALL | SYSCALL => {
                get_block_stack_table_inclusion_multiplicand(main_trace, i, alphas, op_code)
            }
            _ => E::ONE,
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Computes the multiplicand representing the removal of a row from the block stack table.
fn get_block_stack_table_removal_multiplicand<E: FieldElement<BaseField = Felt>>(
    main_trace: &MainTrace,
    i: usize,
    is_respan: bool,
    alphas: &[E],
) -> E {
    let block_id = main_trace.addr(i);
    let parent_id = if is_respan {
        main_trace.decoder_hasher_state_element(1, i + 1)
    } else {
        main_trace.addr(i + 1)
    };
    let is_loop = main_trace.is_loop_flag(i);

    let elements = if main_trace.is_call_flag(i) == ONE || main_trace.is_syscall_flag(i) == ONE {
        let parent_ctx = main_trace.ctx(i + 1);
        let parent_fmp = main_trace.fmp(i + 1);
        let parent_stack_depth = main_trace.stack_depth(i + 1);
        let parent_next_overflow_addr = main_trace.parent_overflow_address(i + 1);
        let parent_fn_hash = main_trace.fn_hash(i);

        [
            ONE,
            block_id,
            parent_id,
            is_loop,
            parent_ctx,
            parent_fmp,
            parent_stack_depth,
            parent_next_overflow_addr,
            parent_fn_hash[0],
            parent_fn_hash[1],
            parent_fn_hash[2],
            parent_fn_hash[0],
        ]
    } else {
        let mut result = [ZERO; 12];
        result[0] = ONE;
        result[1] = block_id;
        result[2] = parent_id;
        result[3] = is_loop;
        result
    };

    let mut value = E::ZERO;

    for (&alpha, &element) in alphas.iter().zip(elements.iter()) {
        value += alpha.mul_base(element);
    }
    value
}

/// Computes the multiplicand representing the inclusion of a new row to the block stack table.
fn get_block_stack_table_inclusion_multiplicand<E: FieldElement<BaseField = Felt>>(
    main_trace: &MainTrace,
    i: usize,
    alphas: &[E],
    op_code: u8,
) -> E {
    let block_id = main_trace.addr(i + 1);
    let parent_id = if op_code == RESPAN {
        main_trace.decoder_hasher_state_element(1, i + 1)
    } else {
        main_trace.addr(i)
    };
    let is_loop = if op_code == LOOP {
        main_trace.stack_element(0, i)
    } else {
        ZERO
    };
    let elements = if op_code == CALL || op_code == SYSCALL {
        let parent_ctx = main_trace.ctx(i);
        let parent_fmp = main_trace.fmp(i);
        let parent_stack_depth = main_trace.stack_depth(i);
        let parent_next_overflow_addr = main_trace.parent_overflow_address(i);
        let parent_fn_hash = main_trace.decoder_hasher_state_first_half(i);
        [
            ONE,
            block_id,
            parent_id,
            is_loop,
            parent_ctx,
            parent_fmp,
            parent_stack_depth,
            parent_next_overflow_addr,
            parent_fn_hash[0],
            parent_fn_hash[1],
            parent_fn_hash[2],
            parent_fn_hash[3],
        ]
    } else {
        let mut result = [ZERO; 12];
        result[0] = ONE;
        result[1] = block_id;
        result[2] = parent_id;
        result[3] = is_loop;
        result
    };

    let mut value = E::ZERO;

    for (&alpha, &element) in alphas.iter().zip(elements.iter()) {
        value += alpha.mul_base(element);
    }
    value
}
