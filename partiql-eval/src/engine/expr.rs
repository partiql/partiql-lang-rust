use crate::engine::arena::{Arena, SlotId};
use crate::engine::error::{EngineError, Result};
use crate::engine::value::{value_get_field_ref, ValueOwned, ValueRef};
use partiql_logical::{CallExpr, CallName, Lit, PathComponent, ValueExpr, VarRefType};
use partiql_value::BindingsName;
use regex;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(ValueOwned),
    SlotRef(SlotId),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Exp(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Neg(Box<Expr>),
    Pos(Box<Expr>),
    Concat(Box<Expr>, Box<Expr>),
    In(Box<Expr>, Box<Expr>),
    Like {
        value: Box<Expr>,
        pattern: String,
        escape: String,
    },
    LikeDynamic {
        value: Box<Expr>,
        pattern: Box<Expr>,
        escape: Box<Expr>,
    },
    GetField(Box<Expr>, String),
    UdfCall {
        name: String,
        args: Vec<Expr>,
    },
    Tuple {
        attrs: Vec<Expr>,
        values: Vec<Expr>,
    },
    List {
        elements: Vec<Expr>,
    },
    Bag {
        elements: Vec<Expr>,
    },
}

// TODO: Implement fully
#[derive(Clone, Debug)]
pub enum CastTarget {
    #[allow(dead_code)]
    I64,
    Decimal,
    F64,
    #[allow(dead_code)]
    String,
}

#[derive(Clone, Debug)]
pub enum Inst {
    LoadConst {
        dst: u16,
        const_idx: u16,
    },
    Cast {
        dst: u16,
        from: u16,
        to: CastTarget,
    },
    AddI64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    AddF64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    AddDecimal {
        dst: u16,
        a: u16,
        b: u16,
    },
    // TODO: Is this the right thing to do here?
    AddDynamic {
        dst: u16,
        a: u16,
        a_cast: u16,
        b: u16,
        b_cast: u16,
    },
    SubI64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    SubF64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    SubDecimal {
        dst: u16,
        a: u16,
        b: u16,
    },
    SubDynamic {
        dst: u16,
        a: u16,
        a_cast: u16,
        b: u16,
        b_cast: u16,
    },
    ModI64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    ModF64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    ModDecimal {
        dst: u16,
        a: u16,
        b: u16,
    },
    ModDynamic {
        dst: u16,
        a: u16,
        a_cast: u16,
        b: u16,
        b_cast: u16,
    },
    EqI64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    EqF64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    EqDecimal {
        dst: u16,
        a: u16,
        b: u16,
    },
    EqBool {
        dst: u16,
        a: u16,
        b: u16,
    },
    EqStr {
        dst: u16,
        a: u16,
        b: u16,
    },
    EqDynamic {
        dst: u16,
        a: u16,
        a_cast: u16,
        b: u16,
        b_cast: u16,
    },
    // --- Mul ---
    MulI64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    MulF64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    MulDecimal {
        dst: u16,
        a: u16,
        b: u16,
    },
    MulDynamic {
        dst: u16,
        a: u16,
        a_cast: u16,
        b: u16,
        b_cast: u16,
    },
    // --- Div ---
    DivI64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    DivF64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    DivDecimal {
        dst: u16,
        a: u16,
        b: u16,
    },
    DivDynamic {
        dst: u16,
        a: u16,
        a_cast: u16,
        b: u16,
        b_cast: u16,
    },
    // --- Exp ---
    ExpF64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    ExpDynamic {
        dst: u16,
        a: u16,
        a_cast: u16,
        b: u16,
        b_cast: u16,
    },
    // --- Gt ---
    GtI64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    GtF64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    GtDecimal {
        dst: u16,
        a: u16,
        b: u16,
    },
    GtStr {
        dst: u16,
        a: u16,
        b: u16,
    },
    GtDynamic {
        dst: u16,
        a: u16,
        a_cast: u16,
        b: u16,
        b_cast: u16,
    },
    // --- Lt ---
    LtI64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    LtF64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    LtDecimal {
        dst: u16,
        a: u16,
        b: u16,
    },
    LtStr {
        dst: u16,
        a: u16,
        b: u16,
    },
    LtDynamic {
        dst: u16,
        a: u16,
        a_cast: u16,
        b: u16,
        b_cast: u16,
    },
    // --- Concat ---
    ConcatStr {
        dst: u16,
        a: u16,
        b: u16,
    },
    ConcatDynamic {
        dst: u16,
        a: u16,
        b: u16,
    },
    // --- In ---
    InDynamic {
        dst: u16,
        value: u16,
        collection: u16,
    },
    AndBool {
        dst: u16,
        a: u16,
        b: u16,
    },
    OrBool {
        dst: u16,
        a: u16,
        b: u16,
    },
    NotBool {
        dst: u16,
        src: u16,
    },
    NegNum {
        dst: u16,
        src: u16,
    },
    LikeMatch {
        dst: u16,
        value: u16,
        pattern_idx: u16,
    },
    LikeDynamicMatch {
        dst: u16,
        value: u16,
        pattern: u16,
        escape: u16,
    },
    MaterializeCursor {
        src: u16,
        cursor_id: u16,
    },
    GetField {
        dst: u16,
        base: u16,
        key_idx: u16,
    },
    StoreSlot {
        slot: SlotId,
        src: u16,
    },
    CallUdf {
        dst: u16,
        func_idx: u16,
        args: Vec<u16>,
    },
    MakeTuple {
        dst: u16,
        attr_regs: Vec<u16>,
        value_regs: Vec<u16>,
    },
    MakeList {
        dst: u16,
        element_regs: Vec<u16>,
    },
    MakeBag {
        dst: u16,
        element_regs: Vec<u16>,
    },

    // === Relational Instructions (SFW bytecode) ===
    /// Open a data source cursor. cursor_id indexes into the VM's cursor array.
    /// The cursor's ScanLayout is determined at compile time from CursorMetadata.
    OpenCursor {
        cursor_id: u16,
    },

    /// Advance cursor to next row, writing columns into registers per ScanLayout.
    /// If no more rows, jump to `eof_target` instruction index.
    NextRow {
        cursor_id: u16,
        eof_target: u32,
    },

    /// Close cursor and release resources.
    CloseCursor {
        cursor_id: u16,
    },

    /// Unconditional jump to instruction at `target`.
    Jump {
        target: u32,
    },

    /// Jump to `target` if register `src` is NOT true (false/null/missing).
    /// Used for WHERE clause filtering.
    JumpIfNotTrue {
        src: u16,
        target: u32,
    },

    /// Yield the current register state as a result row. The VM pauses here
    /// and returns control to the consumer. Execution resumes at the next
    /// instruction when the consumer requests the next row.
    EmitRow,

    /// Halt execution. The query is complete.
    Halt,

    /// Decrement the i64 counter in `counter_reg`. If it was already 0, jump to `target`.
    /// Used for LIMIT: initialize counter with the limit value, then DecrOrJump before EmitRow.
    DecrOrJump {
        counter_reg: u16,
        target: u32,
    },
}

impl Inst {
    fn new_add_i64(dst: u16, a: u16, b: u16) -> Self {
        Inst::AddI64 { dst, a, b }
    }

    fn new_add_f64(dst: u16, a: u16, b: u16) -> Self {
        Inst::AddF64 { dst, a, b }
    }

    fn new_add_decimal(dst: u16, a: u16, b: u16) -> Self {
        Inst::AddDecimal { dst, a, b }
    }

    fn new_sub_i64(dst: u16, a: u16, b: u16) -> Self {
        Inst::SubI64 { dst, a, b }
    }

    fn new_sub_f64(dst: u16, a: u16, b: u16) -> Self {
        Inst::SubF64 { dst, a, b }
    }

    fn new_sub_decimal(dst: u16, a: u16, b: u16) -> Self {
        Inst::SubDecimal { dst, a, b }
    }

    fn new_mod_i64(dst: u16, a: u16, b: u16) -> Self {
        Inst::ModI64 { dst, a, b }
    }

    fn new_mod_f64(dst: u16, a: u16, b: u16) -> Self {
        Inst::ModF64 { dst, a, b }
    }

    fn new_mod_decimal(dst: u16, a: u16, b: u16) -> Self {
        Inst::ModDecimal { dst, a, b }
    }

    fn new_eq_i64(dst: u16, a: u16, b: u16) -> Self {
        Inst::EqI64 { dst, a, b }
    }

    fn new_eq_f64(dst: u16, a: u16, b: u16) -> Self {
        Inst::EqF64 { dst, a, b }
    }

    fn new_eq_decimal(dst: u16, a: u16, b: u16) -> Self {
        Inst::EqDecimal { dst, a, b }
    }

    // --- Mul constructors ---
    fn new_mul_i64(dst: u16, a: u16, b: u16) -> Self {
        Inst::MulI64 { dst, a, b }
    }
    fn new_mul_f64(dst: u16, a: u16, b: u16) -> Self {
        Inst::MulF64 { dst, a, b }
    }
    fn new_mul_decimal(dst: u16, a: u16, b: u16) -> Self {
        Inst::MulDecimal { dst, a, b }
    }

    // --- Div constructors ---
    fn new_div_i64(dst: u16, a: u16, b: u16) -> Self {
        Inst::DivI64 { dst, a, b }
    }
    fn new_div_f64(dst: u16, a: u16, b: u16) -> Self {
        Inst::DivF64 { dst, a, b }
    }
    fn new_div_decimal(dst: u16, a: u16, b: u16) -> Self {
        Inst::DivDecimal { dst, a, b }
    }

    // --- Exp constructors ---
    fn new_exp_f64(dst: u16, a: u16, b: u16) -> Self {
        Inst::ExpF64 { dst, a, b }
    }

    // --- Gt constructors ---
    fn new_gt_i64(dst: u16, a: u16, b: u16) -> Self {
        Inst::GtI64 { dst, a, b }
    }
    fn new_gt_f64(dst: u16, a: u16, b: u16) -> Self {
        Inst::GtF64 { dst, a, b }
    }
    fn new_gt_decimal(dst: u16, a: u16, b: u16) -> Self {
        Inst::GtDecimal { dst, a, b }
    }
    fn new_gt_str(dst: u16, a: u16, b: u16) -> Self {
        Inst::GtStr { dst, a, b }
    }

    // --- Lt constructors ---
    fn new_lt_i64(dst: u16, a: u16, b: u16) -> Self {
        Inst::LtI64 { dst, a, b }
    }
    fn new_lt_f64(dst: u16, a: u16, b: u16) -> Self {
        Inst::LtF64 { dst, a, b }
    }
    fn new_lt_decimal(dst: u16, a: u16, b: u16) -> Self {
        Inst::LtDecimal { dst, a, b }
    }
    fn new_lt_str(dst: u16, a: u16, b: u16) -> Self {
        Inst::LtStr { dst, a, b }
    }
}

pub struct Program {
    pub insts: Vec<Inst>,
    pub(crate) consts: Vec<ValueOwned>,
    #[allow(dead_code)] // Arena is used to back const_refs but not accessed directly
    arena: Arena,
    const_refs: Vec<ValueRef<'static>>,
    pub keys: Vec<String>,
    pub reg_count: u16,
    #[allow(dead_code)]
    pub slot_count: u16,
}

// Safety: Program is Sync despite containing an Arena because:
// 1. The arena is only written to during Program::build() and Program::clone()
// 2. After construction, the arena is never mutated - it only serves as backing storage
// 3. All ValueRef references into the arena are read-only
// 4. The arena and consts are co-owned by Program and dropped together
unsafe impl Sync for Program {}

impl Clone for Program {
    fn clone(&self) -> Self {
        // Create a new arena for the cloned program
        let arena = Arena::default();

        // Re-convert all constants to ValueRef using the new arena
        let const_refs: Vec<ValueRef<'_>> = self
            .consts
            .iter()
            .map(|c| ValueRef::from_owned(c, &arena))
            .collect();

        // Safety: Same as in build() - Program owns both consts and arena
        let const_refs: Vec<ValueRef<'static>> = unsafe { std::mem::transmute(const_refs) };

        Program {
            insts: self.insts.clone(),
            consts: self.consts.clone(),
            arena,
            const_refs,
            keys: self.keys.clone(),
            reg_count: self.reg_count,
            slot_count: self.slot_count,
        }
    }
}

impl Program {
    /// Create an empty program (no instructions). Used as a default.
    pub fn empty() -> Self {
        Program {
            insts: Vec::new(),
            consts: Vec::new(),
            arena: Arena::default(),
            const_refs: Vec::new(),
            keys: Vec::new(),
            reg_count: 0,
            slot_count: 0,
        }
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn eval_binary_arithmetic_dynamic<'a>(
        &self,
        dst: u16,
        lhs: u16,
        rhs: u16,
        lhs_cast: u16,
        rhs_cast: u16,
        fn_i64: fn(u16, u16, u16) -> Inst,
        fn_f64: fn(u16, u16, u16) -> Inst,
        fn_decimal: fn(u16, u16, u16) -> Inst,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<()> {
        let av = regs[lhs as usize];
        let bv = regs[rhs as usize];
        match (av, bv) {
            (ValueRef::I64(_), ValueRef::I64(_)) => {
                self.eval_inst(&fn_i64(dst, lhs, rhs), arena, regs, udf)?
            }
            (ValueRef::I64(_), ValueRef::F64(_)) => {
                let cast = Inst::Cast {
                    dst: lhs_cast,
                    from: lhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_f64(dst, lhs_cast, rhs), arena, regs, udf)?
            }
            (ValueRef::I64(_), ValueRef::Decimal(_)) => {
                let cast = Inst::Cast {
                    dst: lhs_cast,
                    from: lhs,
                    to: CastTarget::Decimal,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_decimal(dst, lhs_cast, rhs), arena, regs, udf)?
            }
            (ValueRef::F64(_), ValueRef::F64(_)) => {
                self.eval_inst(&fn_f64(dst, lhs, rhs), arena, regs, udf)?
            }
            // TODO: Check this.
            (ValueRef::F64(_), ValueRef::I64(_)) => {
                let cast = Inst::Cast {
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_i64(dst, lhs, rhs_cast), arena, regs, udf)?
            }
            // TODO: Check this. May need to be other way around.
            (ValueRef::F64(_), ValueRef::Decimal(_)) => {
                let cast = Inst::Cast {
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_i64(dst, lhs, rhs_cast), arena, regs, udf)?
            }
            (ValueRef::Decimal(_), ValueRef::Decimal(_)) => {
                self.eval_inst(&fn_decimal(dst, lhs, rhs), arena, regs, udf)?
            }
            (ValueRef::Decimal(_), ValueRef::I64(_)) => {
                let cast = Inst::Cast {
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::Decimal,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_decimal(dst, lhs, rhs_cast), arena, regs, udf)?
            }
            (ValueRef::Decimal(_), ValueRef::F64(_)) => {
                let cast = Inst::Cast {
                    dst: lhs_cast,
                    from: lhs,
                    to: CastTarget::Decimal,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_decimal(dst, lhs_cast, rhs), arena, regs, udf)?
            }
            (_, _) => {
                return Err(EngineError::IllegalState(
                    "Type mismatch for add!".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Evaluate dynamic equality comparison with type coercion.
    ///
    /// Semantics:
    /// - If either operand is Null or Missing, the result is Null.
    /// - Numeric types (I64, F64, Decimal) are coerced to a common type before comparison.
    /// - Same-type non-numeric comparisons (Bool==Bool, Str==Str) are handled directly.
    /// - Type mismatches between incompatible types return false.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn eval_eq_dynamic<'a>(
        &self,
        dst: u16,
        lhs: u16,
        rhs: u16,
        lhs_cast: u16,
        rhs_cast: u16,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<()> {
        let av = regs[lhs as usize];
        let bv = regs[rhs as usize];

        // Null/Missing propagation: any operand is Null or Missing → result is Null
        if matches!(av, ValueRef::Null | ValueRef::Missing)
            || matches!(bv, ValueRef::Null | ValueRef::Missing)
        {
            regs[dst as usize] = ValueRef::Null;
            return Ok(());
        }

        match (av, bv) {
            // Numeric same-type
            (ValueRef::I64(_), ValueRef::I64(_)) => {
                self.eval_inst(&Inst::new_eq_i64(dst, lhs, rhs), arena, regs, udf)?
            }
            (ValueRef::F64(_), ValueRef::F64(_)) => {
                self.eval_inst(&Inst::new_eq_f64(dst, lhs, rhs), arena, regs, udf)?
            }
            (ValueRef::Decimal(_), ValueRef::Decimal(_)) => {
                self.eval_inst(&Inst::new_eq_decimal(dst, lhs, rhs), arena, regs, udf)?
            }

            // Numeric cross-type: I64 vs F64
            (ValueRef::I64(_), ValueRef::F64(_)) => {
                let cast = Inst::Cast {
                    dst: lhs_cast,
                    from: lhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&Inst::new_eq_f64(dst, lhs_cast, rhs), arena, regs, udf)?
            }
            (ValueRef::F64(_), ValueRef::I64(_)) => {
                let cast = Inst::Cast {
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&Inst::new_eq_f64(dst, lhs, rhs_cast), arena, regs, udf)?
            }

            // Numeric cross-type: I64 vs Decimal
            (ValueRef::I64(_), ValueRef::Decimal(_)) => {
                let cast = Inst::Cast {
                    dst: lhs_cast,
                    from: lhs,
                    to: CastTarget::Decimal,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&Inst::new_eq_decimal(dst, lhs_cast, rhs), arena, regs, udf)?
            }
            (ValueRef::Decimal(_), ValueRef::I64(_)) => {
                let cast = Inst::Cast {
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::Decimal,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&Inst::new_eq_decimal(dst, lhs, rhs_cast), arena, regs, udf)?
            }

            // Numeric cross-type: F64 vs Decimal — cast Decimal to F64
            (ValueRef::F64(_), ValueRef::Decimal(_)) => {
                let cast = Inst::Cast {
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&Inst::new_eq_f64(dst, lhs, rhs_cast), arena, regs, udf)?
            }
            (ValueRef::Decimal(_), ValueRef::F64(_)) => {
                let cast = Inst::Cast {
                    dst: lhs_cast,
                    from: lhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&Inst::new_eq_f64(dst, lhs_cast, rhs), arena, regs, udf)?
            }

            // Non-numeric same-type
            (ValueRef::Bool(_), ValueRef::Bool(_)) => self.eval_inst(
                &Inst::EqBool {
                    dst,
                    a: lhs,
                    b: rhs,
                },
                arena,
                regs,
                udf,
            )?,
            (ValueRef::Str(_), ValueRef::Str(_)) => self.eval_inst(
                &Inst::EqStr {
                    dst,
                    a: lhs,
                    b: rhs,
                },
                arena,
                regs,
                udf,
            )?,

            // Type mismatch between incompatible types → false
            (_, _) => {
                regs[dst as usize] = ValueRef::Bool(false);
            }
        }
        Ok(())
    }

    /// Evaluate a dynamic binary comparison (Gt or Lt) with numeric type coercion.
    ///
    /// Semantics:
    /// - If either operand is Null or Missing, the result is Null.
    /// - Numeric types (I64, F64, Decimal) are coerced to a common type before comparison.
    /// - Same-type string comparisons are handled directly.
    /// - Type mismatches between incompatible types return Null.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn eval_binary_comparison_dynamic<'a>(
        &self,
        dst: u16,
        lhs: u16,
        rhs: u16,
        lhs_cast: u16,
        rhs_cast: u16,
        fn_i64: fn(u16, u16, u16) -> Inst,
        fn_f64: fn(u16, u16, u16) -> Inst,
        fn_decimal: fn(u16, u16, u16) -> Inst,
        fn_str: fn(u16, u16, u16) -> Inst,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<()> {
        let av = regs[lhs as usize];
        let bv = regs[rhs as usize];

        // Null/Missing propagation
        if matches!(av, ValueRef::Null | ValueRef::Missing)
            || matches!(bv, ValueRef::Null | ValueRef::Missing)
        {
            regs[dst as usize] = ValueRef::Null;
            return Ok(());
        }

        match (av, bv) {
            // Numeric same-type
            (ValueRef::I64(_), ValueRef::I64(_)) => {
                self.eval_inst(&fn_i64(dst, lhs, rhs), arena, regs, udf)?
            }
            (ValueRef::F64(_), ValueRef::F64(_)) => {
                self.eval_inst(&fn_f64(dst, lhs, rhs), arena, regs, udf)?
            }
            (ValueRef::Decimal(_), ValueRef::Decimal(_)) => {
                self.eval_inst(&fn_decimal(dst, lhs, rhs), arena, regs, udf)?
            }
            // Numeric cross-type: I64 vs F64
            (ValueRef::I64(_), ValueRef::F64(_)) => {
                let cast = Inst::Cast {
                    dst: lhs_cast,
                    from: lhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_f64(dst, lhs_cast, rhs), arena, regs, udf)?
            }
            (ValueRef::F64(_), ValueRef::I64(_)) => {
                let cast = Inst::Cast {
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_f64(dst, lhs, rhs_cast), arena, regs, udf)?
            }
            // Numeric cross-type: I64 vs Decimal
            (ValueRef::I64(_), ValueRef::Decimal(_)) => {
                let cast = Inst::Cast {
                    dst: lhs_cast,
                    from: lhs,
                    to: CastTarget::Decimal,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_decimal(dst, lhs_cast, rhs), arena, regs, udf)?
            }
            (ValueRef::Decimal(_), ValueRef::I64(_)) => {
                let cast = Inst::Cast {
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::Decimal,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_decimal(dst, lhs, rhs_cast), arena, regs, udf)?
            }
            // Numeric cross-type: F64 vs Decimal — cast Decimal to F64
            (ValueRef::F64(_), ValueRef::Decimal(_)) => {
                let cast = Inst::Cast {
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_f64(dst, lhs, rhs_cast), arena, regs, udf)?
            }
            (ValueRef::Decimal(_), ValueRef::F64(_)) => {
                let cast = Inst::Cast {
                    dst: lhs_cast,
                    from: lhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_f64(dst, lhs_cast, rhs), arena, regs, udf)?
            }
            // String comparison
            (ValueRef::Str(_), ValueRef::Str(_)) => {
                self.eval_inst(&fn_str(dst, lhs, rhs), arena, regs, udf)?
            }
            // Incompatible types
            (_, _) => {
                regs[dst as usize] = ValueRef::Null;
            }
        }
        Ok(())
    }

    /// Evaluate dynamic exponentiation with type coercion to f64.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn eval_exp_dynamic<'a>(
        &self,
        dst: u16,
        lhs: u16,
        rhs: u16,
        lhs_cast: u16,
        rhs_cast: u16,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<()> {
        let av = regs[lhs as usize];
        let bv = regs[rhs as usize];
        // Cast both operands to f64 if needed, then use ExpF64
        let a_reg = match av {
            ValueRef::F64(_) => lhs,
            ValueRef::I64(_) | ValueRef::Decimal(_) => {
                let cast = Inst::Cast {
                    dst: lhs_cast,
                    from: lhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                lhs_cast
            }
            _ => {
                return Err(EngineError::IllegalState(
                    "Type mismatch for exp!".to_string(),
                ));
            }
        };
        let b_reg = match bv {
            ValueRef::F64(_) => rhs,
            ValueRef::I64(_) | ValueRef::Decimal(_) => {
                let cast = Inst::Cast {
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                rhs_cast
            }
            _ => {
                return Err(EngineError::IllegalState(
                    "Type mismatch for exp!".to_string(),
                ));
            }
        };
        self.eval_inst(&Inst::new_exp_f64(dst, a_reg, b_reg), arena, regs, udf)?;
        Ok(())
    }

    #[inline]
    fn eval_cast<'a>(
        &self,
        dst: u16,
        from: u16,
        to: &CastTarget,
        _arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
    ) -> Result<()> {
        let v = regs[from as usize];
        let result = match v {
            ValueRef::I64(i) => match to {
                CastTarget::Decimal => ValueRef::Decimal(Decimal::new(i, 0)),
                CastTarget::F64 => ValueRef::F64(i as f64),
                CastTarget::I64 => v,
                CastTarget::String => return Err(EngineError::NotImplemented),
            },
            ValueRef::Decimal(d) => match to {
                CastTarget::Decimal => v,
                CastTarget::I64 => {
                    ValueRef::I64(d.to_i64().expect("Could not cast decimal to integer."))
                }
                CastTarget::F64 => {
                    ValueRef::F64(d.to_f64().expect("Could not cast decimal to f64."))
                }
                CastTarget::String => return Err(EngineError::NotImplemented),
            },
            _ => return Err(EngineError::NotImplemented),
        };
        regs[dst as usize] = result;
        Ok(())
    }

    #[inline]
    pub(crate) fn eval_inst<'a>(
        &self,
        inst: &Inst,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<()> {
        match inst {
            Inst::LoadConst { dst, const_idx } => {
                // Zero-copy: Just index into pre-converted const_refs
                let value_ref = self
                    .const_refs
                    .get(*const_idx as usize)
                    .ok_or_else(|| EngineError::IllegalState("invalid const index".to_string()))?;
                // Safety: Extend lifetime from 'static to 'a. This is safe because
                // Program (which owns the data) outlives the query execution lifetime 'a.
                let value_ref: ValueRef<'a> = unsafe { std::mem::transmute(*value_ref) };
                regs[*dst as usize] = value_ref;
            }
            Inst::AddI64 { dst, a, b } => {
                let av = regs[*a as usize].as_i64()?;
                let bv = regs[*b as usize].as_i64()?;
                regs[*dst as usize] = ValueRef::I64(av + bv);
            }
            Inst::AddF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::F64(av + bv);
            }
            Inst::AddDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Decimal(av + bv);
            }
            Inst::Cast { dst, from, to } => {
                self.eval_cast(*dst, *from, to, arena, regs)?;
            }
            // TODO: If we want to make sure that we inline the other functions, we can create two variants of Inst (Static and Dynamic). Then, eval_inst will match
            // on type and then match on instruction. Dynamic instructions will be the only ones that may not be inlined.
            Inst::AddDynamic {
                dst,
                a,
                b,
                a_cast,
                b_cast,
            } => {
                self.eval_binary_arithmetic_dynamic(
                    *dst,
                    *a,
                    *b,
                    *a_cast,
                    *b_cast,
                    Inst::new_add_i64,
                    Inst::new_add_f64,
                    Inst::new_add_decimal,
                    arena,
                    regs,
                    udf,
                )?;
            }
            Inst::SubI64 { dst, a, b } => {
                let av = regs[*a as usize].as_i64()?;
                let bv = regs[*b as usize].as_i64()?;
                regs[*dst as usize] = ValueRef::I64(av - bv);
            }
            Inst::SubF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::F64(av - bv);
            }
            Inst::SubDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Decimal(av - bv);
            }
            Inst::SubDynamic {
                dst,
                a,
                b,
                a_cast,
                b_cast,
            } => {
                self.eval_binary_arithmetic_dynamic(
                    *dst,
                    *a,
                    *b,
                    *a_cast,
                    *b_cast,
                    Inst::new_sub_i64,
                    Inst::new_sub_f64,
                    Inst::new_sub_decimal,
                    arena,
                    regs,
                    udf,
                )?;
            }
            Inst::ModI64 { dst, a, b } => {
                let av = regs[*a as usize].as_i64()?;
                let bv = regs[*b as usize].as_i64()?;
                regs[*dst as usize] = ValueRef::I64(av % bv);
            }
            Inst::ModF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::F64(av % bv);
            }
            Inst::ModDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Decimal(av % bv);
            }
            Inst::ModDynamic {
                dst,
                a,
                b,
                a_cast,
                b_cast,
            } => {
                self.eval_binary_arithmetic_dynamic(
                    *dst,
                    *a,
                    *b,
                    *a_cast,
                    *b_cast,
                    Inst::new_mod_i64,
                    Inst::new_mod_f64,
                    Inst::new_mod_decimal,
                    arena,
                    regs,
                    udf,
                )?;
            }
            Inst::EqI64 { dst, a, b } => {
                let av = regs[*a as usize].as_i64()?;
                let bv = regs[*b as usize].as_i64()?;
                regs[*dst as usize] = ValueRef::Bool(av == bv);
            }
            Inst::EqF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::Bool(av == bv);
            }
            Inst::EqDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Bool(av == bv);
            }
            Inst::EqBool { dst, a, b } => {
                let av = regs[*a as usize].as_bool()?;
                let bv = regs[*b as usize].as_bool()?;
                regs[*dst as usize] = ValueRef::Bool(av == bv);
            }
            Inst::EqStr { dst, a, b } => {
                let av = regs[*a as usize];
                let bv = regs[*b as usize];
                let result = match (av, bv) {
                    (ValueRef::Str(a), ValueRef::Str(b)) => a == b,
                    _ => false,
                };
                regs[*dst as usize] = ValueRef::Bool(result);
            }
            Inst::EqDynamic {
                dst,
                a,
                b,
                a_cast,
                b_cast,
            } => {
                self.eval_eq_dynamic(*dst, *a, *b, *a_cast, *b_cast, arena, regs, udf)?;
            }
            // --- Mul ---
            Inst::MulI64 { dst, a, b } => {
                let av = regs[*a as usize].as_i64()?;
                let bv = regs[*b as usize].as_i64()?;
                regs[*dst as usize] = ValueRef::I64(av * bv);
            }
            Inst::MulF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::F64(av * bv);
            }
            Inst::MulDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Decimal(av * bv);
            }
            Inst::MulDynamic {
                dst,
                a,
                b,
                a_cast,
                b_cast,
            } => {
                self.eval_binary_arithmetic_dynamic(
                    *dst,
                    *a,
                    *b,
                    *a_cast,
                    *b_cast,
                    Inst::new_mul_i64,
                    Inst::new_mul_f64,
                    Inst::new_mul_decimal,
                    arena,
                    regs,
                    udf,
                )?;
            }
            // --- Div ---
            Inst::DivI64 { dst, a, b } => {
                let av = regs[*a as usize].as_i64()?;
                let bv = regs[*b as usize].as_i64()?;
                regs[*dst as usize] = ValueRef::I64(av / bv);
            }
            Inst::DivF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::F64(av / bv);
            }
            Inst::DivDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Decimal(av / bv);
            }
            Inst::DivDynamic {
                dst,
                a,
                b,
                a_cast,
                b_cast,
            } => {
                self.eval_binary_arithmetic_dynamic(
                    *dst,
                    *a,
                    *b,
                    *a_cast,
                    *b_cast,
                    Inst::new_div_i64,
                    Inst::new_div_f64,
                    Inst::new_div_decimal,
                    arena,
                    regs,
                    udf,
                )?;
            }
            // --- Exp ---
            Inst::ExpF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::F64(av.powf(bv));
            }
            Inst::ExpDynamic {
                dst,
                a,
                b,
                a_cast,
                b_cast,
            } => {
                self.eval_exp_dynamic(*dst, *a, *b, *a_cast, *b_cast, arena, regs, udf)?;
            }
            // --- Gt ---
            Inst::GtI64 { dst, a, b } => {
                let av = regs[*a as usize].as_i64()?;
                let bv = regs[*b as usize].as_i64()?;
                regs[*dst as usize] = ValueRef::Bool(av > bv);
            }
            Inst::GtF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::Bool(av > bv);
            }
            Inst::GtDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Bool(av > bv);
            }
            Inst::GtStr { dst, a, b } => {
                let result = match (regs[*a as usize], regs[*b as usize]) {
                    (ValueRef::Str(a), ValueRef::Str(b)) => a > b,
                    _ => false,
                };
                regs[*dst as usize] = ValueRef::Bool(result);
            }
            Inst::GtDynamic {
                dst,
                a,
                b,
                a_cast,
                b_cast,
            } => {
                self.eval_binary_comparison_dynamic(
                    *dst,
                    *a,
                    *b,
                    *a_cast,
                    *b_cast,
                    Inst::new_gt_i64,
                    Inst::new_gt_f64,
                    Inst::new_gt_decimal,
                    Inst::new_gt_str,
                    arena,
                    regs,
                    udf,
                )?;
            }
            // --- Lt ---
            Inst::LtI64 { dst, a, b } => {
                let av = regs[*a as usize].as_i64()?;
                let bv = regs[*b as usize].as_i64()?;
                regs[*dst as usize] = ValueRef::Bool(av < bv);
            }
            Inst::LtF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::Bool(av < bv);
            }
            Inst::LtDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Bool(av < bv);
            }
            Inst::LtStr { dst, a, b } => {
                let result = match (regs[*a as usize], regs[*b as usize]) {
                    (ValueRef::Str(a), ValueRef::Str(b)) => a < b,
                    _ => false,
                };
                regs[*dst as usize] = ValueRef::Bool(result);
            }
            Inst::LtDynamic {
                dst,
                a,
                b,
                a_cast,
                b_cast,
            } => {
                self.eval_binary_comparison_dynamic(
                    *dst,
                    *a,
                    *b,
                    *a_cast,
                    *b_cast,
                    Inst::new_lt_i64,
                    Inst::new_lt_f64,
                    Inst::new_lt_decimal,
                    Inst::new_lt_str,
                    arena,
                    regs,
                    udf,
                )?;
            }
            // --- Concat ---
            Inst::ConcatStr { dst, a, b } => {
                let result = match (regs[*a as usize], regs[*b as usize]) {
                    (ValueRef::Str(a_str), ValueRef::Str(b_str)) => {
                        let mut s = String::with_capacity(a_str.len() + b_str.len());
                        s.push_str(a_str);
                        s.push_str(b_str);
                        let bytes = s.into_bytes();
                        let slice = arena.alloc_slice(&bytes);
                        // Safety: we just built this from valid UTF-8 strings
                        let str_ref = unsafe { std::str::from_utf8_unchecked(slice) };
                        ValueRef::Str(str_ref)
                    }
                    _ => {
                        return Err(EngineError::TypeError(
                            "concat requires string operands".to_string(),
                        ));
                    }
                };
                regs[*dst as usize] = result;
            }
            Inst::ConcatDynamic { dst, a, b } => {
                let av = regs[*a as usize];
                let bv = regs[*b as usize];
                // Null/Missing propagation
                if matches!(av, ValueRef::Null | ValueRef::Missing)
                    || matches!(bv, ValueRef::Null | ValueRef::Missing)
                {
                    regs[*dst as usize] = ValueRef::Null;
                } else {
                    self.eval_inst(
                        &Inst::ConcatStr {
                            dst: *dst,
                            a: *a,
                            b: *b,
                        },
                        arena,
                        regs,
                        udf,
                    )?;
                }
            }
            // --- In ---
            Inst::InDynamic {
                dst,
                value,
                collection,
            } => {
                let val = regs[*value as usize];
                let coll = regs[*collection as usize];
                // Null/Missing propagation for value
                if matches!(val, ValueRef::Null | ValueRef::Missing) {
                    regs[*dst as usize] = ValueRef::Null;
                } else {
                    match coll {
                        ValueRef::List(items) | ValueRef::Bag(items) => {
                            let mut found = false;
                            for item in items.iter() {
                                if value_ref_eq(val, *item) {
                                    found = true;
                                    break;
                                }
                            }
                            regs[*dst as usize] = ValueRef::Bool(found);
                        }
                        ValueRef::Null | ValueRef::Missing => {
                            regs[*dst as usize] = ValueRef::Null;
                        }
                        _ => {
                            // Single value comparison: val IN non-collection
                            regs[*dst as usize] = ValueRef::Bool(value_ref_eq(val, coll));
                        }
                    }
                }
            }
            Inst::AndBool { dst, a, b } => {
                let av = regs[*a as usize].as_bool()?;
                let bv = regs[*b as usize].as_bool()?;
                regs[*dst as usize] = ValueRef::Bool(av && bv);
            }
            Inst::OrBool { dst, a, b } => {
                let av = regs[*a as usize].as_bool()?;
                let bv = regs[*b as usize].as_bool()?;
                regs[*dst as usize] = ValueRef::Bool(av || bv);
            }
            Inst::NotBool { dst, src } => {
                let sv = regs[*src as usize].as_bool()?;
                regs[*dst as usize] = ValueRef::Bool(!sv);
            }
            Inst::NegNum { dst, src } => {
                regs[*dst as usize] = match regs[*src as usize] {
                    ValueRef::I64(n) => ValueRef::I64(-n),
                    ValueRef::F64(n) => ValueRef::F64(-n),
                    ValueRef::Decimal(d) => ValueRef::Decimal(-d),
                    ValueRef::Null => ValueRef::Null,
                    ValueRef::Missing => ValueRef::Missing,
                    _ => ValueRef::Missing,
                };
            }
            Inst::LikeMatch {
                dst,
                value,
                pattern_idx,
            } => {
                let pattern_str = self
                    .keys
                    .get(*pattern_idx as usize)
                    .ok_or_else(|| EngineError::IllegalState("invalid pattern key".to_string()))?;
                regs[*dst as usize] = match regs[*value as usize] {
                    ValueRef::Str(s) => match regex::Regex::new(pattern_str) {
                        Ok(re) => ValueRef::Bool(re.is_match(s)),
                        Err(_) => ValueRef::Missing,
                    },
                    ValueRef::Null => ValueRef::Null,
                    ValueRef::Missing => ValueRef::Missing,
                    _ => ValueRef::Missing,
                };
            }
            Inst::LikeDynamicMatch {
                dst,
                value,
                pattern,
                escape,
            } => {
                regs[*dst as usize] = match (
                    regs[*value as usize],
                    regs[*pattern as usize],
                    regs[*escape as usize],
                ) {
                    (ValueRef::Str(v), ValueRef::Str(p), ValueRef::Str(e)) => {
                        let re_pattern = like_to_re_pattern(p, e);
                        match regex::Regex::new(&re_pattern) {
                            Ok(re) => ValueRef::Bool(re.is_match(v)),
                            Err(_) => ValueRef::Missing,
                        }
                    }
                    (ValueRef::Null, _, _) | (_, ValueRef::Null, _) => ValueRef::Null,
                    (ValueRef::Missing, _, _) | (_, ValueRef::Missing, _) => ValueRef::Missing,
                    _ => ValueRef::Missing,
                };
            }
            Inst::GetField { dst, base, key_idx } => {
                let key = self
                    .keys
                    .get(*key_idx as usize)
                    .ok_or_else(|| EngineError::IllegalState("invalid key index".to_string()))?;
                regs[*dst as usize] = value_get_field_ref(regs[*base as usize], key);
            }
            Inst::StoreSlot { slot, src } => {
                regs[*slot as usize] = regs[*src as usize];
            }
            Inst::CallUdf {
                dst,
                func_idx,
                args,
            } => {
                let name = self
                    .keys
                    .get(*func_idx as usize)
                    .ok_or_else(|| EngineError::IllegalState("invalid udf key".to_string()))?;
                let registry = udf.ok_or_else(|| EngineError::UdfNotFound(name.clone()))?;
                let mut argv = Vec::with_capacity(args.len());
                for arg in args {
                    argv.push(regs[*arg as usize]);
                }
                let result = registry.call(name, &argv, arena)?;
                regs[*dst as usize] = result;
            }
            Inst::MakeTuple {
                dst,
                attr_regs,
                value_regs,
            } => {
                // Zero-copy tuple construction!
                // Build iterator of (name ValueRef, value ValueRef) pairs
                // Attribute names MUST be ValueRef::Str
                let fields =
                    attr_regs
                        .iter()
                        .zip(value_regs.iter())
                        .map(|(attr_reg, value_reg)| {
                            (regs[*attr_reg as usize], regs[*value_reg as usize])
                        });

                // Single arena allocation for entire tuple structure
                let tuple_ref = arena.alloc_tuple(fields);
                regs[*dst as usize] = ValueRef::Tuple(tuple_ref);
            }
            Inst::MakeList { dst, element_regs } => {
                // Zero-copy list construction!
                // Collect ValueRefs from element registers
                let elements: Vec<ValueRef<'a>> =
                    element_regs.iter().map(|reg| regs[*reg as usize]).collect();

                // Single arena allocation for the element slice
                let list_slice = arena.alloc_slice(&elements);
                regs[*dst as usize] = ValueRef::List(list_slice);
            }
            Inst::MakeBag { dst, element_regs } => {
                // Zero-copy bag construction!
                // Collect ValueRefs from element registers
                let elements: Vec<ValueRef<'a>> =
                    element_regs.iter().map(|reg| regs[*reg as usize]).collect();

                // Single arena allocation for the element slice
                let bag_slice = arena.alloc_slice(&elements);
                regs[*dst as usize] = ValueRef::Bag(bag_slice);
            }

            // Relational instructions are handled by the VM dispatch loop,
            // not by eval_inst. If we reach them here, it's a bug.
            Inst::OpenCursor { .. }
            | Inst::NextRow { .. }
            | Inst::CloseCursor { .. }
            | Inst::Jump { .. }
            | Inst::JumpIfNotTrue { .. }
            | Inst::EmitRow
            | Inst::Halt
            | Inst::DecrOrJump { .. }
            | Inst::MaterializeCursor { .. } => {
                return Err(EngineError::IllegalState(
                    "relational instruction encountered in scalar eval_inst".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Evaluate the program using borrowed registers from the VM
    ///
    /// # Arguments
    /// * `frame` - Row frame with arena for value storage
    /// * `regs` - Pre-allocated register array from VM (first N are slots, rest are temporaries)
    /// * `udf` - Optional UDF registry for function calls
    ///
    /// The register array is borrowed from PartiQLVM and reused across all rows,
    /// eliminating heap allocations during expression evaluation.
    /// The first `slot_count` registers are reserved for slot data.
    #[allow(dead_code)]
    pub(crate) fn eval<'a>(
        &self,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<()> {
        for inst in &self.insts {
            self.eval_inst(inst, arena, regs, udf)?
        }
        Ok(())
    }
}

pub(crate) trait UdfRegistry {
    fn call<'a>(&self, name: &str, args: &[ValueRef<'a>], arena: &'a Arena)
        -> Result<ValueRef<'a>>;
}

pub trait SlotResolver {
    fn resolve_var(&self, name: &BindingsName<'_>, scope: VarRefType) -> Option<SlotId>;
    fn resolve_alias(&self, name: &BindingsName<'_>) -> Option<SlotId>;
    fn resolve_field(&self, name: &BindingsName<'_>) -> Option<SlotId>;
    fn is_alias(&self, name: &BindingsName<'_>) -> bool;
}

#[derive(Default)]
pub struct ProgramBuilder {
    pub(crate) insts: Vec<Inst>,
    consts: Vec<ValueOwned>,
    keys: Vec<String>,
    next_reg: u16,
    slot_count: u16,
}

impl ProgramBuilder {
    pub fn new(slot_count: u16) -> Self {
        ProgramBuilder {
            insts: Vec::new(),
            consts: Vec::new(),
            keys: Vec::new(),
            next_reg: slot_count,
            slot_count,
        }
    }

    // === Relational instruction emission helpers ===

    /// Emit an OpenCursor instruction
    pub fn emit_open_cursor(&mut self, cursor_id: u16) {
        self.insts.push(Inst::OpenCursor { cursor_id });
    }

    /// Emit a NextRow instruction. Returns the index of this instruction
    /// so the caller can patch the eof_target later.
    pub fn emit_next_row(&mut self, cursor_id: u16) -> usize {
        let idx = self.insts.len();
        // Placeholder eof_target — must be patched by caller
        self.insts.push(Inst::NextRow {
            cursor_id,
            eof_target: 0,
        });
        idx
    }

    /// Emit a CloseCursor instruction
    pub fn emit_close_cursor(&mut self, cursor_id: u16) {
        self.insts.push(Inst::CloseCursor { cursor_id });
    }

    /// Emit an unconditional Jump. Returns the index so it can be patched.
    pub fn emit_jump(&mut self) -> usize {
        let idx = self.insts.len();
        self.insts.push(Inst::Jump { target: 0 });
        idx
    }

    /// Emit a conditional jump (JumpIfNotTrue). Returns the index so it can be patched.
    pub fn emit_jump_if_not_true(&mut self, src: u16) -> usize {
        let idx = self.insts.len();
        self.insts.push(Inst::JumpIfNotTrue { src, target: 0 });
        idx
    }

    /// Emit an EmitRow instruction
    pub fn emit_emit_row(&mut self) {
        self.insts.push(Inst::EmitRow);
    }

    /// Emit a Halt instruction
    pub fn emit_halt(&mut self) {
        self.insts.push(Inst::Halt);
    }

    /// Emit a DecrOrJump instruction. Returns the index so it can be patched.
    pub fn emit_decr_or_jump(&mut self, counter_reg: u16) -> usize {
        let idx = self.insts.len();
        self.insts.push(Inst::DecrOrJump {
            counter_reg,
            target: 0,
        });
        idx
    }

    /// Patch a previously emitted instruction's jump target.
    /// Works for NextRow (eof_target), Jump, JumpIfNotTrue, DecrOrJump.
    pub fn patch_target(&mut self, inst_idx: usize, target: u32) {
        match &mut self.insts[inst_idx] {
            Inst::NextRow { eof_target, .. } => *eof_target = target,
            Inst::Jump { target: t } => *t = target,
            Inst::JumpIfNotTrue { target: t, .. } => *t = target,
            Inst::DecrOrJump { target: t, .. } => *t = target,
            _ => panic!("patch_target called on non-patchable instruction"),
        }
    }

    /// Get the current instruction count (next instruction index)
    pub fn current_offset(&self) -> u32 {
        self.insts.len() as u32
    }

    // === Public accessors for inline_program support ===

    /// Allocate a register (public for compiler use).
    pub fn alloc_reg_pub(&mut self) -> u16 {
        self.alloc_reg()
    }

    /// Push a constant and return its index (public for compiler use).
    pub fn push_const_pub(&mut self, value: ValueOwned) -> u16 {
        self.push_const(value)
    }

    /// Push a key and return its index (public for compiler use).
    pub fn push_key_pub(&mut self, key: String) -> u16 {
        self.intern_key(key)
    }

    /// Get the number of constants currently in the pool.
    pub fn consts_len(&self) -> usize {
        self.consts.len()
    }

    /// Get the number of keys currently in the pool.
    pub fn keys_len(&self) -> usize {
        self.keys.len()
    }

    /// Update next_reg to at least the given value (for merging sub-programs).
    pub fn update_next_reg(&mut self, reg_count: u16) {
        if reg_count > self.next_reg {
            self.next_reg = reg_count;
        }
    }

    pub fn build(self) -> Program {
        // Create arena for tuple field arrays
        let arena = Arena::default();

        // Pre-convert all constants to ValueRef
        let const_refs: Vec<ValueRef<'_>> = self
            .consts
            .iter()
            .map(|c| ValueRef::from_owned(c, &arena))
            .collect();

        // Safety: This transmute extends the lifetime from the temporary borrow to 'static.
        // This is safe because:
        // 1. Program owns both `consts` (Vec<ValueOwned>) and `arena`
        // 2. ValueRef references point into either:
        //    - The owned data in `consts` (for String/Bytes via .as_str()/.as_slice())
        //    - The arena (for tuple field arrays)
        // 3. Both live for the entire lifetime of Program
        // 4. They're dropped together when Program is dropped
        let const_refs: Vec<ValueRef<'static>> = unsafe { std::mem::transmute(const_refs) };

        Program {
            insts: self.insts,
            consts: self.consts,
            arena,
            const_refs,
            keys: self.keys,
            reg_count: self.next_reg,
            slot_count: self.slot_count,
        }
    }

    fn alloc_reg(&mut self) -> u16 {
        let reg = self.next_reg;
        self.next_reg = self.next_reg.checked_add(1).expect("register overflow");
        reg
    }

    fn push_const(&mut self, value: ValueOwned) -> u16 {
        self.consts.push(value);
        (self.consts.len() - 1) as u16
    }

    fn intern_key(&mut self, key: String) -> u16 {
        if let Some((idx, _)) = self.keys.iter().enumerate().find(|(_, k)| *k == &key) {
            return idx as u16;
        }
        self.keys.push(key);
        (self.keys.len() - 1) as u16
    }
}

pub struct ExprCompiler {
    builder: ProgramBuilder,
}

impl ExprCompiler {
    pub fn new(slot_count: u16) -> Self {
        ExprCompiler {
            builder: ProgramBuilder::new(slot_count),
        }
    }

    pub fn compile_expr(&mut self, expr: &Expr) -> Result<u16> {
        match expr {
            Expr::Literal(value) => {
                let reg = self.builder.alloc_reg();
                let const_idx = self.builder.push_const(value.clone());
                self.builder.insts.push(Inst::LoadConst {
                    dst: reg,
                    const_idx,
                });
                Ok(reg)
            }
            Expr::SlotRef(slot) => {
                // Slots are already in registers at indices [0..slot_count]
                // No LoadSlot instruction needed!
                Ok(*slot)
            }
            Expr::Add(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                let l_cast = self.builder.alloc_reg();
                let r_cast = self.builder.alloc_reg();
                self.builder.insts.push(Inst::AddDynamic {
                    dst,
                    a: l,
                    b: r,
                    a_cast: l_cast,
                    b_cast: r_cast,
                });
                Ok(dst)
            }
            Expr::Sub(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                let l_cast = self.builder.alloc_reg();
                let r_cast = self.builder.alloc_reg();
                self.builder.insts.push(Inst::SubDynamic {
                    dst,
                    a: l,
                    b: r,
                    a_cast: l_cast,
                    b_cast: r_cast,
                });
                Ok(dst)
            }
            Expr::Mod(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                let l_cast = self.builder.alloc_reg();
                let r_cast = self.builder.alloc_reg();
                self.builder.insts.push(Inst::ModDynamic {
                    dst,
                    a: l,
                    b: r,
                    a_cast: l_cast,
                    b_cast: r_cast,
                });
                Ok(dst)
            }
            Expr::Eq(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                let l_cast = self.builder.alloc_reg();
                let r_cast = self.builder.alloc_reg();
                self.builder.insts.push(Inst::EqDynamic {
                    dst,
                    a: l,
                    b: r,
                    a_cast: l_cast,
                    b_cast: r_cast,
                });
                Ok(dst)
            }
            Expr::And(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::AndBool { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::Or(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::OrBool { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::Not(expr) => {
                let src = self.compile_expr(expr)?;
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::NotBool { dst, src });
                Ok(dst)
            }
            Expr::Neg(expr) => {
                let src = self.compile_expr(expr)?;
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::NegNum { dst, src });
                Ok(dst)
            }
            Expr::Pos(expr) => {
                // Unary + is a no-op on numeric values
                self.compile_expr(expr)
            }
            Expr::Like {
                value,
                pattern,
                escape,
            } => {
                let value_reg = self.compile_expr(value)?;
                let dst = self.builder.alloc_reg();
                let regex_str = like_to_re_pattern(pattern, escape);
                let pattern_idx = self.builder.intern_key(regex_str);
                self.builder.insts.push(Inst::LikeMatch {
                    dst,
                    value: value_reg,
                    pattern_idx,
                });
                Ok(dst)
            }
            Expr::LikeDynamic {
                value,
                pattern,
                escape,
            } => {
                let value_reg = self.compile_expr(value)?;
                let pattern_reg = self.compile_expr(pattern)?;
                let escape_reg = self.compile_expr(escape)?;
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::LikeDynamicMatch {
                    dst,
                    value: value_reg,
                    pattern: pattern_reg,
                    escape: escape_reg,
                });
                Ok(dst)
            }
            Expr::GetField(base, key) => {
                let base_reg = self.compile_expr(base)?;
                let dst = self.builder.alloc_reg();
                let key_idx = self.builder.intern_key(key.clone());
                self.builder.insts.push(Inst::GetField {
                    dst,
                    base: base_reg,
                    key_idx,
                });
                Ok(dst)
            }
            Expr::UdfCall { name, args } => {
                let mut arg_regs = Vec::with_capacity(args.len());
                for arg in args {
                    arg_regs.push(self.compile_expr(arg)?);
                }
                let dst = self.builder.alloc_reg();
                let func_idx = self.builder.intern_key(name.clone());
                self.builder.insts.push(Inst::CallUdf {
                    dst,
                    func_idx,
                    args: arg_regs,
                });
                Ok(dst)
            }
            Expr::Tuple { attrs, values } => {
                // Compile attribute expressions
                let mut attr_regs = Vec::with_capacity(attrs.len());
                for attr in attrs {
                    attr_regs.push(self.compile_expr(attr)?);
                }
                // Compile value expressions
                let mut value_regs = Vec::with_capacity(values.len());
                for value in values {
                    value_regs.push(self.compile_expr(value)?);
                }
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::MakeTuple {
                    dst,
                    attr_regs,
                    value_regs,
                });
                Ok(dst)
            }
            Expr::List { elements } => {
                let mut elements_regs = Vec::with_capacity(elements.len());
                for element in elements {
                    elements_regs.push(self.compile_expr(element)?);
                }
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::MakeList {
                    dst,
                    element_regs: elements_regs,
                });
                Ok(dst)
            }
            Expr::Mul(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                let l_cast = self.builder.alloc_reg();
                let r_cast = self.builder.alloc_reg();
                self.builder.insts.push(Inst::MulDynamic {
                    dst,
                    a: l,
                    b: r,
                    a_cast: l_cast,
                    b_cast: r_cast,
                });
                Ok(dst)
            }
            Expr::Div(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                let l_cast = self.builder.alloc_reg();
                let r_cast = self.builder.alloc_reg();
                self.builder.insts.push(Inst::DivDynamic {
                    dst,
                    a: l,
                    b: r,
                    a_cast: l_cast,
                    b_cast: r_cast,
                });
                Ok(dst)
            }
            Expr::Exp(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                let l_cast = self.builder.alloc_reg();
                let r_cast = self.builder.alloc_reg();
                self.builder.insts.push(Inst::ExpDynamic {
                    dst,
                    a: l,
                    b: r,
                    a_cast: l_cast,
                    b_cast: r_cast,
                });
                Ok(dst)
            }
            Expr::Gt(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                let l_cast = self.builder.alloc_reg();
                let r_cast = self.builder.alloc_reg();
                self.builder.insts.push(Inst::GtDynamic {
                    dst,
                    a: l,
                    b: r,
                    a_cast: l_cast,
                    b_cast: r_cast,
                });
                Ok(dst)
            }
            Expr::Lt(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                let l_cast = self.builder.alloc_reg();
                let r_cast = self.builder.alloc_reg();
                self.builder.insts.push(Inst::LtDynamic {
                    dst,
                    a: l,
                    b: r,
                    a_cast: l_cast,
                    b_cast: r_cast,
                });
                Ok(dst)
            }
            Expr::Concat(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                self.builder
                    .insts
                    .push(Inst::ConcatDynamic { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::In(left, right) => {
                let value = self.compile_expr(left)?;
                let collection = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::InDynamic {
                    dst,
                    value,
                    collection,
                });
                Ok(dst)
            }
            Expr::Bag { elements } => {
                let mut elements_regs = Vec::with_capacity(elements.len());
                for element in elements {
                    elements_regs.push(self.compile_expr(element)?);
                }
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::MakeBag {
                    dst,
                    element_regs: elements_regs,
                });
                Ok(dst)
            }
        }
    }

    pub fn compile_to_slot(&mut self, expr: &Expr, slot: SlotId) -> Result<()> {
        let reg = self.compile_expr(expr)?;
        self.builder.insts.push(Inst::StoreSlot { slot, src: reg });
        Ok(())
    }

    pub fn finish(self) -> Program {
        self.builder.build()
    }
}

pub struct LogicalExprCompiler<'a, R: SlotResolver> {
    resolver: &'a R,
}

impl<'a, R: SlotResolver> LogicalExprCompiler<'a, R> {
    pub fn new(resolver: &'a R) -> Self {
        LogicalExprCompiler { resolver }
    }

    pub fn compile_to_program(
        &self,
        expr: &ValueExpr,
        slot: SlotId,
        slot_count: u16,
    ) -> Result<Program> {
        let expr = self.lower_expr(expr)?;
        let mut compiler = ExprCompiler::new(slot_count);
        compiler.compile_to_slot(&expr, slot)?;
        Ok(compiler.finish())
    }

    pub fn compile_to_program_multi(
        &self,
        exprs: &[(SlotId, ValueExpr)],
        slot_count: u16,
    ) -> Result<Program> {
        let mut compiler = ExprCompiler::new(slot_count);
        for (slot, expr) in exprs {
            let lowered = self.lower_expr(expr)?;
            compiler.compile_to_slot(&lowered, *slot)?;
        }
        Ok(compiler.finish())
    }

    fn lower_expr(&self, expr: &ValueExpr) -> Result<Expr> {
        match expr {
            ValueExpr::Lit(lit) => Ok(Expr::Literal(lit_to_value(lit)?)),
            ValueExpr::VarRef(name, scope) => self
                .resolver
                .resolve_var(name, scope.clone())
                .map(Expr::SlotRef)
                .ok_or_else(|| {
                    EngineError::UnsupportedExpr(format!(
                        "unresolved var {name:?} with scope {scope:?}"
                    ))
                }),
            ValueExpr::DBRef(db_ref) => {
                // DBRef represents a catalog-registered database object (table/view)
                // Resolve the first component of the path as a global variable
                if let Some(name) = db_ref.path.first() {
                    self.resolver
                        .resolve_var(name, VarRefType::Global)
                        .map(Expr::SlotRef)
                        .ok_or_else(|| {
                            EngineError::UnsupportedExpr(format!(
                                "unresolved DB object {}.{}",
                                db_ref.catalog,
                                bindings_name_to_string(name)
                            ))
                        })
                } else {
                    Err(EngineError::UnsupportedExpr(
                        "DBRef with empty path".to_string(),
                    ))
                }
            }
            ValueExpr::DynamicLookup(lookups) => {
                for lookup in lookups.iter() {
                    if let Ok(expr) = self.lower_expr(lookup) {
                        return Ok(expr);
                    }
                }
                Err(EngineError::UnsupportedExpr("dynamic lookup".to_string()))
            }
            ValueExpr::Path(base, components) => {
                // First, try to resolve the first component directly as a field
                if let Some(PathComponent::Key(name)) = components.first() {
                    if let Some(slot) = self.resolver.resolve_field(name) {
                        return Ok(Expr::SlotRef(slot));
                    }
                }

                // Check if base is an alias with a resolvable slot
                if let Some((is_alias, base_slot)) = resolve_alias_info(self.resolver, base) {
                    if is_alias {
                        if let Some(base_slot) = base_slot {
                            // We have a resolved base slot - build GetField chain
                            let mut current = Expr::SlotRef(base_slot);
                            for component in components {
                                match component {
                                    PathComponent::Key(name) => {
                                        current = Expr::GetField(
                                            current.into(),
                                            bindings_name_to_string(name),
                                        );
                                    }
                                    _ => {
                                        return Err(EngineError::UnsupportedExpr(format!(
                                            "unsupported path component: {:?}",
                                            component
                                        )));
                                    }
                                }
                            }
                            return Ok(current);
                        }
                        // is_alias=true but base_slot=None means VarRef(Local) that can't be resolved yet
                        // This can happen for table names in FROM clause - fall through to generic handling
                    }
                }

                // Generic path handling - lower the base expression and build GetField chain
                let mut current = self.lower_expr(base)?;
                for component in components {
                    match component {
                        PathComponent::Key(name) => {
                            current = Expr::GetField(current.into(), bindings_name_to_string(name));
                        }
                        _ => {
                            return Err(EngineError::UnsupportedExpr(format!(
                                "unsupported path component: {:?}",
                                component
                            )));
                        }
                    }
                }
                Ok(current)
            }
            ValueExpr::BinaryExpr(op, left, right) => {
                let left = self.lower_expr(left)?;
                let right = self.lower_expr(right)?;
                match op {
                    partiql_logical::BinaryOp::Add => Ok(Expr::Add(left.into(), right.into())),
                    partiql_logical::BinaryOp::Sub => Ok(Expr::Sub(left.into(), right.into())),
                    partiql_logical::BinaryOp::Mod => Ok(Expr::Mod(left.into(), right.into())),
                    partiql_logical::BinaryOp::Neq => {
                        Ok(Expr::Not(Box::new(Expr::Eq(left.into(), right.into()))))
                    }
                    partiql_logical::BinaryOp::Eq => Ok(Expr::Eq(left.into(), right.into())),
                    partiql_logical::BinaryOp::And => Ok(Expr::And(left.into(), right.into())),
                    partiql_logical::BinaryOp::Or => Ok(Expr::Or(left.into(), right.into())),
                    partiql_logical::BinaryOp::Concat => {
                        Ok(Expr::Concat(left.into(), right.into()))
                    }
                    partiql_logical::BinaryOp::Gt => Ok(Expr::Gt(left.into(), right.into())),
                    partiql_logical::BinaryOp::Gteq => {
                        // Gteq(l, r) => Or(Gt(l, r), Eq(l', r'))
                        // We clone left/right since they're used twice
                        let left_clone = left.clone();
                        let right_clone = right.clone();
                        Ok(Expr::Or(
                            Box::new(Expr::Gt(left.into(), right.into())),
                            Box::new(Expr::Eq(left_clone.into(), right_clone.into())),
                        ))
                    }
                    partiql_logical::BinaryOp::Lt => Ok(Expr::Lt(left.into(), right.into())),
                    partiql_logical::BinaryOp::Lteq => {
                        // Lteq(l, r) => Or(Lt(l, r), Eq(l', r'))
                        let left_clone = left.clone();
                        let right_clone = right.clone();
                        Ok(Expr::Or(
                            Box::new(Expr::Lt(left.into(), right.into())),
                            Box::new(Expr::Eq(left_clone.into(), right_clone.into())),
                        ))
                    }
                    partiql_logical::BinaryOp::Mul => Ok(Expr::Mul(left.into(), right.into())),
                    partiql_logical::BinaryOp::Div => Ok(Expr::Div(left.into(), right.into())),
                    partiql_logical::BinaryOp::Exp => Ok(Expr::Exp(left.into(), right.into())),
                    partiql_logical::BinaryOp::In => Ok(Expr::In(left.into(), right.into())),
                }
            }
            ValueExpr::UnExpr(op, expr) => {
                let expr = self.lower_expr(expr)?;
                match op {
                    partiql_logical::UnaryOp::Not => Ok(Expr::Not(expr.into())),
                    partiql_logical::UnaryOp::Neg => Ok(Expr::Neg(expr.into())),
                    partiql_logical::UnaryOp::Pos => Ok(Expr::Pos(expr.into())),
                }
            }
            ValueExpr::Call(call) => Ok(Expr::UdfCall {
                name: call_name(call),
                args: call
                    .arguments
                    .iter()
                    .map(|arg| self.lower_expr(arg))
                    .collect::<Result<Vec<_>>>()?,
            }),
            ValueExpr::ListExpr(list_expr) => {
                let elements = list_expr
                    .elements
                    .iter()
                    .map(|e| self.lower_expr(e))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Expr::List { elements })
            }
            ValueExpr::BagExpr(bag_expr) => {
                let elements = bag_expr
                    .elements
                    .iter()
                    .map(|e| self.lower_expr(e))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Expr::Bag { elements })
            }
            ValueExpr::TupleExpr(tuple_expr) => {
                let attrs = tuple_expr
                    .attrs
                    .iter()
                    .map(|attr| self.lower_expr(attr))
                    .collect::<Result<Vec<_>>>()?;
                let values = tuple_expr
                    .values
                    .iter()
                    .map(|value| self.lower_expr(value))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Expr::Tuple { attrs, values })
            }
            ValueExpr::BetweenExpr(_between_expr) => {
                Err(EngineError::UnsupportedExpr(format!("{:?}", *expr)))
            }
            ValueExpr::PatternMatchExpr(pm) => {
                let value = self.lower_expr(&pm.value)?;
                match &pm.pattern {
                    partiql_logical::Pattern::Like(like) => Ok(Expr::Like {
                        value: value.into(),
                        pattern: like.pattern.clone(),
                        escape: like.escape.clone(),
                    }),
                    partiql_logical::Pattern::LikeNonStringNonLiteral(like) => {
                        let pattern = self.lower_expr(&like.pattern)?;
                        let escape = self.lower_expr(&like.escape)?;
                        Ok(Expr::LikeDynamic {
                            value: value.into(),
                            pattern: pattern.into(),
                            escape: escape.into(),
                        })
                    }
                }
            }
            ValueExpr::SubQueryExpr(_sub_query_expr) => {
                Err(EngineError::UnsupportedExpr(format!("{:?}", *expr)))
            }
            ValueExpr::SimpleCase(_simple_case) => {
                Err(EngineError::UnsupportedExpr(format!("{:?}", *expr)))
            }
            ValueExpr::SearchedCase(_searched_case) => {
                Err(EngineError::UnsupportedExpr(format!("{:?}", *expr)))
            }
            ValueExpr::IsTypeExpr(_is_type_expr) => {
                Err(EngineError::UnsupportedExpr(format!("{:?}", *expr)))
            }
            ValueExpr::NullIfExpr(_null_if_expr) => {
                Err(EngineError::UnsupportedExpr(format!("{:?}", *expr)))
            }
            ValueExpr::CoalesceExpr(_coalesce_expr) => {
                Err(EngineError::UnsupportedExpr(format!("{:?}", *expr)))
            }
            ValueExpr::GraphMatch(_graph_match_expr) => {
                Err(EngineError::UnsupportedExpr(format!("{:?}", *expr)))
            }
        }
    }
}

pub(crate) fn lit_to_value(lit: &Lit) -> Result<ValueOwned> {
    Ok(match lit {
        Lit::Missing => ValueOwned::Missing,
        Lit::Null => ValueOwned::Null,
        Lit::Int64(v) => ValueOwned::I64(*v),
        Lit::Bool(v) => ValueOwned::Bool(*v),
        Lit::String(v) => ValueOwned::String(v.clone()),
        Lit::Struct(fields) => {
            // Recursively convert each field in the struct
            let tuple_fields = fields
                .iter()
                .map(|(name, field_lit)| {
                    Ok(super::value::TupleFieldOwned {
                        name: name.clone(),
                        value: lit_to_value(field_lit)?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            ValueOwned::Tuple(super::value::TupleOwned {
                fields: tuple_fields,
            })
        }
        Lit::List(elements) => {
            // Recursively convert each element in the list
            let items = elements
                .iter()
                .map(lit_to_value)
                .collect::<Result<Vec<_>>>()?;
            ValueOwned::List(items)
        }
        Lit::Bag(elements) => {
            // Recursively convert each element in the bag
            let items = elements
                .iter()
                .map(lit_to_value)
                .collect::<Result<Vec<_>>>()?;
            ValueOwned::Bag(items)
        }
        Lit::Int8(v) => ValueOwned::I64((*v).into()),
        Lit::Int16(v) => ValueOwned::I64((*v).into()),
        Lit::Int32(v) => ValueOwned::I64((*v).into()),
        Lit::Decimal(d) => ValueOwned::Decimal(*d),
        Lit::Double(f) => ValueOwned::F64(*f),
        Lit::Variant(_, _) => todo!("Variant literals are not (yet) supported."),
    })
}

fn call_name(call: &CallExpr) -> String {
    match &call.name {
        CallName::ByName(name) => name.clone(),
        CallName::ById(name, _, _) => name.clone(),
        other => format!("{other:?}"),
    }
}

fn like_to_re_pattern(like_expr: &str, escape: &str) -> String {
    let escape_ch = escape.chars().next();
    let mut pattern = String::from("^");
    pattern.reserve(like_expr.len() + 6);
    let mut escaped = false;
    let mut wildcard = false;
    for ch in like_expr.chars() {
        let is_any = std::mem::replace(&mut wildcard, false);
        let is_escaped = std::mem::replace(&mut escaped, false);
        match (ch, is_escaped) {
            (_, false) if Some(ch) == escape_ch => escaped = true,
            ('%', false) => {
                if !is_any {
                    pattern.push_str(".*?");
                }
                wildcard = true;
            }
            ('_', false) => pattern.push('.'),
            _ => {
                if regex_syntax::is_meta_character(ch) {
                    pattern.push('\\');
                }
                pattern.push(ch);
            }
        }
    }
    pattern.push('$');
    pattern
}

fn bindings_name_to_string(name: &BindingsName<'_>) -> String {
    match name {
        BindingsName::CaseSensitive(s) => s.to_string(),
        BindingsName::CaseInsensitive(s) => s.to_string(),
    }
}

fn resolve_alias_info<R: SlotResolver>(
    resolver: &R,
    base: &ValueExpr,
) -> Option<(bool, Option<SlotId>)> {
    match base {
        ValueExpr::VarRef(name, scope) => {
            // For Local scope VarRefs, treat them as aliases
            // This handles cases like "FROM data" where "data" is both table and alias
            if *scope == VarRefType::Local {
                Some((true, resolver.resolve_var(name, scope.clone())))
            } else if resolver.is_alias(name) {
                Some((true, resolver.resolve_alias(name)))
            } else {
                Some((false, None))
            }
        }
        ValueExpr::DynamicLookup(lookups) => lookups.iter().find_map(|lookup| {
            if let ValueExpr::VarRef(name, scope) = lookup {
                if *scope == VarRefType::Local {
                    Some((true, resolver.resolve_var(name, scope.clone())))
                } else if resolver.is_alias(name) {
                    Some((true, resolver.resolve_alias(name)))
                } else {
                    None
                }
            } else {
                None
            }
        }),
        _ => None,
    }
}

/// Shallow structural equality for ValueRef values.
///
/// Used by the `IN` operator to check membership in collections.
/// Returns `false` for Null/Missing comparisons (following SQL semantics).
fn value_ref_eq(a: ValueRef<'_>, b: ValueRef<'_>) -> bool {
    match (a, b) {
        (ValueRef::Null, ValueRef::Null) => false,
        (ValueRef::Missing, _) | (_, ValueRef::Missing) => false,
        (ValueRef::Null, _) | (_, ValueRef::Null) => false,
        (ValueRef::Bool(a), ValueRef::Bool(b)) => a == b,
        (ValueRef::I64(a), ValueRef::I64(b)) => a == b,
        (ValueRef::F64(a), ValueRef::F64(b)) => a == b,
        (ValueRef::Decimal(a), ValueRef::Decimal(b)) => a == b,
        (ValueRef::I64(a), ValueRef::F64(b)) => (a as f64) == b,
        (ValueRef::F64(a), ValueRef::I64(b)) => a == (b as f64),
        (ValueRef::I64(a), ValueRef::Decimal(b)) => Decimal::new(a, 0) == b,
        (ValueRef::Decimal(a), ValueRef::I64(b)) => a == Decimal::new(b, 0),
        (ValueRef::F64(a), ValueRef::Decimal(b)) => a == b.to_f64().unwrap_or(f64::NAN),
        (ValueRef::Decimal(a), ValueRef::F64(b)) => a.to_f64().unwrap_or(f64::NAN) == b,
        (ValueRef::Str(a), ValueRef::Str(b)) => a == b,
        (ValueRef::Bytes(a), ValueRef::Bytes(b)) => a == b,
        _ => false,
    }
}
