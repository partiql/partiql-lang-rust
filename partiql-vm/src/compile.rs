//! Pass 2 of the compiler: emit the bytecode for a [`Query`] lowered by
//! [`crate::lower_ir`].
//!
//! Lowering has assigned every register and cursor and resolved every name, so emission
//! is a walk over the operator tree. Each operator is emitted by
//! `emit_op(op, row)`: the code producing `op`'s rows, with `row` (the operator above it)
//! emitted where each row is ready. A scan becomes a cursor loop, and the operators above
//! it are emitted inside the loop:
//! ```text
//!         OpenCursor c
//!   head: NextRow c, eof→done
//!         <row>                 // e.g. a WHERE that fails jumps back to head
//!         Jump head
//!   done: CloseCursor c
//! ```
//! An operator that runs `row` from more than one place (an outer join: a matched row and
//! a padded row; GROUP BY: a change of keys and the last group) emits it once, as a
//! subroutine each place calls with `Gosub`.
//!
//! Code is emitted into routines, laid out one after another when the program is
//! finished: the main program, then each subroutine, in the order made. Jumps target
//! labels, resolved then.

use crate::arena::SlotId;
use crate::catalog::CompilationContext;
use crate::error::{EngineError, Result};
use crate::expr::{Inst, Program, ProgramBuilder};
use crate::lower_ir::{
    self, Aggregate, Cond, Filter, Join, JoinType, Limit, Op, Open, Output, Project, Query, Root,
    ScanLoop,
};
use crate::plan::{CompiledPlan, EvaluationMode};
use crate::value::ValueOwned;
use partiql_logical::{BindingsOp, LogicalPlan};
use rustc_hash::FxHashMap;

pub struct PlanCompiler<'a> {
    compilation_context: &'a CompilationContext,
    mode: EvaluationMode,
}

impl<'a> PlanCompiler<'a> {
    pub fn new(compilation_context: &'a CompilationContext, mode: EvaluationMode) -> Self {
        PlanCompiler {
            compilation_context,
            mode,
        }
    }

    /// Compile a logical plan into a flat bytecode `CompiledPlan`.
    pub fn compile(&mut self, plan: &LogicalPlan<BindingsOp>) -> Result<CompiledPlan> {
        let query = lower_ir::lower(self.compilation_context, self.mode, plan)?;
        let program = Emitter::new(&query).emit()?;
        Ok(CompiledPlan {
            program,
            cursors: query.cursors,
            shape: query.shape,
            slot_count: query.reg_count as usize,
            scan_metadata: query.scan_metadata,
            inline_scans: query.inline_scans,
            expr_scan_ids: query.expr_scan_ids,
            table_fn_scans: query.table_fn_scans,
            sorter_metadata: query.sorters,
        })
    }
}

// ---------------------------------------------------------------------------
// Assembler
// ---------------------------------------------------------------------------

/// A jump target, placed with [`Asm::bind`] before or after the jumps to it.
#[derive(Clone, Copy)]
struct Label(usize);

/// A [`ProgramBuilder`] whose jumps target labels, emitting into routines: runs of code
/// laid out one after another when the program is finished. Routine 0 is the main program;
/// the others (subroutines, and code jumped to once) follow it, in the order they were made.
struct Asm {
    /// `b.insts` is the code of the routine being emitted.
    b: ProgramBuilder,
    /// Each routine's code; the current routine's is in `b.insts` instead.
    routines: Vec<Vec<Inst>>,
    current: usize,
    /// Each label's (routine, offset), once bound.
    labels: Vec<Option<(usize, u32)>>,
    /// Jumps to resolve: (routine, offset, target).
    fixups: Vec<(usize, usize, Label)>,
}

impl Asm {
    fn new(b: ProgramBuilder) -> Self {
        Asm {
            b,
            routines: vec![vec![]],
            current: 0,
            labels: vec![],
            fixups: vec![],
        }
    }

    fn label(&mut self) -> Label {
        self.labels.push(None);
        Label(self.labels.len() - 1)
    }

    /// Place `label` at the next instruction.
    fn bind(&mut self, label: Label) {
        debug_assert!(self.labels[label.0].is_none(), "label bound twice");
        self.labels[label.0] = Some((self.current, self.b.insts.len() as u32));
    }

    /// Start a new routine and emit into it, until [`Asm::resume`] returns to the routine
    /// this returns. `entry` is bound to the new routine's start.
    fn begin_routine(&mut self, entry: Label) -> usize {
        let caller = self.current;
        self.routines.push(vec![]);
        self.resume(self.routines.len() - 1);
        self.bind(entry);
        caller
    }

    /// Emit into `routine`, after its code so far.
    fn resume(&mut self, routine: usize) {
        debug_assert_ne!(routine, self.current);
        let code = std::mem::take(&mut self.routines[routine]);
        self.routines[self.current] = std::mem::replace(&mut self.b.insts, code);
        self.current = routine;
    }

    fn push(&mut self, inst: Inst) {
        self.b.insts.push(inst);
    }

    /// Push `inst`, a jump whose target is filled in with `target`'s address.
    fn push_to(&mut self, inst: Inst, target: Label) {
        self.fixups.push((self.current, self.b.insts.len(), target));
        self.b.insts.push(inst);
    }

    fn jump(&mut self, target: Label) {
        self.push_to(Inst::Jump { target: 0 }, target);
    }

    fn jump_if_not_true(&mut self, src: SlotId, target: Label) {
        self.push_to(Inst::JumpIfNotTrue { src, target: 0 }, target);
    }

    fn gosub(&mut self, ret_reg: SlotId, target: Label) {
        self.push_to(Inst::Gosub { ret_reg, target: 0 }, target);
    }

    fn load_const(&mut self, dst: SlotId, value: ValueOwned) {
        let const_idx = self.b.push_const_pub(value);
        self.push(Inst::LoadConst { dst, const_idx });
    }

    /// Lay the routines out in order and resolve the jumps.
    fn finish(mut self) -> Program {
        debug_assert_eq!(self.current, 0, "a routine was left unfinished");
        self.routines[0] = std::mem::take(&mut self.b.insts);
        let mut base = Vec::with_capacity(self.routines.len());
        let mut insts = Vec::new();
        for code in &mut self.routines {
            base.push(insts.len() as u32);
            insts.append(code);
        }
        let labels = &self.labels;
        let address = |label: Label| {
            let (routine, offset) = labels[label.0].expect("jump to an unbound label");
            base[routine] + offset
        };
        let fixups: Vec<(usize, Label)> = self
            .fixups
            .iter()
            .map(|&(routine, offset, label)| (base[routine] as usize + offset, label))
            .collect();
        // A jump to a `Jump` goes straight to that jump's target
        let jumps: FxHashMap<u32, Label> = fixups
            .iter()
            .filter(|(idx, _)| matches!(insts[*idx], Inst::Jump { .. }))
            .map(|&(idx, target)| (idx as u32, target))
            .collect();
        for &(idx, label) in &fixups {
            let mut addr = address(label);
            for _ in 0..jumps.len() {
                match jumps.get(&addr) {
                    Some(&next) => addr = address(next),
                    None => break,
                }
            }
            match &mut insts[idx] {
                Inst::NextRow { eof_target: t, .. }
                | Inst::SorterSort { eof_target: t, .. }
                | Inst::Jump { target: t }
                | Inst::JumpIfNotTrue { target: t, .. }
                | Inst::DecrOrJump { target: t, .. }
                | Inst::Gosub { target: t, .. }
                | Inst::SorterNext { target: t, .. } => *t = addr,
                other => unreachable!("not a jump: {other:?}"),
            }
        }
        self.b.insts = insts;
        self.b.build()
    }
}

// ---------------------------------------------------------------------------
// Emission
// ---------------------------------------------------------------------------

/// Emits what runs for each row an operator produces: the operator above it.
type Row<'r, 'q> = &'r mut dyn FnMut(&mut Emitter<'q>) -> Result<()>;

struct Emitter<'q> {
    query: &'q Query,
    asm: Asm,
}

impl<'q> Emitter<'q> {
    fn new(query: &'q Query) -> Self {
        Emitter {
            query,
            asm: Asm::new(ProgramBuilder::new(query.reg_count)),
        }
    }

    fn emit(mut self) -> Result<Program> {
        match &self.query.root {
            Root::Expr { cursors, expr, dst } => {
                let halt = self.asm.label();
                for &cursor_id in cursors {
                    self.asm.push(Inst::OpenCursor { cursor_id });
                    self.asm.push_to(
                        Inst::NextRow {
                            cursor_id,
                            eof_target: 0,
                        },
                        halt,
                    );
                }
                self.asm.b.emit_expr(expr, *dst)?;
                self.asm.push(Inst::EmitRow);
                self.asm.bind(halt);
                self.asm.push(Inst::Halt);
            }
            Root::Rows(op) => {
                self.emit_op(op, &mut |e| {
                    e.asm.push(Inst::EmitRow);
                    Ok(())
                })?;
                self.asm.push(Inst::Halt);
            }
        }
        Ok(self.asm.finish())
    }

    /// The code producing the rows of `op`, running `row` for each.
    fn emit_op(&mut self, op: &'q Op, row: Row<'_, 'q>) -> Result<()> {
        match op {
            Op::Scan(scan) => self.emit_scan(scan, row),
            Op::Join(join) => self.emit_join(join, row),
            Op::Filter(filter) => self.emit_filter(filter, row),
            Op::Project(project) => self.emit_project(project, row),
            Op::Aggregate(aggregate) => self.emit_aggregate(aggregate, row),
            Op::Limit(limit) => self.emit_limit(limit, row),
        }
    }

    /// ```text
    ///         OpenCursor c
    ///   head: NextRow c, eof→done
    ///         <row>
    ///         Jump head
    ///   done: CloseCursor c
    /// ```
    fn emit_scan(&mut self, scan: &'q ScanLoop, row: Row<'_, 'q>) -> Result<()> {
        let cursor_id = scan.cursor;
        self.emit_open(&scan.open, cursor_id)?;
        let (head, done) = (self.asm.label(), self.asm.label());
        self.asm.bind(head);
        self.asm.push_to(
            Inst::NextRow {
                cursor_id,
                eof_target: 0,
            },
            done,
        );
        row(self)?;
        self.asm.jump(head);
        self.asm.bind(done);
        self.asm.push(Inst::CloseCursor { cursor_id });
        Ok(())
    }

    /// ```text
    /// for each input row:
    ///     if cond: <row>
    /// ```
    fn emit_filter(&mut self, filter: &'q Filter, row: Row<'_, 'q>) -> Result<()> {
        self.emit_op(&filter.input, &mut |e| e.emit_if(Some(&filter.cond), row))
    }

    /// ```text
    /// for each input row:
    ///     evaluate the outputs; <row>
    /// ```
    fn emit_project(&mut self, project: &'q Project, row: Row<'_, 'q>) -> Result<()> {
        self.emit_op(&project.input, &mut |e| {
            for output in &project.outputs {
                e.emit_output(output)?;
            }
            row(e)
        })
    }

    fn emit_output(&mut self, output: &Output) -> Result<()> {
        match output {
            Output::Eval { dst, expr } => self.asm.b.emit_expr(expr, *dst)?,
            Output::CoerceToTuple(reg) => {
                let const_idx = self.asm.b.push_const_pub(ValueOwned::String("_1".into()));
                self.asm.push(Inst::CoerceToTuple {
                    dst: *reg,
                    src: *reg,
                    const_idx,
                });
            }
            Output::AssertTuple(reg) => self.asm.push(Inst::AssertTuple { src: *reg }),
            Output::ConcatTuples { dst, srcs } => self.asm.push(Inst::ConcatTuples {
                dst: *dst,
                srcs: srcs.clone(),
            }),
        }
        Ok(())
    }

    /// Stopping mid-row leaves the input's cursors open, so `done` closes them (closing
    /// is idempotent):
    /// ```text
    ///         counter = count
    ///         for each input row:
    ///             if counter = 0: goto done
    ///             counter -= 1; <row>
    ///   done: CloseCursor each of the input's cursors
    /// ```
    fn emit_limit(&mut self, limit: &'q Limit, row: Row<'_, 'q>) -> Result<()> {
        let done = self.asm.label();
        self.asm
            .load_const(limit.counter, ValueOwned::I64(limit.count));
        self.emit_op(&limit.input, &mut |e| {
            e.asm.push_to(
                Inst::DecrOrJump {
                    counter_reg: limit.counter,
                    target: 0,
                },
                done,
            );
            row(e)
        })?;
        self.asm.bind(done);
        for cursor_id in cursors(&limit.input) {
            self.asm.push(Inst::CloseCursor { cursor_id });
        }
        Ok(())
    }

    // Each side of a join is reopened for every row of the side it is nested in, so a
    // lateral right side (`FROM t, t.arr x`) sees the current left row.
    //
    // TODO: a side that does not read the other side is recomputed on every pass. Memoize
    // it instead: build it once per entry into the join, e.g. into a B-tree keyed on the ON
    // equality (a `CreateBTree` / probe instruction pair), then replay or probe it. That
    // also keeps nondeterministic or remote sources (`rand()`, plugins) from being re-read.
    fn emit_join(&mut self, join: &'q Join, row: Row<'_, 'q>) -> Result<()> {
        match join.kind {
            JoinType::Inner => self.emit_inner_join(join, row),
            JoinType::Left => self.emit_left_join(join, row),
            JoinType::Right => self.emit_right_join(join, row),
            JoinType::Full => self.emit_full_join(join, row),
        }
    }

    /// ```text
    /// for l in L:
    ///     for r in R:
    ///         if ON: <row>
    /// ```
    fn emit_inner_join(&mut self, join: &'q Join, row: Row<'_, 'q>) -> Result<()> {
        self.emit_op(&join.left, &mut |e| {
            e.emit_op(&join.right, &mut |e| e.emit_if(join.on.as_ref(), row))
        })
    }

    /// ```text
    /// for l in L:
    ///     matched = false
    ///     for r in R:
    ///         if ON: matched = true; Gosub ROW
    ///     if not matched: R = NULL; Gosub ROW
    /// ```
    fn emit_left_join(&mut self, join: &'q Join, row: Row<'_, 'q>) -> Result<()> {
        let sub = self.emit_subroutine(join.ret, row)?;
        self.emit_preserving(join, &join.left, &join.right, sub)
    }

    /// LEFT JOIN with the sides swapped:
    /// ```text
    /// for r in R:
    ///     matched = false
    ///     for l in L:
    ///         if ON: matched = true; Gosub ROW
    ///     if not matched: L = NULL; Gosub ROW
    /// ```
    fn emit_right_join(&mut self, join: &'q Join, row: Row<'_, 'q>) -> Result<()> {
        let sub = self.emit_subroutine(join.ret, row)?;
        self.emit_preserving(join, &join.right, &join.left, sub)
    }

    /// LEFT JOIN, then the right rows no left row matched:
    /// ```text
    /// for l in L:
    ///     matched = false
    ///     for r in R:
    ///         if ON: matched = true; Gosub ROW
    ///     if not matched: R = NULL; Gosub ROW
    /// for r in R:
    ///     for l in L:
    ///         if ON: goto next r
    ///     L = NULL; Gosub ROW
    /// ```
    fn emit_full_join(&mut self, join: &'q Join, row: Row<'_, 'q>) -> Result<()> {
        let sub = self.emit_subroutine(join.ret, row)?;
        self.emit_preserving(join, &join.left, &join.right, sub)?;
        self.emit_op(&join.right, &mut |e| {
            let (found, next) = (e.asm.label(), e.asm.label());
            e.emit_op(&join.left, &mut |e| {
                e.emit_if(join.on.as_ref(), &mut |e| {
                    e.asm.jump(found);
                    Ok(())
                })
            })?;
            e.emit_null(&join.left)?;
            e.asm.gosub(join.ret, sub);
            e.asm.jump(next);
            // Stopping at a match leaves the left side's cursors open
            e.asm.bind(found);
            for cursor_id in cursors(&join.left) {
                e.asm.push(Inst::CloseCursor { cursor_id });
            }
            e.asm.bind(next);
            Ok(())
        })
    }

    /// The loops of an outer join preserving `kept`: each `kept` row with every `other`
    /// row ON matches, or with `other` NULL if none does, calling `sub` for each.
    /// ```text
    /// for k in KEPT:
    ///     matched = false
    ///     for o in OTHER:
    ///         if ON: matched = true; Gosub sub
    ///     if not matched: OTHER = NULL; Gosub sub
    /// ```
    fn emit_preserving(
        &mut self,
        join: &'q Join,
        kept: &'q Op,
        other: &'q Op,
        sub: Label,
    ) -> Result<()> {
        self.emit_op(kept, &mut |e| {
            e.asm.load_const(join.matched, ValueOwned::Bool(false));
            e.emit_op(other, &mut |e| {
                e.emit_if(join.on.as_ref(), &mut |e| {
                    e.asm.load_const(join.matched, ValueOwned::Bool(true));
                    e.asm.gosub(join.ret, sub);
                    Ok(())
                })
            })?;
            let (pad, skip) = (e.asm.label(), e.asm.label());
            e.asm.jump_if_not_true(join.matched, pad);
            e.asm.jump(skip);
            e.asm.bind(pad);
            e.emit_null(other)?;
            e.asm.gosub(join.ret, sub);
            e.asm.bind(skip);
            Ok(())
        })
    }

    /// `row`, unless `cond` is given and fails.
    fn emit_if(&mut self, cond: Option<&Cond>, row: Row<'_, 'q>) -> Result<()> {
        let skip = self.asm.label();
        if let Some(cond) = cond {
            self.asm.b.emit_expr(&cond.expr, cond.pred)?;
            self.asm.jump_if_not_true(cond.pred, skip);
        }
        row(self)?;
        self.asm.bind(skip);
        Ok(())
    }

    /// Emit `row` as a subroutine returning through `ret`, in a routine of its own.
    fn emit_subroutine(&mut self, ret: SlotId, row: Row<'_, 'q>) -> Result<Label> {
        let sub = self.asm.label();
        let caller = self.asm.begin_routine(sub);
        row(self)?;
        self.asm.push(Inst::Return { ret_reg: ret });
        self.asm.resume(caller);
        Ok(sub)
    }

    /// Bind each variable of a join side to NULL. A scan that loads only some fields gets
    /// MISSING in them instead, as `NULL.field` is MISSING.
    fn emit_null(&mut self, side: &Op) -> Result<()> {
        match side {
            Op::Scan(scan) => {
                let value = match scan.whole_row {
                    true => ValueOwned::Null,
                    false => ValueOwned::Missing,
                };
                for dst in scan.window.clone() {
                    self.asm.load_const(dst, value.clone());
                }
            }
            Op::Join(join) => {
                self.emit_null(&join.left)?;
                self.emit_null(&join.right)?;
            }
            Op::Filter(_) | Op::Project(_) | Op::Aggregate(_) | Op::Limit(_) => {
                return Err(EngineError::InvalidPlan(
                    "an outer join side that is not a FROM source".to_string(),
                ))
            }
        }
        Ok(())
    }

    /// Bind a scan's cursor to its data, then open it.
    fn emit_open(&mut self, open: &Open, cursor_id: u16) -> Result<()> {
        match open {
            Open::Bound => {}
            Open::TableFn { func_name, args } => {
                for (dst, arg) in args {
                    self.asm.b.emit_expr(arg, *dst)?;
                }
                let func_name_idx = self.asm.b.push_key_pub(func_name.clone());
                self.asm.push(Inst::CreateTableFnCursor {
                    cursor_id,
                    func_name_idx,
                });
            }
            Open::Expr {
                src,
                expr,
                assert_collection,
            } => {
                self.asm.b.emit_expr(expr, *src)?;
                if *assert_collection {
                    self.asm.push(Inst::AssertCollection { src: *src });
                }
                self.asm.push(Inst::MaterializeCursor {
                    src: *src,
                    cursor_id,
                });
            }
        }
        self.asm.push(Inst::OpenCursor { cursor_id });
        Ok(())
    }

    /// A sort-based GROUP BY. It reads all of its input into the sorter, then reads the
    /// records back sorted, calling `row` (as subroutine OUTPUT) once per group:
    /// ```text
    ///         has_data = false; reset the accumulators
    ///         for each input row: evaluate its record; SorterInsert
    ///         SorterSort, empty→done
    ///   read: SorterData
    ///         if the keys differ from the group's: Gosub OUTPUT; AggReset
    ///         copy the keys; AggStep each aggregate; has_data = true
    ///         SorterNext → read
    ///         Gosub OUTPUT             // the last group
    ///   done:
    /// ```
    fn emit_aggregate(&mut self, agg: &'q Aggregate, row: Row<'_, 'q>) -> Result<()> {
        let sorter_id = agg.sorter;
        let reset_accums = Inst::AggReset {
            accum_start: agg.accums.start,
            count: agg.accums.len() as u16,
        };
        self.asm.load_const(agg.has_data, ValueOwned::Bool(false));
        self.asm.push(reset_accums.clone());
        self.emit_op(&agg.input, &mut |e| {
            for (dst, expr) in (agg.insert..).zip(&agg.record_exprs) {
                e.asm.b.emit_expr(expr, dst)?;
            }
            e.asm.push(Inst::SorterInsert {
                sorter_id,
                src_reg: agg.insert,
                reg_count: agg.record_exprs.len() as u16,
            });
            Ok(())
        })?;
        let output = self.emit_group_output(agg, row)?;

        let (read, accumulate, done) = (self.asm.label(), self.asm.label(), self.asm.label());
        self.asm.push_to(
            Inst::SorterSort {
                sorter_id,
                eof_target: 0,
            },
            done,
        );
        self.asm.bind(read);
        self.asm.push(Inst::SorterData {
            sorter_id,
            dst_reg: agg.record,
        });
        if agg.key_count > 0 {
            for k in 0..agg.key_count {
                self.asm.push(Inst::CompareEq {
                    lhs_reg: agg.keys + k,
                    rhs_reg: agg.record + k,
                    dst_reg: if k == 0 { agg.same_group } else { agg.key_eq },
                });
                if k > 0 {
                    self.asm.push(Inst::AndBool {
                        dst: agg.same_group,
                        a: agg.same_group,
                        b: agg.key_eq,
                    });
                }
            }
            self.asm.push(Inst::NotBool {
                dst: agg.new_group,
                src: agg.same_group,
            });
            self.asm.jump_if_not_true(agg.new_group, accumulate);
            self.asm.gosub(agg.ret, output);
            self.asm.push(reset_accums);
        }
        self.asm.bind(accumulate);
        for k in 0..agg.key_count {
            self.asm.push(Inst::Copy {
                dst: agg.keys + k,
                src: agg.record + k,
            });
        }
        for group_agg in &agg.aggs {
            for &(func, accum_reg) in &group_agg.steps {
                self.asm.push(Inst::AggStep {
                    func,
                    accum_reg,
                    input_reg: group_agg.input,
                });
            }
        }
        self.asm.load_const(agg.has_data, ValueOwned::Bool(true));
        self.asm.push_to(
            Inst::SorterNext {
                sorter_id,
                target: 0,
            },
            read,
        );
        self.asm.gosub(agg.ret, output);
        self.asm.bind(done);
        Ok(())
    }

    /// GROUP BY's OUTPUT subroutine, run per group: skip an empty group, finalize the
    /// aggregates, then `row`.
    fn emit_group_output(&mut self, agg: &'q Aggregate, row: Row<'_, 'q>) -> Result<Label> {
        let output = self.asm.label();
        let caller = self.asm.begin_routine(output);
        let ret = self.asm.label();
        self.asm.jump_if_not_true(agg.has_data, ret);
        for group_agg in &agg.aggs {
            self.asm.push(Inst::AggFinal {
                func: group_agg.finalize,
                accum_reg: group_agg.accum,
                dst_reg: group_agg.accum,
            });
        }
        row(self)?;
        self.asm.bind(ret);
        self.asm.push(Inst::Return { ret_reg: agg.ret });
        self.asm.resume(caller);
        Ok(output)
    }
}

/// Every cursor an operator's subtree reads.
fn cursors(op: &Op) -> Vec<u16> {
    match op {
        Op::Scan(scan) => vec![scan.cursor],
        Op::Join(join) => {
            let mut ids = cursors(&join.left);
            ids.extend(cursors(&join.right));
            ids
        }
        Op::Filter(filter) => cursors(&filter.input),
        Op::Project(project) => cursors(&project.input),
        Op::Aggregate(aggregate) => cursors(&aggregate.input),
        Op::Limit(limit) => cursors(&limit.input),
    }
}
