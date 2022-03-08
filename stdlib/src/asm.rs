//! This module is automatically generated during build time and should not be modified manually.

/// An array of modules defined in Miden standard library.
///
/// Entries in the array are tuples containing module namespace and module source code.
#[rustfmt::skip]
pub const MODULES: [(&str, &str); 3] = [
// ----- std::crypto::hashes::blake3 --------------------------------------------------------------
("std::crypto::hashes::blake3", ""),
// ----- std::math::u256 --------------------------------------------------------------------------
("std::math::u256", "export.add_unsafe
    swapw.3
    movup.3
    movup.7
    u32add.unsafe
    movup.4
    movup.7
    u32addc.unsafe
    movup.4
    movup.6
    u32addc.unsafe
    movup.4
    movup.5
    u32addc.unsafe
    movdn.12
    swapw.2
    movup.12
    movup.4
    movup.8
    u32addc.unsafe
    movup.4
    movup.7
    u32addc.unsafe
    movup.4
    movup.6
    u32addc.unsafe
    movup.4
    movup.5
    u32addc.unsafe
    drop
end

export.sub_unsafe
    swapw.3
    movup.3
    movup.7
    u32sub.unsafe
    movup.7
    u32add.unsafe
    movup.5
    movup.2
    u32sub.unsafe
    movup.2
    add
    movup.6
    u32add.unsafe
    movup.5
    movup.2
    u32sub.unsafe
    movup.2
    add
    movup.5
    u32add.unsafe
    movup.5
    movup.2
    u32sub.unsafe
    movup.2
    add
    movdn.12
    swapw.2
    movup.12
    movup.4
    u32add.unsafe
    movup.8
    movup.2
    u32sub.unsafe
    movup.2
    add
    movup.4
    u32add.unsafe
    movup.7
    movup.2
    u32sub.unsafe
    movup.2
    add
    movup.4
    u32add.unsafe
    movup.6
    movup.2
    u32sub.unsafe
    movup.2
    add
    movup.5
    movup.5
    movup.2
    u32add.unsafe
    drop
    u32sub.unsafe
    drop
end

export.and
    swapw.3
    movup.3
    movup.7
    u32and
    movup.3
    movup.6
    u32and
    movup.3
    movup.5
    u32and
    movup.3
    movup.4
    u32and
    swapw.2
    movup.3
    movup.7
    u32and
    movup.3
    movup.6
    u32and
    movup.3
    movup.5
    u32and
    movup.3
    movup.4
    u32and
end

export.or
    swapw.3
    movup.3
    movup.7
    u32or
    movup.3
    movup.6
    u32or
    movup.3
    movup.5
    u32or
    movup.3
    movup.4
    u32or
    swapw.2
    movup.3
    movup.7
    u32or
    movup.3
    movup.6
    u32or
    movup.3
    movup.5
    u32or
    movup.3
    movup.4
    u32or
end

export.xor
    swapw.3
    movup.3
    movup.7
    u32xor
    movup.3
    movup.6
    u32xor
    movup.3
    movup.5
    u32xor
    movup.3
    movup.4
    u32xor
    swapw.2
    movup.3
    movup.7
    u32xor
    movup.3
    movup.6
    u32xor
    movup.3
    movup.5
    u32xor
    movup.3
    movup.4
    u32xor
end

export.iszero_unsafe
    eq.0
    repeat.7
        swap
        eq.0
        and
    end
end

export.eq_unsafe
    swapw.3
    eqw
    movdn.8
    dropw
    dropw
    movdn.8
    eqw
    movdn.8
    dropw
    dropw
    and
end"),
// ----- std::math::u64 ---------------------------------------------------------------------------
("std::math::u64", "# Performs addition of two unsigned 64 bit integers discarding the overflow. #
# The input values are assumed to be represented using 32 bit limbs, but this is not checked. #
# Stack transition looks as follows: #
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = (a + b) % 2^64 #
export.add_unsafe
    swap
    movup.3
    u32add.unsafe
    movup.3
    movup.3
    u32addc
    drop
end

# Performs multiplication of two unsigned 64 bit integers discarding the overflow. #
# The input values are assumed to be represented using 32 bit limbs, but this is not checked. #
# Stack transition looks as follows: #
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = (a * b) % 2^64 #
export.mul_unsafe
    dup.3
    dup.2
    u32mul.unsafe
    movup.4
    movup.4
    u32madd
    drop
    movup.3
    movup.3
    u32madd
    drop
end

# ===== DIVISION ================================================================================ #

# Performs division of two unsigned 64 bit integers discarding the remainder. #
# The input values are assumed to be represented using 32 bit limbs, but this is not checked. #
# Stack transition looks as follows: #
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a // b #
export.div_unsafe
    adv.u64div          # inject the quotient and the remainder into the advice tape #
    
    push.adv.1          # read the values from the advice tape and make sure they are u32's #
    u32assert           # TODO: this can be optimized once we have u32assert2 instruction #
    push.adv.1
    u32assert
    push.adv.1
    u32assert
    push.adv.1
    u32assert

    dup.5               # multiply quotient by the divisor; this also consumes the divisor #
    dup.4
    u32mul.unsafe
    dup.6
    dup.6
    u32madd
    eq.0
    assert
    movup.7
    dup.5
    u32madd
    eq.0
    assert
    movup.6
    dup.5
    mul
    eq.0
    assert
    swap

    movup.3             # add remainder to the previous result; this also consumes the remainder #
    u32add.unsafe
    movup.3
    movup.3
    u32addc
    eq.0
    assert

    movup.4             # make sure the result we got is equal to the dividend #
    assert.eq
    movup.3
    assert.eq           # quotient remains on the stack #
end
"),
];
