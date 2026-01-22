use crate::engine::error::{EngineError, Result};
use crate::engine::row::{RowArena, RowFrame, SlotId, SlotValue};
use crate::engine::value::{value_get_field, ValueOwned, ValueRef};
use partiql_logical::{CallExpr, CallName, Lit, PathComponent, ValueExpr, VarRefType};
use partiql_value::BindingsName;
use partiql_value::Value;

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(ValueOwned),
    SlotRef(SlotId),
    Add(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    GetField(Box<Expr>, String),
    UdfCall { name: String, args: Vec<Expr> },
}

#[derive(Clone, Debug)]
pub enum Inst {
    LoadSlot { dst: u16, slot: SlotId },
    LoadConst { dst: u16, const_idx: u16 },
    AddI64 { dst: u16, a: u16, b: u16 },
    EqI64 { dst: u16, a: u16, b: u16 },
    AndBool { dst: u16, a: u16, b: u16 },
    OrBool { dst: u16, a: u16, b: u16 },
    NotBool { dst: u16, src: u16 },
    GetField { dst: u16, base: u16, key_idx: u16 },
    StoreSlot { slot: SlotId, src: u16 },
    CallUdf { dst: u16, func_idx: u16, args: Vec<u16> },
}

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub insts: Vec<Inst>,
    pub consts: Vec<ValueOwned>,
    pub keys: Vec<String>,
    pub reg_count: u16,
}

impl Program {
    pub fn eval<'a>(
        &self,
        frame: &mut RowFrame<'a>,
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<()> {
        let mut regs = vec![ValueRef::Missing; self.reg_count as usize];

        for inst in &self.insts {
            match inst {
                Inst::LoadSlot { dst, slot } => {
                    regs[*dst as usize] = match frame.slots[*slot as usize] {
                        SlotValue::Val(v) => v,
                        SlotValue::Owned(v) => ValueRef::from_owned(v),
                    };
                }
                Inst::LoadConst { dst, const_idx } => {
                    let value = self
                        .consts
                        .get(*const_idx as usize)
                        .ok_or_else(|| EngineError::IllegalState("invalid const index".to_string()))?;
                    let owned = frame.arena.alloc(value.clone());
                    regs[*dst as usize] = ValueRef::from_owned(owned);
                }
                Inst::AddI64 { dst, a, b } => {
                    let av = regs[*a as usize].as_i64()?;
                    let bv = regs[*b as usize].as_i64()?;
                    let owned = frame.arena.alloc(Value::Integer(av + bv));
                    regs[*dst as usize] = ValueRef::from_owned(owned);
                }
                Inst::EqI64 { dst, a, b } => {
                    let av = regs[*a as usize].as_i64()?;
                    let bv = regs[*b as usize].as_i64()?;
                    let owned = frame.arena.alloc(Value::Boolean(av == bv));
                    regs[*dst as usize] = ValueRef::from_owned(owned);
                }
                Inst::AndBool { dst, a, b } => {
                    let av = regs[*a as usize].as_bool()?;
                    let bv = regs[*b as usize].as_bool()?;
                    let owned = frame.arena.alloc(Value::Boolean(av && bv));
                    regs[*dst as usize] = ValueRef::from_owned(owned);
                }
                Inst::OrBool { dst, a, b } => {
                    let av = regs[*a as usize].as_bool()?;
                    let bv = regs[*b as usize].as_bool()?;
                    let owned = frame.arena.alloc(Value::Boolean(av || bv));
                    regs[*dst as usize] = ValueRef::from_owned(owned);
                }
                Inst::NotBool { dst, src } => {
                    let sv = regs[*src as usize].as_bool()?;
                    let owned = frame.arena.alloc(Value::Boolean(!sv));
                    regs[*dst as usize] = ValueRef::from_owned(owned);
                }
                Inst::GetField { dst, base, key_idx } => {
                    let key = self
                        .keys
                        .get(*key_idx as usize)
                        .ok_or_else(|| EngineError::IllegalState("invalid key index".to_string()))?;
                    regs[*dst as usize] = value_get_field(regs[*base as usize], key);
                }
                Inst::StoreSlot { slot, src } => {
                    frame.slots[*slot as usize] = SlotValue::Val(regs[*src as usize]);
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
                    let result = registry.call(name, &argv, frame.arena)?;
                    regs[*dst as usize] = result;
                }
            }
        }

        Ok(())
    }
}

pub trait UdfRegistry {
    fn call(&self, name: &str, args: &[ValueRef<'_>], arena: &RowArena) -> Result<ValueRef<'_>>;
}

pub trait SlotResolver {
    fn resolve_var(&self, name: &BindingsName<'_>, scope: VarRefType) -> Option<SlotId>;
    fn resolve_alias(&self, name: &BindingsName<'_>) -> Option<SlotId>;
    fn resolve_field(&self, name: &BindingsName<'_>) -> Option<SlotId>;
    fn is_alias(&self, name: &BindingsName<'_>) -> bool;
}

#[derive(Default)]
pub struct ProgramBuilder {
    insts: Vec<Inst>,
    consts: Vec<ValueOwned>,
    keys: Vec<String>,
    next_reg: u16,
}

impl ProgramBuilder {
    pub fn build(self) -> Program {
        Program {
            insts: self.insts,
            consts: self.consts,
            keys: self.keys,
            reg_count: self.next_reg,
        }
    }

    fn alloc_reg(&mut self) -> u16 {
        let reg = self.next_reg;
        self.next_reg = self
            .next_reg
            .checked_add(1)
            .expect("register overflow");
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
    pub fn new() -> Self {
        ExprCompiler {
            builder: ProgramBuilder::default(),
        }
    }

    pub fn compile_expr(&mut self, expr: &Expr) -> Result<u16> {
        match expr {
            Expr::Literal(value) => {
                let reg = self.builder.alloc_reg();
                let const_idx = self.builder.push_const(value.clone());
                self.builder
                    .insts
                    .push(Inst::LoadConst { dst: reg, const_idx });
                Ok(reg)
            }
            Expr::SlotRef(slot) => {
                let reg = self.builder.alloc_reg();
                self.builder
                    .insts
                    .push(Inst::LoadSlot { dst: reg, slot: *slot });
                Ok(reg)
            }
            Expr::Add(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::AddI64 { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::Eq(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::EqI64 { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::And(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                self.builder
                    .insts
                    .push(Inst::AndBool { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::Or(left, right) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let dst = self.builder.alloc_reg();
                self.builder
                    .insts
                    .push(Inst::OrBool { dst, a: l, b: r });
                Ok(dst)
            }
            Expr::Not(expr) => {
                let src = self.compile_expr(expr)?;
                let dst = self.builder.alloc_reg();
                self.builder.insts.push(Inst::NotBool { dst, src });
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
                self.builder
                    .insts
                    .push(Inst::CallUdf { dst, func_idx, args: arg_regs });
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

    pub fn compile_to_program(&self, expr: &ValueExpr, slot: SlotId) -> Result<Program> {
        let expr = self.lower_expr(expr)?;
        let mut compiler = ExprCompiler::new();
        compiler.compile_to_slot(&expr, slot)?;
        Ok(compiler.finish())
    }

    pub fn compile_to_program_multi(
        &self,
        exprs: &[(SlotId, ValueExpr)],
    ) -> Result<Program> {
        let mut compiler = ExprCompiler::new();
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
                    EngineError::UnsupportedExpr(format!("unresolved var {name:?}"))
                }),
            ValueExpr::DynamicLookup(lookups) => {
                for lookup in lookups.iter() {
                    if let Ok(expr) = self.lower_expr(lookup) {
                        return Ok(expr);
                    }
                }
                Err(EngineError::UnsupportedExpr(
                    "dynamic lookup".to_string(),
                ))
            }
            ValueExpr::Path(base, components) => {
                if let Some((is_alias, base_slot)) = resolve_alias_info(self.resolver, base) {
                    if let Some(PathComponent::Key(name)) = components.first() {
                        if let Some(slot) = self.resolver.resolve_field(name) {
                            return Ok(Expr::SlotRef(slot));
                        }
                    }

                    if is_alias {
                        if let Some(base_slot) = base_slot {
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
                                        return Err(EngineError::UnsupportedExpr(
                                            "unsupported path component".to_string(),
                                        ));
                                    }
                                }
                            }
                            Ok(current)
                        } else {
                            Err(EngineError::UnsupportedExpr(
                                "alias path requires base row slot".to_string(),
                            ))
                        }
                    } else {
                        Err(EngineError::UnsupportedExpr(
                            "unsupported path base".to_string(),
                        ))
                    }
                } else {
                    let mut current = self.lower_expr(base)?;
                    for component in components {
                        match component {
                            PathComponent::Key(name) => {
                                current = Expr::GetField(
                                    current.into(),
                                    bindings_name_to_string(name),
                                );
                            }
                            _ => {
                                return Err(EngineError::UnsupportedExpr(
                                    "unsupported path component".to_string(),
                                ));
                            }
                        }
                    }
                    Ok(current)
                }
            }
            ValueExpr::BinaryExpr(op, left, right) => {
                let left = self.lower_expr(left)?;
                let right = self.lower_expr(right)?;
                match op {
                    partiql_logical::BinaryOp::Add => Ok(Expr::Add(left.into(), right.into())),
                    partiql_logical::BinaryOp::Eq => Ok(Expr::Eq(left.into(), right.into())),
                    partiql_logical::BinaryOp::And => Ok(Expr::And(left.into(), right.into())),
                    partiql_logical::BinaryOp::Or => Ok(Expr::Or(left.into(), right.into())),
                    _ => Err(EngineError::UnsupportedExpr(format!(
                        "binary op {op:?}"
                    ))),
                }
            }
            ValueExpr::UnExpr(op, expr) => {
                let expr = self.lower_expr(expr)?;
                match op {
                    partiql_logical::UnaryOp::Not => Ok(Expr::Not(expr.into())),
                    _ => Err(EngineError::UnsupportedExpr(format!(
                        "unary op {op:?}"
                    ))),
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
            _ => Err(EngineError::UnsupportedExpr(format!("{:?}", *expr))),
        }
    }
}

fn lit_to_value(lit: &Lit) -> Result<ValueOwned> {
    match lit {
        Lit::Missing => Ok(Value::Missing),
        Lit::Null => Ok(Value::Null),
        Lit::Int64(v) => Ok(Value::Integer(*v)),
        Lit::Bool(v) => Ok(Value::Boolean(*v)),
        Lit::String(v) => Ok(Value::String(Box::new(v.clone()))),
        _ => Err(EngineError::UnsupportedExpr("literal".to_string())),
    }
}

fn call_name(call: &CallExpr) -> String {
    match &call.name {
        CallName::ByName(name) => name.clone(),
        CallName::ById(name, _, _) => name.clone(),
        other => format!("{other:?}"),
    }
}

fn bindings_name_to_string(name: &BindingsName<'_>) -> String {
    match name {
        BindingsName::CaseSensitive(s) => s.to_string(),
        BindingsName::CaseInsensitive(s) => s.to_string(),
    }
}

fn resolve_alias_slot<R: SlotResolver>(
    resolver: &R,
    base: &ValueExpr,
) -> Option<SlotId> {
    match base {
        ValueExpr::VarRef(name, _) => resolver.resolve_alias(name),
        ValueExpr::DynamicLookup(lookups) => lookups.iter().find_map(|lookup| {
            if let ValueExpr::VarRef(name, _) = lookup {
                resolver.resolve_alias(name)
            } else {
                None
            }
        }),
        _ => None,
    }
}

fn resolve_alias_info<R: SlotResolver>(
    resolver: &R,
    base: &ValueExpr,
) -> Option<(bool, Option<SlotId>)> {
    match base {
        ValueExpr::VarRef(name, _) => {
            if resolver.is_alias(name) {
                Some((true, resolver.resolve_alias(name)))
            } else {
                Some((false, None))
            }
        }
        ValueExpr::DynamicLookup(lookups) => lookups.iter().find_map(|lookup| {
            if let ValueExpr::VarRef(name, _) = lookup {
                if resolver.is_alias(name) {
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
