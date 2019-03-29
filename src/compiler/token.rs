#![allow(dead_code)]

/// Lua Token
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// ...
    Vararg,
    /// ;
    SepSemi,
    /// ,
    SepComma,
    /// .
    SepDot,
    /// :
    SepColon,
    /// ::
    SepLabel,
    /// (
    SepLparen,
    /// )
    SepRparen,
    /// [
    SepLbrack,
    /// ]
    SepRbrack,
    /// {
    SepLcurly,
    /// }
    SepRcurly,
    /// =
    OpAssign,
    /// - (sub or unm)
    OpMinus,
    /// ~ (bnot or bxor)
    OpWave,
    /// +
    OpAdd,
    /// *
    OpMul,
    /// /
    OpDiv,
    /// //
    OpIDiv,
    /// ^
    OpPow,
    /// %
    OpMod,
    /// &
    OpBitAnd,
    /// |
    OpBitOr,
    /// >>
    OpShr,
    /// <<
    OpShl,
    /// ..
    OpConcat,
    /// <
    OpLt,
    /// <=
    OpLe,
    /// >
    OpGt,
    /// >=
    OpGe,
    /// ==
    OPEq,
    /// ~=
    OpNe,
    /// #
    OpLen,
    /// and
    OpAnd,
    /// or
    OpOr,
    /// not
    OpNot,
    /// break
    KwBreak,
    /// do
    KwDo,
    /// else
    KwElse,
    /// elseif
    KwElseIf,
    /// end
    KwEnd,
    /// false
    KwFalse,
    /// for
    KwFor,
    /// function
    KwFunction,
    /// goto
    KwGoto,
    /// if
    KwIf,
    /// in
    KwIn,
    /// local
    KwLocal,
    /// nil
    KwNil,
    /// repeat
    KwRepeat,
    /// return
    KwReturn,
    /// then
    KwThen,
    /// true
    KwTrue,
    /// until
    KwUntil,
    /// while
    KwWhile,
    /// `id`
    Identifier(String),
    /// `number`
    Number(String),
    /// `string`
    String(String),
}