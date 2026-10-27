use crate::arena::{Arena, SlotId};
use crate::compare;
use crate::error::{count_u16, EngineError, Result};
use crate::value::{value_get_field_ref, variant, ValueOwned, ValueRef};
use partiql_logical::{
    CallExpr, CallName, Lit, PathComponent, SubQueryExpr, ValueExpr, VarRefType,
};
use partiql_value::BindingsName;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use rustc_hash::FxHashMap;
use std::cmp::Ordering;

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
    IsNull(Box<Expr>),
    IsMissing(Box<Expr>),
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
    /// The value of subquery `id` (an index into the query's subqueries), which the emitter
    /// evaluates into register `dst` before the expression that reads it.
    Subquery {
        id: u32,
        dst: SlotId,
    },
    /// COLL_TO_SCALAR: the one attribute of the one tuple in a collection, else MISSING.
    CollToScalar(Box<Expr>),
    /// NULLIF(a, b): NULL when `a = b` is true, else `a`.
    NullIf(Box<Expr>, Box<Expr>),
    /// COALESCE: the first value that is neither NULL nor MISSING, else NULL. Each argument
    /// is evaluated only if those before it are absent; the subqueries an argument reads,
    /// though, run with the expression's others, before any of its code.
    Coalesce(Vec<Expr>),
    /// `v BETWEEN lo AND hi`, each evaluated once.
    Between(Box<Expr>, Box<Expr>, Box<Expr>),
}

impl Expr {
    /// Call `f` with the id of each subquery in the expression, in evaluation order.
    pub(crate) fn for_each_subquery(&self, f: &mut dyn FnMut(u32)) {
        match self {
            Expr::Subquery { id, .. } => f(*id),
            Expr::Literal(_) | Expr::SlotRef(_) => {}
            Expr::Like { value, .. } => value.for_each_subquery(f),
            Expr::Add(a, b)
            | Expr::Sub(a, b)
            | Expr::Mul(a, b)
            | Expr::Div(a, b)
            | Expr::Mod(a, b)
            | Expr::Exp(a, b)
            | Expr::Eq(a, b)
            | Expr::Gt(a, b)
            | Expr::Lt(a, b)
            | Expr::And(a, b)
            | Expr::Or(a, b)
            | Expr::Concat(a, b)
            | Expr::In(a, b) => {
                a.for_each_subquery(f);
                b.for_each_subquery(f);
            }
            Expr::Not(e)
            | Expr::Neg(e)
            | Expr::Pos(e)
            | Expr::GetField(e, _)
            | Expr::CollToScalar(e)
            | Expr::IsNull(e)
            | Expr::IsMissing(e) => e.for_each_subquery(f),
            Expr::NullIf(a, b) => {
                a.for_each_subquery(f);
                b.for_each_subquery(f);
            }
            Expr::Coalesce(exprs) => exprs.iter().for_each(|e| e.for_each_subquery(f)),
            Expr::Between(v, lo, hi) => {
                for e in [v, lo, hi] {
                    e.for_each_subquery(f);
                }
            }
            Expr::LikeDynamic {
                value,
                pattern,
                escape,
            } => {
                for e in [value, pattern, escape] {
                    e.for_each_subquery(f);
                }
            }
            Expr::UdfCall { args: exprs, .. }
            | Expr::List { elements: exprs }
            | Expr::Bag { elements: exprs } => {
                exprs.iter().for_each(|e| e.for_each_subquery(f));
            }
            Expr::Tuple { attrs, values } => {
                attrs
                    .iter()
                    .chain(values)
                    .for_each(|e| e.for_each_subquery(f));
            }
        }
    }
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
    /// PartiQL's `=` of any two values (`compare::eq`).
    EqDynamic {
        dst: u16,
        a: u16,
        b: u16,
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
    /// `a > b` of any two values (`eval_ordering_dynamic`).
    GtDynamic {
        dst: u16,
        a: u16,
        b: u16,
    },
    // --- Lt ---
    /// `a < b` of any two values (`eval_ordering_dynamic`).
    LtDynamic {
        dst: u16,
        a: u16,
        b: u16,
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
    /// Copy `src` into `dst`, MISSING becoming NULL.
    MissingToNull {
        dst: u16,
        src: u16,
    },
    /// NULLIF: NULL when `eq` holds true, else the value in `value`.
    NullIf {
        dst: u16,
        value: u16,
        eq: u16,
    },
    /// One argument of COALESCE: if `src` is neither NULL nor MISSING, copy it into `dst` and
    /// skip the next `skip` instructions (the arguments after it). Handled in the VM loop.
    Coalesce {
        dst: u16,
        src: u16,
        skip: u32,
    },
    /// COLL_TO_SCALAR (spec 9.1): a collection of one tuple with one attribute becomes that
    /// attribute's value; anything else becomes MISSING.
    CollToScalar {
        dst: u16,
        src: u16,
    },
    IsNull {
        dst: u16,
        src: u16,
    },
    IsMissing {
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
    /// Bind cursor `cursor_id` to the elements of the collection in `src`, or to the value
    /// as `coercion` reads one that is no collection.
    MaterializeCursor {
        src: u16,
        cursor_id: u16,
        coercion: Coercion,
    },
    /// Create a table function cursor from evaluated arguments in registers.
    ///
    /// Looks up the table function by name (from the keys pool at `func_name_idx`),
    /// reads arguments from the register bank using the slot mapping stored in
    /// `TableFnScanMetadata`, calls `create()` to produce a `DataSource`, and
    /// stores it in the cursor slot. The subsequent `OpenCursor` opens it.
    CreateTableFnCursor {
        cursor_id: u16,
        func_name_idx: u16,
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

    /// If src is already a Tuple, copy to dst. Otherwise wrap as {'key': value}.
    /// const_idx indexes into the Program's const pool for the field name string.
    CoerceToTuple {
        dst: u16,
        src: u16,
        const_idx: u16,
    },

    /// Build one tuple from the values in `srcs`, in order: `SELECT *` over a join, where
    /// each register holds one FROM binding's row (as the legacy evaluator's
    /// `coerce_into_tuple` + concatenation).
    ///
    /// Each source contributes:
    /// - a tuple: all of its fields, in order (duplicate names are kept);
    /// - MISSING: nothing;
    /// - anything else (including the NULL of an outer join's padded side): one field
    ///   named `_1` holding the value.
    ///
    /// The result is allocated in the row's arena.
    ConcatTuples {
        /// Register receiving the new tuple.
        dst: u16,
        /// Registers holding the bindings' values, in FROM order.
        srcs: Vec<u16>,
    },

    /// Strict mode: assert src is a List or Bag. If not, runtime error.
    /// Handled in the VM dispatch loop (relational instruction).
    AssertCollection {
        src: u16,
        coercion: Coercion,
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
    /// Used for LIMIT: with the counter loaded with the count, once before the first row
    /// and then after each row.
    DecrOrJump {
        counter_reg: u16,
        target: u32,
    },

    /// Used for OFFSET: if the i64 counter in `counter_reg` is above 0, decrement it and jump
    /// to `target` (skipping the row); else fall through.
    SkipRow {
        counter_reg: u16,
        target: u32,
    },

    /// A LIMIT or OFFSET count: leave a non-negative integer in `reg`. Replace NULL or
    /// MISSING with `otherwise`, and anything else too, except in strict mode, where it fails.
    RowCount {
        reg: u16,
        strict: bool,
        otherwise: i64,
    },

    // === Sorter Instructions (GROUP BY, ORDER BY) ===
    /// Drop the sorter's records from a previous run and free its bank. Values read out
    /// of the sorter (by `SorterData`) are invalid afterwards: the operator must write a
    /// register before reading it again.
    SorterReset {
        sorter_id: u16,
    },

    /// Copy registers `src_reg..src_reg+reg_count` from the row bank into the
    /// sorter's persistent bank, appending a new record.
    SorterInsert {
        sorter_id: u16,
        src_reg: u16,
        reg_count: u16,
    },

    /// Sort all records in the sorter by their key fields (see `SortKeys`).
    /// If the sorter is empty, jump to `eof_target`.
    SorterSort {
        sorter_id: u16,
        eof_target: u32,
    },

    /// Write the current record's fields into registers starting at `dst_reg`.
    /// Zero-copy: registers point directly into the sorter's bank.
    SorterData {
        sorter_id: u16,
        dst_reg: u16,
    },

    /// GROUP AS: make field `field` of the records of the group that just ended (from its
    /// first record up to the current one, or to the end once the sorter is exhausted) into a
    /// bag in `dst`, in input order. It lives in the sorter's bank, as `SorterData`'s values do.
    SorterGroupBag {
        sorter_id: u16,
        field: u16,
        dst: u16,
    },

    /// GROUP BY: write to `dst` whether the current record starts a group (its keys are not
    /// `eqg` to the previous record's).
    SorterNewGroup {
        sorter_id: u16,
        dst: u16,
    },

    /// Advance to the next record. If exhausted, fall through; otherwise jump to `target`.
    SorterNext {
        sorter_id: u16,
        target: u32,
    },

    /// Start a subquery: until the matching `ScratchLeave`, `NextRow` rewinds the scratch
    /// bank only to where it is now, so the temporaries of the expressions around the
    /// subquery survive its scans.
    ScratchEnter,
    /// End a subquery: free its temporaries and restore the previous floor.
    ScratchLeave,

    /// Make the sorter's records, in insertion order, into a collection in `dst` (a list
    /// when `list`, else a bag) that lives in the sorter's bank. With `fields`, each record
    /// becomes a tuple whose attributes are named by those keys (MISSING values are left
    /// out); without, each record is its one value.
    SorterCollect {
        sorter_id: u16,
        dst: u16,
        fields: Option<Vec<u16>>,
        list: bool,
    },

    // === Hash Table Instructions (DISTINCT, set operators) ===
    /// Drop the table's keys from a previous run and free its bank.
    HashReset {
        table: u16,
    },

    /// Add the key in registers `key` to the table unless it holds one `eqg` to it. Writes
    /// whether it was new to `new`.
    HashInsert {
        table: u16,
        key: Vec<u16>,
        new: u16,
    },

    /// Count the key in registers `key` once more, adding it if new.
    HashAdd {
        table: u16,
        key: Vec<u16>,
    },

    /// If the table counts the key in registers `key`, count it once less. Writes whether it
    /// did to `found`.
    HashTake {
        table: u16,
        key: Vec<u16>,
        found: u16,
    },

    // === Aggregation Instructions ===
    /// Update accumulator register with a new input value.
    /// For Count, `input_reg` is ignored (counts rows, not values).
    AggStep {
        func: AggFunc,
        accum_reg: u16,
        input_reg: u16,
    },

    /// Finalize the accumulator and write the result to `dst_reg`.
    AggFinal {
        func: AggFunc,
        accum_reg: u16,
        dst_reg: u16,
    },

    /// Reset accumulator registers `accum_start..accum_start+count` to Missing.
    AggReset {
        accum_start: u16,
        count: u16,
    },

    // === Subroutine Instructions ===
    /// Save the next instruction address into `ret_reg`, then jump to `target`.
    Gosub {
        ret_reg: u16,
        target: u32,
    },

    /// Jump to the address stored in `ret_reg`.
    Return {
        ret_reg: u16,
    },

    // === Register Operations ===
    /// Copy the value in `src` to `dst`.
    Copy {
        dst: u16,
        src: u16,
    },
}

/// What a collection a cursor scans is for, which decides how a value that is no collection
/// is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coercion {
    /// A FROM source: a value that is no collection is its one row.
    From,
    /// An operand of OUTER UNION, INTERSECT or EXCEPT (RFC-0007): NULL and MISSING are no
    /// rows, and any other value that is no collection is its one row.
    SetOp,
}

/// Aggregate function type used by AggStep and AggFinal instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggFunc {
    Sum,
    Count,
    Avg,
    Min,
    Max,
    Any,
    Every,
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

    // --- Lt constructors ---
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
        // An unknown operand makes the result unknown: MISSING over NULL
        if is_null_or_missing(av) || is_null_or_missing(bv) {
            regs[dst as usize] = match is_missing(av) || is_missing(bv) {
                true => ValueRef::Missing,
                false => ValueRef::Null,
            };
            return Ok(());
        }
        // Mixed numbers widen: an int to the other operand's type; a float with a decimal to
        // a decimal (as in the legacy evaluator)
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
            (ValueRef::F64(_), ValueRef::I64(_)) => {
                let cast = Inst::Cast {
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::F64,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                self.eval_inst(&fn_f64(dst, lhs, rhs_cast), arena, regs, udf)?
            }
            (ValueRef::F64(_), ValueRef::Decimal(_)) => {
                let cast = Inst::Cast {
                    dst: lhs_cast,
                    from: lhs,
                    to: CastTarget::Decimal,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                match regs[lhs_cast as usize] {
                    ValueRef::Missing => regs[dst as usize] = ValueRef::Missing,
                    _ => self.eval_inst(&fn_decimal(dst, lhs_cast, rhs), arena, regs, udf)?,
                }
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
                    dst: rhs_cast,
                    from: rhs,
                    to: CastTarget::Decimal,
                };
                self.eval_inst(&cast, arena, regs, udf)?;
                match regs[rhs_cast as usize] {
                    ValueRef::Missing => regs[dst as usize] = ValueRef::Missing,
                    _ => self.eval_inst(&fn_decimal(dst, lhs, rhs_cast), arena, regs, udf)?,
                }
            }
            (_, _) => {
                return Err(EngineError::IllegalState(
                    "Type mismatch for add!".to_string(),
                ));
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
            // A float that no decimal holds (NaN, infinity) is MISSING, as in the legacy
            // evaluator
            ValueRef::F64(f) => match to {
                CastTarget::Decimal => Decimal::from_f64(f)
                    .filter(|_| f.is_finite())
                    .map_or(ValueRef::Missing, ValueRef::Decimal),
                CastTarget::F64 => v,
                _ => return Err(EngineError::NotImplemented),
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
    /// Run a scalar instruction. An Ion variant it reads as a value is first decoded (into
    /// `arena`) in its register, so each operator sees VM values only, then put back: a
    /// result row shows the literal as written.
    pub(crate) fn eval_inst<'a>(
        &self,
        inst: &Inst,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<()> {
        let Some(operands) = ValueOperands::of(inst) else {
            return self.eval_decoded(inst, arena, regs, udf);
        };
        let mut variants = [None; 3];
        for (saved, &reg) in variants.iter_mut().zip(operands.regs()) {
            let value = regs[reg as usize];
            if let ValueRef::Variant(..) = value {
                regs[reg as usize] = variant::lower(value, arena)?;
                *saved = Some((reg, value));
            }
        }
        let result = self.eval_decoded(inst, arena, regs, udf);
        for (reg, value) in variants.into_iter().flatten() {
            if operands.dst != Some(reg) {
                regs[reg as usize] = value;
            }
        }
        result
    }

    /// Run a scalar instruction whose operands are decoded.
    fn eval_decoded<'a>(
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
                regs[*dst as usize] = ValueRef::I64(arithmetic(av.checked_add(bv))?);
            }
            Inst::AddF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::F64(av + bv);
            }
            Inst::AddDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Decimal(arithmetic(av.checked_add(bv))?);
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
                regs[*dst as usize] = ValueRef::I64(arithmetic(av.checked_sub(bv))?);
            }
            Inst::SubF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::F64(av - bv);
            }
            Inst::SubDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Decimal(arithmetic(av.checked_sub(bv))?);
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
                regs[*dst as usize] = ValueRef::I64(arithmetic(av.checked_rem(bv))?);
            }
            Inst::ModF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::F64(av % bv);
            }
            Inst::ModDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Decimal(arithmetic(av.checked_rem(bv))?);
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
            Inst::EqDynamic { dst, a, b } => {
                regs[*dst as usize] = compare::eq(regs[*a as usize], regs[*b as usize], arena)?;
            }
            // --- Mul ---
            Inst::MulI64 { dst, a, b } => {
                let av = regs[*a as usize].as_i64()?;
                let bv = regs[*b as usize].as_i64()?;
                regs[*dst as usize] = ValueRef::I64(arithmetic(av.checked_mul(bv))?);
            }
            Inst::MulF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::F64(av * bv);
            }
            Inst::MulDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Decimal(arithmetic(av.checked_mul(bv))?);
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
                regs[*dst as usize] = ValueRef::I64(arithmetic(av.checked_div(bv))?);
            }
            Inst::DivF64 { dst, a, b } => {
                let av = regs[*a as usize].as_f64()?;
                let bv = regs[*b as usize].as_f64()?;
                regs[*dst as usize] = ValueRef::F64(av / bv);
            }
            Inst::DivDecimal { dst, a, b } => {
                let av = regs[*a as usize].as_decimal()?;
                let bv = regs[*b as usize].as_decimal()?;
                regs[*dst as usize] = ValueRef::Decimal(arithmetic(av.checked_div(bv))?);
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
            Inst::GtDynamic { dst, a, b } => {
                regs[*dst as usize] =
                    eval_ordering_dynamic(regs[*a as usize], regs[*b as usize], Ordering::Greater);
            }
            // --- Lt ---
            Inst::LtDynamic { dst, a, b } => {
                regs[*dst as usize] =
                    eval_ordering_dynamic(regs[*a as usize], regs[*b as usize], Ordering::Less);
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
                    regs[*dst as usize] = match coll {
                        // True if an element `=` the value; else NULL if a comparison was
                        // NULL or MISSING (an element was absent: it might have been equal);
                        // else false (spec section 8)
                        ValueRef::List(items) | ValueRef::Bag(items) => {
                            let mut result = ValueRef::Bool(false);
                            for item in items.iter() {
                                match compare::eq(val, *item, arena)? {
                                    ValueRef::Bool(true) => {
                                        result = ValueRef::Bool(true);
                                        break;
                                    }
                                    ValueRef::Null | ValueRef::Missing => result = ValueRef::Null,
                                    _ => {}
                                }
                            }
                            result
                        }
                        ValueRef::Null | ValueRef::Missing => ValueRef::Null,
                        // A value that is no collection: whether the value `=` it
                        other => match compare::eq(val, other, arena)? {
                            ValueRef::Missing => ValueRef::Null,
                            result => result,
                        },
                    };
                }
            }
            // Three-valued logic, as `partiql_value`'s `BinaryAnd`/`BinaryOr`/`Not`: NULL and
            // MISSING are unknown; any other non-boolean is a type mismatch (MISSING).
            Inst::AndBool { dst, a, b } => {
                regs[*dst as usize] = logic(regs[*a as usize], regs[*b as usize], false);
            }
            Inst::OrBool { dst, a, b } => {
                regs[*dst as usize] = logic(regs[*a as usize], regs[*b as usize], true);
            }
            Inst::NotBool { dst, src } => {
                regs[*dst as usize] = match regs[*src as usize] {
                    ValueRef::Bool(v) => ValueRef::Bool(!v),
                    ValueRef::Null | ValueRef::Missing => ValueRef::Null,
                    _ => ValueRef::Missing,
                };
            }
            Inst::MissingToNull { dst, src } => {
                regs[*dst as usize] = match regs[*src as usize] {
                    ValueRef::Missing => ValueRef::Null,
                    other => other,
                };
            }
            Inst::RowCount {
                reg,
                strict,
                otherwise,
            } => {
                regs[*reg as usize] = match regs[*reg as usize] {
                    ValueRef::I64(n) if n >= 0 => ValueRef::I64(n),
                    // An unknown count is no count (`LIMIT NULL`), in strict mode too
                    ValueRef::Null | ValueRef::Missing => ValueRef::I64(*otherwise),
                    other if *strict => {
                        return Err(EngineError::StrictModeViolation(format!(
                            "LIMIT and OFFSET take a non-negative integer, not {other:?}"
                        )))
                    }
                    _ => ValueRef::I64(*otherwise),
                };
            }
            Inst::NullIf { dst, value, eq } => {
                regs[*dst as usize] = match regs[*eq as usize] {
                    ValueRef::Bool(true) => ValueRef::Null,
                    _ => regs[*value as usize],
                };
            }
            Inst::CollToScalar { dst, src } => {
                regs[*dst as usize] = match regs[*src as usize] {
                    ValueRef::Bag([ValueRef::Tuple(t)]) | ValueRef::List([ValueRef::Tuple(t)]) => {
                        match t.fields {
                            [field] => field.value,
                            _ => ValueRef::Missing,
                        }
                    }
                    _ => ValueRef::Missing,
                };
            }
            // An Ion typed null (`null.int`) is NULL once decoded
            Inst::IsNull { dst, src } => {
                regs[*dst as usize] = ValueRef::Bool(is_null_or_missing(regs[*src as usize]));
            }
            Inst::IsMissing { dst, src } => {
                regs[*dst as usize] =
                    ValueRef::Bool(matches!(regs[*src as usize], ValueRef::Missing));
            }
            Inst::NegNum { dst, src } => {
                regs[*dst as usize] = match regs[*src as usize] {
                    ValueRef::I64(n) => ValueRef::I64(arithmetic(n.checked_neg())?),
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
                    argv.push(variant::lower(regs[*arg as usize], arena)?);
                }
                let result = registry.call(name, &argv, arena)?;
                regs[*dst as usize] = result;
            }
            Inst::MakeTuple {
                dst,
                attr_regs,
                value_regs,
            } => {
                // As in the legacy evaluator, an attribute whose name is no string, or whose
                // value is MISSING, is left out
                let mut fields = Vec::with_capacity(attr_regs.len());
                for (attr_reg, value_reg) in attr_regs.iter().zip(value_regs) {
                    let name = variant::lower(regs[*attr_reg as usize], arena)?;
                    let value = regs[*value_reg as usize];
                    if matches!(name, ValueRef::Str(_)) && !matches!(value, ValueRef::Missing) {
                        fields.push((name, value));
                    }
                }
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

            Inst::CoerceToTuple {
                dst,
                src,
                const_idx,
            } => {
                let value = regs[*src as usize];
                match value {
                    ValueRef::Tuple(_) => {
                        regs[*dst as usize] = value;
                    }
                    // A MISSING row has no attributes
                    ValueRef::Missing => {
                        regs[*dst as usize] = ValueRef::Tuple(arena.alloc_tuple([]));
                    }
                    _ => {
                        let name_ref = self
                            .const_refs
                            .get(*const_idx as usize)
                            .copied()
                            .ok_or_else(|| {
                                EngineError::IllegalState("invalid const index".to_string())
                            })?;
                        let fields = std::iter::once((name_ref, value));
                        let tuple_ref = arena.alloc_tuple(fields);
                        regs[*dst as usize] = ValueRef::Tuple(tuple_ref);
                    }
                }
            }

            Inst::ConcatTuples { dst, srcs } => {
                let name = ValueRef::Str("_1");
                let mut fields = Vec::new();
                // `srcs` has no fixed length, so its variants are decoded here
                for src in srcs {
                    match variant::lower(regs[*src as usize], arena)? {
                        ValueRef::Tuple(t) => {
                            fields.extend(t.fields.iter().map(|f| (ValueRef::Str(f.name), f.value)))
                        }
                        ValueRef::Missing => {}
                        value => fields.push((name, value)),
                    }
                }
                regs[*dst as usize] = ValueRef::Tuple(arena.alloc_tuple(fields));
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
            | Inst::SkipRow { .. }
            | Inst::Coalesce { .. }
            | Inst::MaterializeCursor { .. }
            | Inst::CreateTableFnCursor { .. }
            | Inst::AssertCollection { .. }
            | Inst::SorterReset { .. }
            | Inst::SorterInsert { .. }
            | Inst::SorterSort { .. }
            | Inst::SorterData { .. }
            | Inst::SorterNext { .. }
            | Inst::SorterCollect { .. }
            | Inst::HashReset { .. }
            | Inst::HashInsert { .. }
            | Inst::HashAdd { .. }
            | Inst::HashTake { .. }
            | Inst::ScratchEnter
            | Inst::ScratchLeave
            | Inst::AggStep { .. }
            | Inst::AggFinal { .. }
            | Inst::AggReset { .. }
            | Inst::Gosub { .. }
            | Inst::Return { .. }
            | Inst::Copy { .. }
            | Inst::SorterNewGroup { .. }
            | Inst::SorterGroupBag { .. } => {
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
    /// The slot holding field `field` of the binding named `alias`, when the scan projected it.
    fn resolve_qualified(
        &self,
        alias: &BindingsName<'_>,
        field: &BindingsName<'_>,
    ) -> Option<SlotId>;
}

#[derive(Default)]
pub struct ProgramBuilder {
    pub(crate) insts: Vec<Inst>,
    consts: Vec<ValueOwned>,
    /// The index of each constant that is shared rather than pushed again.
    shared_consts: FxHashMap<ValueOwned, u16>,
    keys: Vec<String>,
    next_reg: u16,
    /// Registers used: the high-water mark of `next_reg`.
    reg_count: u16,
    slot_count: u16,
    /// Set while the emitter compiles an expression whose subqueries it has just run, so
    /// that reading a subquery's register before it ran fails to compile.
    pub(crate) subqueries_run: bool,
}

impl ProgramBuilder {
    pub fn new(slot_count: u16) -> Self {
        ProgramBuilder {
            insts: Vec::new(),
            consts: Vec::new(),
            shared_consts: FxHashMap::default(),
            keys: Vec::new(),
            next_reg: slot_count,
            reg_count: slot_count,
            slot_count,
            subqueries_run: false,
        }
    }

    /// Emit `expr`, evaluated into register `dst`. Its temporaries start at the first
    /// register the builder was not told about; they are free again afterwards.
    pub fn emit_expr(&mut self, expr: &Expr, dst: SlotId) -> Result<()> {
        let scratch = self.next_reg;
        ExprCompiler { builder: self }.compile_to_slot(expr, dst)?;
        self.reg_count = self.reg_count.max(self.next_reg);
        self.next_reg = scratch;
        Ok(())
    }

    /// Push a constant and return its index (public for compiler use).
    pub fn push_const_pub(&mut self, value: ValueOwned) -> Result<u16> {
        self.push_const(value)
    }

    /// Push a key and return its index (public for compiler use).
    pub fn push_key_pub(&mut self, key: String) -> Result<u16> {
        self.intern_key(key)
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
            reg_count: self.reg_count.max(self.next_reg),
            slot_count: self.slot_count,
        }
    }

    fn alloc_reg(&mut self) -> Result<u16> {
        let reg = self.next_reg;
        self.next_reg = reg
            .checked_add(1)
            .ok_or(EngineError::TooLarge("registers"))?;
        Ok(reg)
    }

    /// Push a constant, sharing one already pushed when equal. Only scalars whose equality
    /// is identity are shared: equal decimals can differ in scale, equal floats in sign.
    fn push_const(&mut self, value: ValueOwned) -> Result<u16> {
        let shared = matches!(
            value,
            ValueOwned::Missing
                | ValueOwned::Null
                | ValueOwned::Bool(_)
                | ValueOwned::I64(_)
                | ValueOwned::String(_)
        );
        if shared {
            if let Some(&idx) = self.shared_consts.get(&value) {
                return Ok(idx);
            }
        }
        let idx = count_u16(self.consts.len(), "constants")?;
        if shared {
            self.shared_consts.insert(value.clone(), idx);
        }
        self.consts.push(value);
        Ok(idx)
    }

    fn intern_key(&mut self, key: String) -> Result<u16> {
        if let Some((idx, _)) = self.keys.iter().enumerate().find(|(_, k)| *k == &key) {
            return count_u16(idx, "keys");
        }
        let idx = count_u16(self.keys.len(), "keys")?;
        self.keys.push(key);
        Ok(idx)
    }
}

/// Compiles an [`Expr`] into a [`ProgramBuilder`], taking temporaries from its next
/// unallocated register.
struct ExprCompiler<'b> {
    builder: &'b mut ProgramBuilder,
}

impl ExprCompiler<'_> {
    fn compile_expr(&mut self, expr: &Expr) -> Result<u16> {
        match expr {
            Expr::Literal(value) => {
                let reg = self.builder.alloc_reg()?;
                let const_idx = self.builder.push_const(value.clone())?;
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
                let dst = self.builder.alloc_reg()?;
                let l_cast = self.builder.alloc_reg()?;
                let r_cast = self.builder.alloc_reg()?;
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
                let dst = self.builder.alloc_reg()?;
                let l_cast = self.builder.alloc_reg()?;
                let r_cast = self.builder.alloc_reg()?;
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
                let dst = self.builder.alloc_reg()?;
                let l_cast = self.builder.alloc_reg()?;
                let r_cast = self.builder.alloc_reg()?;
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
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::EqDynamic { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::And(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::AndBool { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::Or(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::OrBool { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::Not(expr) => {
                let src = self.compile_expr(expr)?;
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::NotBool { dst, src });
                Ok(dst)
            }
            Expr::IsNull(expr) => {
                let src = self.compile_expr(expr)?;
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::IsNull { dst, src });
                Ok(dst)
            }
            Expr::IsMissing(expr) => {
                let src = self.compile_expr(expr)?;
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::IsMissing { dst, src });
                Ok(dst)
            }
            Expr::Neg(expr) => {
                let src = self.compile_expr(expr)?;
                let dst = self.builder.alloc_reg()?;
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
                let dst = self.builder.alloc_reg()?;
                let regex_str = like_to_re_pattern(pattern, escape);
                let pattern_idx = self.builder.intern_key(regex_str)?;
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
                let dst = self.builder.alloc_reg()?;
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
                let dst = self.builder.alloc_reg()?;
                let key_idx = self.builder.intern_key(key.clone())?;
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
                let dst = self.builder.alloc_reg()?;
                let func_idx = self.builder.intern_key(name.clone())?;
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
                let dst = self.builder.alloc_reg()?;
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
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::MakeList {
                    dst,
                    element_regs: elements_regs,
                });
                Ok(dst)
            }
            Expr::Mul(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg()?;
                let l_cast = self.builder.alloc_reg()?;
                let r_cast = self.builder.alloc_reg()?;
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
                let dst = self.builder.alloc_reg()?;
                let l_cast = self.builder.alloc_reg()?;
                let r_cast = self.builder.alloc_reg()?;
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
                let dst = self.builder.alloc_reg()?;
                let l_cast = self.builder.alloc_reg()?;
                let r_cast = self.builder.alloc_reg()?;
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
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::GtDynamic { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::Lt(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::LtDynamic { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::Concat(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg()?;
                self.builder
                    .insts
                    .push(Inst::ConcatDynamic { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::In(left, right) => {
                let value = self.compile_expr(left)?;
                let collection = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg()?;
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
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::MakeBag {
                    dst,
                    element_regs: elements_regs,
                });
                Ok(dst)
            }
            // The emitter has already evaluated the subquery into its register
            Expr::Subquery { dst, .. } if self.builder.subqueries_run => Ok(*dst),
            Expr::Subquery { id, .. } => Err(EngineError::IllegalState(format!(
                "subquery {id} read without running it"
            ))),
            Expr::NullIf(a, b) => {
                let value = self.compile_expr(a)?;
                let other = self.compile_expr(b)?;
                let eq = self.compile_expr(&Expr::Eq(
                    Expr::SlotRef(value).into(),
                    Expr::SlotRef(other).into(),
                ))?;
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::NullIf { dst, value, eq });
                Ok(dst)
            }
            // ```text
            //         dst = NULL
            //         <a1>; if a1 is present: dst = a1, skip to end
            //         <a2>; if a2 is present: dst = a2, skip to end
            //   end:
            // ```
            // An argument's code runs only if those before it are absent. The skips are
            // relative, so they hold wherever the expression's code is laid out.
            Expr::Coalesce(exprs) => {
                let dst = self.builder.alloc_reg()?;
                let const_idx = self.builder.push_const(ValueOwned::Null)?;
                self.builder.insts.push(Inst::LoadConst { dst, const_idx });
                let mut checks = Vec::with_capacity(exprs.len());
                for expr in exprs {
                    let src = self.compile_expr(expr)?;
                    checks.push(self.builder.insts.len());
                    self.builder
                        .insts
                        .push(Inst::Coalesce { dst, src, skip: 0 });
                }
                let end = self.builder.insts.len();
                for at in checks {
                    if let Inst::Coalesce { skip, .. } = &mut self.builder.insts[at] {
                        *skip = u32::try_from(end - at - 1)
                            .map_err(|_| EngineError::TooLarge("instructions"))?;
                    }
                }
                Ok(dst)
            }
            Expr::Between(value, lo, hi) => {
                let (value, lo, hi) = (
                    self.compile_expr(value)?,
                    self.compile_expr(lo)?,
                    self.compile_expr(hi)?,
                );
                let reg = |r| Box::new(Expr::SlotRef(r));
                let at_least = Expr::Or(
                    Expr::Gt(reg(value), reg(lo)).into(),
                    Expr::Eq(reg(value), reg(lo)).into(),
                );
                let at_most = Expr::Or(
                    Expr::Lt(reg(value), reg(hi)).into(),
                    Expr::Eq(reg(value), reg(hi)).into(),
                );
                self.compile_expr(&Expr::And(at_least.into(), at_most.into()))
            }
            Expr::CollToScalar(expr) => {
                let src = self.compile_expr(expr)?;
                let dst = self.builder.alloc_reg()?;
                self.builder.insts.push(Inst::CollToScalar { dst, src });
                Ok(dst)
            }
        }
    }

    fn compile_to_slot(&mut self, expr: &Expr, slot: SlotId) -> Result<()> {
        let reg = self.compile_expr(expr)?;
        self.builder.insts.push(Inst::StoreSlot { slot, src: reg });
        Ok(())
    }
}

/// Subqueries already lowered, by the address of their node in the expression being lowered.
pub(crate) type LoweredSubqueries = FxHashMap<*const SubQueryExpr, Expr>;

pub struct LogicalExprCompiler<'a, R: SlotResolver> {
    resolver: &'a R,
    subqueries: Option<&'a LoweredSubqueries>,
}

impl<'a, R: SlotResolver> LogicalExprCompiler<'a, R> {
    pub fn new(resolver: &'a R) -> Self {
        LogicalExprCompiler {
            resolver,
            subqueries: None,
        }
    }

    /// Lower each subquery of the expression to its entry in `subqueries`.
    pub(crate) fn with_subqueries(mut self, subqueries: &'a LoweredSubqueries) -> Self {
        self.subqueries = Some(subqueries);
        self
    }

    /// Lower `expr`, resolving its names to registers.
    pub fn lower_expr(&self, expr: &ValueExpr) -> Result<Expr> {
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
                // `alias.field` the scan of `alias` projected into its own slot
                if let (ValueExpr::VarRef(alias, _), Some(PathComponent::Key(field))) =
                    (&**base, components.first())
                {
                    if let Some(slot) = self.resolver.resolve_qualified(alias, field) {
                        return path_tail(Expr::SlotRef(slot), &components[1..]);
                    }
                }

                // First, try to resolve the first component as a pushed-down field of the
                // base binding; any remaining components navigate within that field. Skip
                // this when the base is some other binding: after `GROUP BY d.k AS k`, `k.k`
                // reads field `k` of the key `k`, not the key itself.
                // TODO(#695): field slots are keyed by field name only; key them by
                // (binding, field) and drop this name-only lookup.
                if let (ValueExpr::VarRef(base_name, scope), Some(PathComponent::Key(name))) =
                    (&**base, components.first())
                {
                    let base_is_other_binding = !self.resolver.is_alias(base_name)
                        && self
                            .resolver
                            .resolve_var(base_name, scope.clone())
                            .is_some();
                    if let Some(slot) = (!base_is_other_binding)
                        .then(|| self.resolver.resolve_field(name))
                        .flatten()
                    {
                        return path_tail(Expr::SlotRef(slot), &components[1..]);
                    }
                }

                // Check if base is an alias with a resolvable slot
                if let Some((is_alias, base_slot)) = resolve_alias_info(self.resolver, base) {
                    if is_alias {
                        if let Some(base_slot) = base_slot {
                            // We have a resolved base slot - build GetField chain
                            return path_tail(Expr::SlotRef(base_slot), components);
                        }
                        // is_alias=true but base_slot=None means VarRef(Local) that can't be resolved yet
                        // This can happen for table names in FROM clause - fall through to generic handling
                    }
                }

                // Generic path handling - lower the base expression and build GetField chain
                path_tail(self.lower_expr(base)?, components)
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
                    other => Err(EngineError::UnsupportedExpr(format!("{other:?}"))),
                }
            }
            ValueExpr::UnExpr(op, expr) => {
                let expr = self.lower_expr(expr)?;
                match op {
                    partiql_logical::UnaryOp::Not => Ok(Expr::Not(expr.into())),
                    partiql_logical::UnaryOp::Neg => Ok(Expr::Neg(expr.into())),
                    partiql_logical::UnaryOp::Pos => Ok(Expr::Pos(expr.into())),
                    other => Err(EngineError::UnsupportedExpr(format!("{other:?}"))),
                }
            }
            ValueExpr::Call(CallExpr {
                name: CallName::CollToScalar,
                arguments,
            }) => match arguments.as_slice() {
                [arg] => Ok(Expr::CollToScalar(self.lower_expr(arg)?.into())),
                _ => Err(EngineError::UnsupportedExpr(format!("{:?}", *expr))),
            },
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
            // `v BETWEEN lo AND hi` is `v >= lo AND v <= hi`
            ValueExpr::BetweenExpr(between) => Ok(Expr::Between(
                self.lower_expr(&between.value)?.into(),
                self.lower_expr(&between.from)?.into(),
                self.lower_expr(&between.to)?.into(),
            )),
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
                    other => Err(EngineError::UnsupportedExpr(format!("{other:?}"))),
                }
            }
            ValueExpr::SubQueryExpr(sub_query_expr) => self
                .subqueries
                .and_then(|lowered| lowered.get(&std::ptr::from_ref(sub_query_expr)))
                .cloned()
                .ok_or_else(|| EngineError::UnsupportedExpr(format!("{:?}", *expr))),
            ValueExpr::SimpleCase(_simple_case) => {
                Err(EngineError::UnsupportedExpr(format!("{:?}", *expr)))
            }
            ValueExpr::SearchedCase(_searched_case) => {
                Err(EngineError::UnsupportedExpr(format!("{:?}", *expr)))
            }
            ValueExpr::IsTypeExpr(is_type_expr) => {
                use partiql_logical::Type;
                let inner = self.lower_expr(&is_type_expr.expr)?;
                let check = match &is_type_expr.is_type {
                    Type::NullType => Expr::IsNull(inner.into()),
                    Type::MissingType => Expr::IsMissing(inner.into()),
                    other => {
                        return Err(EngineError::UnsupportedExpr(format!(
                            "IS type check for {:?}",
                            other
                        )));
                    }
                };
                if is_type_expr.not {
                    Ok(Expr::Not(check.into()))
                } else {
                    Ok(check)
                }
            }
            ValueExpr::NullIfExpr(nullif) => Ok(Expr::NullIf(
                self.lower_expr(&nullif.lhs)?.into(),
                self.lower_expr(&nullif.rhs)?.into(),
            )),
            ValueExpr::CoalesceExpr(coalesce) if !coalesce.elements.is_empty() => {
                let exprs = coalesce
                    .elements
                    .iter()
                    .map(|e| self.lower_expr(e))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Expr::Coalesce(exprs))
            }
            ValueExpr::GraphMatch(_graph_match_expr) => {
                Err(EngineError::UnsupportedExpr(format!("{:?}", *expr)))
            }
            _ => Err(EngineError::UnsupportedExpr(format!("{:?}", *expr))),
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
        // Kept as written and decoded where an operator reads it, but checked now
        Lit::Variant(bytes, type_name) => {
            variant::validate(bytes)?;
            ValueOwned::Variant(bytes.clone(), type_name.clone())
        }
        other => return Err(EngineError::UnsupportedExpr(format!("{other:?}"))),
    })
}

/// Three-valued AND (`dominant` false) or OR (`dominant` true). The dominant value decides
/// the result even next to an unknown; otherwise an unknown makes NULL, a mismatch MISSING.
fn logic<'a>(a: ValueRef<'a>, b: ValueRef<'a>, dominant: bool) -> ValueRef<'a> {
    let logical =
        |v: &ValueRef<'_>| matches!(v, ValueRef::Null | ValueRef::Missing | ValueRef::Bool(_));
    match (a, b) {
        (ValueRef::Bool(x), ValueRef::Bool(y)) => {
            ValueRef::Bool(if dominant { x || y } else { x && y })
        }
        (ValueRef::Bool(x), other) | (other, ValueRef::Bool(x))
            if x == dominant && matches!(other, ValueRef::Null | ValueRef::Missing) =>
        {
            ValueRef::Bool(dominant)
        }
        (a, b) if logical(&a) && logical(&b) => ValueRef::Null,
        _ => ValueRef::Missing,
    }
}

fn call_name(call: &CallExpr) -> String {
    match &call.name {
        CallName::ByName(name) => name.clone(),
        CallName::ById(name, _, _) => name.clone(),
        other => format!("{other:?}"),
    }
}

/// The result of checked integer or decimal arithmetic: an error on overflow or division by
/// zero, where unchecked arithmetic would panic.
fn arithmetic<T>(result: Option<T>) -> Result<T> {
    result.ok_or_else(|| EngineError::Arithmetic("overflow or division by zero".to_string()))
}

fn is_null_or_missing(v: ValueRef<'_>) -> bool {
    matches!(v, ValueRef::Null | ValueRef::Missing)
}

fn is_missing(v: ValueRef<'_>) -> bool {
    matches!(v, ValueRef::Missing)
}

/// The registers an instruction reads as values of any type, and the one it writes: where
/// an Ion variant is decoded before the instruction runs.
struct ValueOperands {
    regs: [u16; 3],
    len: usize,
    dst: Option<u16>,
}

impl ValueOperands {
    fn of(inst: &Inst) -> Option<Self> {
        let ops = |dst: Option<u16>, regs: &[u16]| {
            let mut ops = ValueOperands {
                regs: [0; 3],
                len: regs.len(),
                dst,
            };
            ops.regs[..regs.len()].copy_from_slice(regs);
            Some(ops)
        };
        match *inst {
            Inst::AddDynamic { dst, a, b, .. }
            | Inst::SubDynamic { dst, a, b, .. }
            | Inst::ModDynamic { dst, a, b, .. }
            | Inst::MulDynamic { dst, a, b, .. }
            | Inst::DivDynamic { dst, a, b, .. }
            | Inst::ExpDynamic { dst, a, b, .. }
            | Inst::EqDynamic { dst, a, b }
            | Inst::GtDynamic { dst, a, b }
            | Inst::LtDynamic { dst, a, b }
            | Inst::ConcatDynamic { dst, a, b }
            | Inst::AndBool { dst, a, b }
            | Inst::OrBool { dst, a, b } => ops(Some(dst), &[a, b]),
            Inst::InDynamic {
                dst,
                value,
                collection,
            } => ops(Some(dst), &[value, collection]),
            Inst::RowCount { reg, .. } => ops(Some(reg), &[reg]),
            Inst::NotBool { dst, src }
            | Inst::NegNum { dst, src }
            | Inst::MissingToNull { dst, src }
            | Inst::CollToScalar { dst, src }
            | Inst::IsNull { dst, src }
            | Inst::IsMissing { dst, src }
            | Inst::CoerceToTuple { dst, src, .. } => ops(Some(dst), &[src]),
            Inst::Cast { dst, from, .. } => ops(Some(dst), &[from]),
            Inst::GetField { dst, base, .. } => ops(Some(dst), &[base]),
            Inst::LikeMatch { dst, value, .. } => ops(Some(dst), &[value]),
            Inst::LikeDynamicMatch {
                dst,
                value,
                pattern,
                escape,
            } => ops(Some(dst), &[value, pattern, escape]),
            _ => None,
        }
    }

    fn regs(&self) -> &[u16] {
        &self.regs[..self.len]
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
        other => format!("{other:?}"),
    }
}

/// Apply path `components` to `base` as a `GetField` chain. The VM has no index
/// navigation yet, so `Index`/`IndexExpr`/`KeyExpr` components are unsupported on
/// every path, pushed-down or not.
fn path_tail(base: Expr, components: &[PathComponent]) -> Result<Expr> {
    components
        .iter()
        .try_fold(base, |current, component| match component {
            PathComponent::Key(name) => Ok(Expr::GetField(
                current.into(),
                bindings_name_to_string(name),
            )),
            _ => Err(EngineError::UnsupportedExpr(format!(
                "unsupported path component: {component:?}"
            ))),
        })
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

/// `a > b` or `a < b` (`want` is the order sought), for any two values: MISSING if either is
/// MISSING, else NULL if either is NULL; numbers by value exactly whatever their type (NaN is
/// ordered with nothing: false); strings by code point; other values NULL, as no order holds.
fn eval_ordering_dynamic<'a>(a: ValueRef<'a>, b: ValueRef<'a>, want: Ordering) -> ValueRef<'a> {
    match (a, b) {
        (ValueRef::Missing, _) | (_, ValueRef::Missing) => ValueRef::Missing,
        (ValueRef::Null, _) | (_, ValueRef::Null) => ValueRef::Null,
        (ValueRef::Str(a), ValueRef::Str(b)) => ValueRef::Bool(a.cmp(b) == want),
        _ => match compare::number_cmp(&a, &b) {
            Some(order) => ValueRef::Bool(order == Some(want)),
            None => ValueRef::Null,
        },
    }
}
