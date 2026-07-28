use crate::mir::*;
use kyc_core::ast::*;

pub(crate) fn literal_to_mir(value: &Literal) -> (MirType, MirConstant) {
    match value {
        Literal::Integer(n) => {
            if *n >= i32::MIN as i64 && *n <= i32::MAX as i64 {
                (MirType::I32, MirConstant::I32(*n as i32))
            } else {
                (MirType::I64, MirConstant::I64(*n))
            }
        }
        Literal::Float(n) => (MirType::F64, MirConstant::F64(*n)),
        Literal::String(s) => (MirType::Str, MirConstant::String(s.clone())),
        Literal::Boolean(b) => (MirType::Bool, MirConstant::Bool(*b)),
                    Literal::Char(c) => (MirType::Char, MirConstant::I32(*c)),
        Literal::None => (MirType::I32, MirConstant::Void),
        Literal::Null => (MirType::Ptr(Box::new(MirType::Void)), MirConstant::Null),
    }
}

/// Return true if the MIR type is an integer type (i1, i8, i16, i32, i64, u8, u16, u32, u64, char, bool).
pub(crate) fn is_int_type(t: &MirType) -> bool {
    matches!(t, MirType::I1 | MirType::I8 | MirType::I16 | MirType::I32 | MirType::I64 | MirType::U8 | MirType::U16 | MirType::U32 | MirType::U64 | MirType::Char | MirType::Bool)
}

/// Return the wider of two types, supporting both int and float widening.
/// F64 > F32 > any integer type.
pub(crate) fn wider_type(a: &MirType, b: &MirType) -> MirType {
    use MirType::*;
    if a == b { return a.clone(); }
    if matches!(a, F64) || matches!(b, F64) { return F64; }
    if matches!(a, F32) || matches!(b, F32) { return F32; }
    wider_int_type(a, b)
}

/// Return the wider of two integer types. If either is not an integer, returns I32.
pub(crate) fn wider_int_type(a: &MirType, b: &MirType) -> MirType {
    use MirType::*;
    if a == b { return a.clone(); }
    let bit_width = |t: &MirType| -> u32 {
        match t {
            I1 | Bool => 1,
            I8 | U8 | Char => 8,
            I16 | U16 => 16,
            I32 | U32 => 32,
            I64 | U64 => 64,
            _ => 32,
        }
    };
    let wa = bit_width(a);
    let wb = bit_width(b);
    if wa >= wb { a.clone() } else { b.clone() }
}
