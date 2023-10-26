use super::{CodeBody, Felt, Instruction, Node, ProcedureId, RpoDigest, ToString};
use crate::MAX_PUSH_INPUTS;
use num_enum::TryFromPrimitive;
use vm_core::utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable};

mod debug;
mod deserialization;
mod serialization;
pub mod signatures;

// OPERATION CODES ENUM
// ================================================================================================

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive)]
pub enum OpCode {
    Assert = 0,
    AssertWithError = 1,
    AssertEq = 2,
    AssertEqWithError = 3,
    AssertEqw = 4,
    AssertEqwWithError = 5,
    Assertz = 6,
    AssertzWithError = 7,
    Add = 8,
    AddImm = 9,
    Sub = 10,
    SubImm = 11,
    Mul = 12,
    MulImm = 13,
    Div = 14,
    DivImm = 15,
    Neg = 16,
    Inv = 17,
    Incr = 18,
    Pow2 = 19,
    Exp = 20,
    ExpImm = 21,
    ExpBitLength = 22,
    Not = 23,
    And = 24,
    Or = 25,
    Xor = 26,
    Eq = 27,
    EqImm = 28,
    Neq = 29,
    NeqImm = 30,
    Eqw = 31,
    Lt = 32,
    Lte = 33,
    Gt = 34,
    Gte = 35,
    IsOdd = 36,

    // ----- ext2 operations ----------------------------------------------------------------------
    Ext2Add = 37,
    Ext2Sub = 38,
    Ext2Mul = 39,
    Ext2Div = 40,
    Ext2Neg = 41,
    Ext2Inv = 42,

    // ----- u32 manipulation ---------------------------------------------------------------------
    U32Test = 43,
    U32TestW = 44,
    U32Assert = 45,
    U32AssertWithError = 46,
    U32Assert2 = 47,
    U32Assert2WithError = 48,
    U32AssertW = 49,
    U32AssertWWithError = 50,
    U32Split = 51,
    U32Cast = 52,
    U32CheckedAdd = 53,
    U32CheckedAddImm = 54,
    U32WrappingAdd = 55,
    U32WrappingAddImm = 56,
    U32OverflowingAdd = 57,
    U32OverflowingAddImm = 58,
    U32OverflowingAdd3 = 59,
    U32WrappingAdd3 = 60,
    U32CheckedSub = 61,
    U32CheckedSubImm = 62,
    U32WrappingSub = 63,
    U32WrappingSubImm = 64,
    U32OverflowingSub = 65,
    U32OverflowingSubImm = 66,
    U32CheckedMul = 67,
    U32CheckedMulImm = 68,
    U32WrappingMul = 69,
    U32WrappingMulImm = 70,
    U32OverflowingMul = 71,
    U32OverflowingMulImm = 72,
    U32OverflowingMadd = 73,
    U32WrappingMadd = 74,
    U32CheckedDiv = 75,
    U32CheckedDivImm = 76,
    U32UncheckedDiv = 77,
    U32UncheckedDivImm = 78,
    U32CheckedMod = 79,
    U32CheckedModImm = 80,
    U32UncheckedMod = 81,
    U32UncheckedModImm = 82,
    U32CheckedDivMod = 83,
    U32CheckedDivModImm = 84,
    U32UncheckedDivMod = 85,
    U32UncheckedDivModImm = 86,
    U32CheckedAnd = 87,
    U32CheckedOr = 88,
    U32CheckedXor = 89,
    U32CheckedNot = 90,
    U32CheckedShr = 91,
    U32CheckedShrImm = 92,
    U32UncheckedShr = 93,
    U32UncheckedShrImm = 94,
    U32CheckedShl = 95,
    U32CheckedShlImm = 96,
    U32UncheckedShl = 97,
    U32UncheckedShlImm = 98,
    U32CheckedRotr = 99,
    U32CheckedRotrImm = 100,
    U32UncheckedRotr = 101,
    U32UncheckedRotrImm = 102,
    U32CheckedRotl = 103,
    U32CheckedRotlImm = 104,
    U32UncheckedRotl = 105,
    U32UncheckedRotlImm = 106,
    U32CheckedPopcnt = 107,
    U32UncheckedPopcnt = 108,
    U32CheckedEq = 109,
    U32CheckedEqImm = 110,
    U32CheckedNeq = 111,
    U32CheckedNeqImm = 112,
    U32CheckedLt = 113,
    U32UncheckedLt = 114,
    U32CheckedLte = 115,
    U32UncheckedLte = 116,
    U32CheckedGt = 117,
    U32UncheckedGt = 118,
    U32CheckedGte = 119,
    U32UncheckedGte = 120,
    U32CheckedMin = 121,
    U32UncheckedMin = 122,
    U32CheckedMax = 123,
    U32UncheckedMax = 124,

    // ----- stack manipulation -------------------------------------------------------------------
    Drop = 125,
    DropW = 126,
    PadW = 127,
    Dup0 = 128,
    Dup1 = 129,
    Dup2 = 130,
    Dup3 = 131,
    Dup4 = 132,
    Dup5 = 133,
    Dup6 = 134,
    Dup7 = 135,
    Dup8 = 136,
    Dup9 = 137,
    Dup10 = 138,
    Dup11 = 139,
    Dup12 = 140,
    Dup13 = 141,
    Dup14 = 142,
    Dup15 = 143,
    DupW0 = 144,
    DupW1 = 145,
    DupW2 = 146,
    DupW3 = 147,
    Swap1 = 148,
    Swap2 = 149,
    Swap3 = 150,
    Swap4 = 151,
    Swap5 = 152,
    Swap6 = 153,
    Swap7 = 154,
    Swap8 = 155,
    Swap9 = 156,
    Swap10 = 157,
    Swap11 = 158,
    Swap12 = 159,
    Swap13 = 160,
    Swap14 = 161,
    Swap15 = 162,
    SwapW1 = 163,
    SwapW2 = 164,
    SwapW3 = 165,
    SwapDW = 166,
    MovUp2 = 167,
    MovUp3 = 168,
    MovUp4 = 169,
    MovUp5 = 170,
    MovUp6 = 171,
    MovUp7 = 172,
    MovUp8 = 173,
    MovUp9 = 174,
    MovUp10 = 175,
    MovUp11 = 176,
    MovUp12 = 177,
    MovUp13 = 178,
    MovUp14 = 179,
    MovUp15 = 180,
    MovUpW2 = 181,
    MovUpW3 = 182,
    MovDn2 = 183,
    MovDn3 = 184,
    MovDn4 = 185,
    MovDn5 = 186,
    MovDn6 = 187,
    MovDn7 = 188,
    MovDn8 = 189,
    MovDn9 = 190,
    MovDn10 = 191,
    MovDn11 = 192,
    MovDn12 = 193,
    MovDn13 = 194,
    MovDn14 = 195,
    MovDn15 = 196,
    MovDnW2 = 197,
    MovDnW3 = 198,
    CSwap = 199,
    CSwapW = 200,
    CDrop = 201,
    CDropW = 202,

    // ----- input / output operations ------------------------------------------------------------
    PushU8 = 203,
    PushU16 = 204,
    PushU32 = 205,
    PushFelt = 206,
    PushWord = 207,
    PushU8List = 208,
    PushU16List = 209,
    PushU32List = 210,
    PushFeltList = 211,

    Locaddr = 212,
    Sdepth = 213,
    Caller = 214,
    Clk = 215,

    MemLoad = 216,
    MemLoadImm = 217,
    MemLoadW = 218,
    MemLoadWImm = 219,
    LocLoad = 220,
    LocLoadW = 221,
    MemStore = 222,
    MemStoreImm = 223,
    LocStore = 224,
    MemStoreW = 225,
    MemStoreWImm = 226,
    LocStoreW = 227,

    MemStream = 228,
    AdvPipe = 229,

    AdvPush = 230,
    AdvLoadW = 231,

    AdvInject = 232,

    // ----- cryptographic operations -------------------------------------------------------------
    Hash = 233,
    HMerge = 234,
    HPerm = 235,
    MTreeGet = 236,
    MTreeSet = 237,
    MTreeMerge = 238,
    MTreeVerify = 239,

    // ----- STARK proof verification -------------------------------------------------------------
    FriExt2Fold4 = 240,

    // ----- exec / call --------------------------------------------------------------------------
    ExecLocal = 241,
    ExecImported = 242,
    CallLocal = 243,
    CallMastRoot = 244,
    CallImported = 245,
    SysCall = 246,
    DynExec = 247,
    DynCall = 248,
    ProcRefLocal = 249,
    ProcRefImported = 250,

    // ----- debugging ----------------------------------------------------------------------------
    Debug = 251,

    // ----- emit --------------------------------------------------------------------------------
    Emit = 252,

    // ----- control flow -------------------------------------------------------------------------
    IfElse = 253,
    Repeat = 254,
    While = 255,
}

impl Serializable for OpCode {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_u8(*self as u8);
    }
}

impl Deserializable for OpCode {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let value = source.read_u8()?;
        Self::try_from(value).map_err(|_| {
            DeserializationError::InvalidValue("could not read a valid opcode".to_string())
        })
    }
}
