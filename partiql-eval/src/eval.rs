use itertools::Itertools;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;

use petgraph::algo::toposort;
use petgraph::prelude::StableGraph;
use petgraph::{Directed, Incoming, Outgoing};

use partiql_value::Value::{Boolean, Missing, Null};
use partiql_value::{partiql_bag, Bag, BindingsName, Tuple, Value};

use crate::env::basic::MapBindings;
use crate::env::Bindings;

#[derive(Debug)]
pub struct EvalPlan(pub StableGraph<Box<dyn DagEvaluable>, (), Directed>);

impl Default for EvalPlan {
    fn default() -> Self {
        Self::new()
    }
}

impl EvalPlan {
    fn new() -> Self {
        EvalPlan(StableGraph::<Box<dyn DagEvaluable>, (), Directed>::new())
    }
}

pub type EvalResult = Result<Evaluated, EvalErr>;

pub struct Evaluated {
    pub result: Option<Value>,
}

pub struct EvalErr {
    pub errors: Vec<EvaluationError>,
}

pub enum EvaluationError {
    InvalidEvaluationPlan(String),
}

#[derive(Debug)]
pub struct Scan {
    pub expr: Box<dyn EvalExpr>,
    pub as_key: String,
    pub output: Option<Value>,
}

impl Scan {
    pub fn new(expr: Box<dyn EvalExpr>, as_key: &str) -> Self {
        Scan {
            expr,
            as_key: as_key.to_string(),
            output: None,
        }
    }
}

impl DagEvaluable for Scan {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let mut value = partiql_bag!();
        let v = self.expr.evaluate(&Tuple(HashMap::new()), ctx);
        for t in v.into_iter() {
            let out = Tuple(HashMap::from([(self.as_key.clone(), t)]));
            value.push(Value::Tuple(Box::new(out)));
        }
        Some(Value::Bag(Box::new(value)))
    }
    fn update_input(&mut self, _input: &Value) {
        todo!("update_input for Scan")
    }
}

#[derive(Debug)]
pub struct Project {
    pub exprs: HashMap<String, Box<dyn EvalExpr>>,
    pub input: Option<Value>,
    pub output: Option<Value>,
}

impl Project {
    pub fn new(exprs: HashMap<String, Box<dyn EvalExpr>>) -> Self {
        Project {
            exprs,
            input: None,
            output: None,
        }
    }
}

impl DagEvaluable for Project {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self
            .input
            .as_ref()
            .expect("Error in retrieving input value")
            .clone();
        let mut value = partiql_bag![];
        for v in input_value.into_iter() {
            let out = v.coerce_to_tuple();

            let proj: HashMap<String, Value> = self
                .exprs
                .iter()
                .map(|(alias, expr)| (alias.to_string(), expr.evaluate(&out, ctx)))
                .collect();
            value.push(Value::Tuple(Box::new(Tuple(proj))));
        }

        Some(Value::Bag(Box::new(value)))
    }
    fn update_input(&mut self, input: &Value) {
        self.input = Some(input.clone());
    }
}

#[derive(Debug)]
pub struct Sink {
    pub input: Option<Value>,
    pub output: Option<Value>,
}

impl DagEvaluable for Sink {
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Option<Value> {
        self.input.clone()
    }
    fn update_input(&mut self, input: &Value) {
        self.input = Some(input.clone());
    }
}

pub trait EvalContext {
    fn bindings(&self) -> &dyn Bindings<Value>;
}

#[derive(Default, Debug)]
pub struct BasicContext {
    bindings: MapBindings<Value>,
}

impl BasicContext {
    pub fn new(bindings: MapBindings<Value>) -> Self {
        BasicContext { bindings }
    }
}

impl EvalContext for BasicContext {
    fn bindings(&self) -> &dyn Bindings<Value> {
        &self.bindings
    }
}

pub struct Evaluator {
    evaluable: Box<dyn Evaluable>,
    ctx: Box<dyn EvalContext>,
}

impl Evaluator {
    pub fn new(bindings: MapBindings<Value>, evaluable: Box<dyn Evaluable>) -> Self {
        let ctx: Box<dyn EvalContext> = Box::new(BasicContext { bindings });
        Evaluator { evaluable, ctx }
    }

    pub fn execute(&mut self) {
        self.evaluable.evaluate(&*self.ctx);
    }
}

pub struct DagEvaluator {
    ctx: Box<dyn EvalContext>,
}

impl DagEvaluator {
    pub fn new(bindings: MapBindings<Value>) -> Self {
        let ctx: Box<dyn EvalContext> = Box::new(BasicContext { bindings });
        DagEvaluator { ctx }
    }

    pub fn execute_dag(&mut self, plan: EvalPlan) -> Result<Evaluated, EvalErr> {
        let mut graph = plan.0;
        // We are only interested in DAGs that can be used as execution plans, which leads to the
        // following definition.
        // A DAG is a directed, cycle-free graph G = (V, E) with a denoted root node v0 ∈ V such
        // that all v ∈ V \{v0} are reachable from v0. Note that this is the definition of trees
        // without the condition |E| = |V | − 1. Hence, all trees are DAGs.
        // Reference: https://link.springer.com/article/10.1007/s00450-009-0061-0
        match graph.externals(Incoming).exactly_one() {
            Ok(_) => {
                let sorted_ops = toposort(&graph, None);
                match sorted_ops {
                    Ok(ops) => {
                        let mut result = None;
                        for idx in ops.into_iter() {
                            let src = graph
                                .node_weight_mut(idx)
                                .expect("Error in retrieving node");
                            result = src.evaluate(&*self.ctx);

                            let mut ne = graph.neighbors_directed(idx, Outgoing).detach();
                            while let Some(n) = ne.next_node(&graph) {
                                let dst =
                                    graph.node_weight_mut(n).expect("Error in retrieving node");
                                dst.update_input(
                                    &result.clone().expect("Error in retrieving source value"),
                                );
                            }
                        }
                        let evaluated = Evaluated { result };
                        Ok(evaluated)
                    }
                    Err(e) => Err(EvalErr {
                        errors: vec![EvaluationError::InvalidEvaluationPlan(format!(
                            "Malformed evaluation plan detected: {:?}",
                            e
                        ))],
                    }),
                }
            }
            Err(e) => Err(EvalErr {
                errors: vec![EvaluationError::InvalidEvaluationPlan(format!(
                    "Malformed evaluation plan detected: {:?}",
                    e
                ))],
            }),
        }
    }
}

pub trait Evaluable: Debug {
    fn evaluate(&mut self, ctx: &dyn EvalContext);
}

// TODO rename to `Evaluable` when moved to DAG model completely
pub trait DagEvaluable: Debug {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value>;
    fn update_input(&mut self, input: &Value);
}

pub trait TupleSink: Debug {
    fn push_tuple(&mut self, bindings: Tuple, ctx: &dyn EvalContext);
}

pub trait ValueSink: Debug {
    fn push_value(&mut self, value: Value, ctx: &dyn EvalContext);
}

pub trait EvalExpr: Debug {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value;
}

#[derive(Debug)]
pub struct EvalVarRef {
    pub name: BindingsName,
}

impl EvalExpr for EvalVarRef {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let value = bindings
            .get(&self.name)
            .or_else(|| ctx.bindings().get(&self.name));
        value.map_or(Null, |v| v.clone())
    }
}

#[derive(Debug)]
pub struct EvalLitExpr {
    pub lit: Box<Value>,
}

impl EvalExpr for EvalLitExpr {
    fn evaluate(&self, _bindings: &Tuple, _ctx: &dyn EvalContext) -> Value {
        *self.lit.clone()
    }
}

#[derive(Debug)]
pub struct EvalUnaryOpExpr {
    pub op: EvalUnaryOp,
    pub operand: Box<dyn EvalExpr>,
}

#[derive(Debug)]
pub struct EvalBinOpExpr {
    pub op: EvalBinOp,
    pub lhs: Box<dyn EvalExpr>,
    pub rhs: Box<dyn EvalExpr>,
}

// TODO we should replace this enum with some identifier that can be looked up in a symtab/funcregistry
#[derive(Debug)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum EvalUnaryOp {
    Pos,
    Neg,
    Not,
}

impl EvalExpr for EvalUnaryOpExpr {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let value = self.operand.evaluate(bindings, ctx);
        match self.op {
            EvalUnaryOp::Pos => match value {
                Value::Null => Value::Null,
                Value::Missing => Value::Missing,
                Value::Integer(_) | Value::Real(_) | Value::Decimal(_) => value,
                _ => Value::Missing, // data type mismatch => Missing
            },
            EvalUnaryOp::Neg => -value,
            EvalUnaryOp::Not => !value,
        }
    }
}

// TODO we should replace this enum with some identifier that can be looked up in a symtab/funcregistry
#[derive(Debug)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum EvalBinOp {
    And,
    Or,
    Concat,
    Eq,
    Neq,
    Gt,
    Gteq,
    Lt,
    Lteq,

    // Arithmetic ops
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
}

impl EvalExpr for EvalBinOpExpr {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let lhs = self.lhs.evaluate(bindings, ctx);
        let rhs = self.rhs.evaluate(bindings, ctx);
        match self.op {
            EvalBinOp::And => {
                match (&lhs, &rhs) {
                    (Value::Boolean(l), Value::Boolean(r)) => Value::from(*l && *r),
                    (Value::Null, Value::Boolean(false))    // short-circuiting
                    | (Value::Boolean(false), Value::Null)
                    | (Value::Missing, Value::Boolean(false))
                    | (Value::Boolean(false), Value::Missing) => Value::from(false),
                    _ => if matches!(lhs, Value::Missing | Value::Null | Value::Boolean(true)) && matches!(rhs, Value::Missing | Value::Null | Value::Boolean(true)) {
                        Value::Null
                    } else {
                        Value::Missing
                    }
                }
            }
            EvalBinOp::Or => {
                match (&lhs, &rhs) {
                    (Value::Boolean(l), Value::Boolean(r)) => Value::from(*l || *r),
                    (Value::Null, Value::Boolean(true))     // short-circuiting
                    | (Value::Boolean(true), Value::Null)
                    | (Value::Missing, Value::Boolean(true))
                    | (Value::Boolean(true), Value::Missing) => Value::from(true),
                    _ => if matches!(lhs, Value::Missing | Value::Null | Value::Boolean(false)) && matches!(rhs, Value::Missing | Value::Null | Value::Boolean(false)) {
                        Value::Null
                    } else {
                        Value::Missing
                    }
                }
            }
            _ => {
                // Missing and Null propagation. Missing has precedence over Null
                if lhs == Value::Missing || rhs == Value::Missing {
                    Value::Missing
                } else if lhs == Value::Null || rhs == Value::Null {
                    Value::Null
                } else {
                    match self.op {
                        EvalBinOp::Concat => {
                            // TODO non-naive concat
                            let lhs = if let Value::String(s) = lhs {
                                *s
                            } else {
                                format!("{:?}", lhs)
                            };
                            let rhs = if let Value::String(s) = rhs {
                                *s
                            } else {
                                format!("{:?}", lhs)
                            };
                            Value::String(Box::new(format!("{}{}", lhs, rhs)))
                        }
                        EvalBinOp::Eq => Value::from(lhs == rhs),
                        EvalBinOp::Neq => Value::from(lhs != rhs),
                        EvalBinOp::Gt => Boolean(lhs > rhs),
                        EvalBinOp::Gteq => Boolean(lhs >= rhs),
                        EvalBinOp::Lt => Boolean(lhs < rhs),
                        EvalBinOp::Lteq => Boolean(lhs <= rhs),
                        EvalBinOp::Add => lhs + rhs,
                        EvalBinOp::Sub => lhs - rhs,
                        EvalBinOp::Mul => lhs * rhs,
                        EvalBinOp::Div => lhs / rhs,
                        EvalBinOp::Mod => lhs % rhs,
                        EvalBinOp::Exp => todo!("Exponentiation"),
                        EvalBinOp::And | EvalBinOp::Or => {
                            unreachable!("Ops already covered")
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum PathComponent {
    Key(String),
    Index(i64),
}

#[derive(Debug)]
pub struct EvalPath {
    pub expr: Box<dyn EvalExpr>,
    pub components: Vec<PathComponent>,
}

impl EvalExpr for EvalPath {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        #[inline]
        fn path_into(value: Value, path: &PathComponent) -> Value {
            match path {
                PathComponent::Key(s) => match value {
                    Value::Tuple(mut tuple) => tuple.0.remove(s).unwrap_or(Missing),
                    _ => Missing,
                },
                PathComponent::Index(idx) => match value {
                    Value::List(mut list) if (*idx as usize) < list.len() => {
                        std::mem::take(list.get_mut(*idx).unwrap())
                    }
                    _ => Missing,
                },
            }
        }

        let mut value = self.expr.evaluate(bindings, ctx);

        for path in &self.components {
            value = path_into(value, path);
        }
        value
    }
}

#[derive(Debug)]
pub struct EvalScan {
    pub expr: Box<dyn EvalExpr>,
    pub output: Box<dyn TupleSink>,
}

impl EvalScan {
    pub fn new(expr: Box<dyn EvalExpr>, output: Box<dyn TupleSink>) -> Self {
        EvalScan { expr, output }
    }
}

impl TupleSink for EvalScan {
    fn push_tuple(&mut self, bindings: Tuple, ctx: &dyn EvalContext) {
        let result = self.expr.evaluate(&bindings, ctx);
        for v in result.into_iter() {
            self.output.push_tuple(v.coerce_to_tuple(), ctx);
        }
    }
}

impl Evaluable for EvalScan {
    fn evaluate(&mut self, ctx: &dyn EvalContext) {
        let empty = Tuple(HashMap::new());
        self.push_tuple(empty, ctx);
    }
}

#[derive(Debug)]
pub struct EvalFrom {
    pub expr: Box<dyn EvalExpr>,
    pub as_key: String,
    pub output: Box<dyn TupleSink>,
}

impl EvalFrom {
    pub fn new(expr: Box<dyn EvalExpr>, as_key: &str, output: Box<dyn TupleSink>) -> Self {
        EvalFrom {
            expr,
            as_key: as_key.to_string(),
            output,
        }
    }
}

impl Evaluable for EvalFrom {
    fn evaluate(&mut self, ctx: &dyn EvalContext) {
        let empty = Tuple(HashMap::new());
        self.push_tuple(empty, ctx);
    }
}

impl TupleSink for EvalFrom {
    #[inline]
    fn push_tuple(&mut self, bindings: Tuple, ctx: &dyn EvalContext) {
        self.push_value(self.expr.evaluate(&bindings, ctx), ctx);
    }
}

impl ValueSink for EvalFrom {
    #[inline]
    fn push_value(&mut self, value: Value, ctx: &dyn EvalContext) {
        for v in value.into_iter() {
            let out = Tuple(HashMap::from([(self.as_key.clone(), v)]));
            self.output.push_tuple(out, ctx);
        }
    }
}

#[derive(Debug)]
pub struct EvalFromAt {
    expr: Box<dyn EvalExpr>,
    as_key: String,
    at_key: String,
    output: Box<dyn TupleSink>,
    curr: i64,
}

impl EvalFromAt {
    pub fn new(
        expr: Box<dyn EvalExpr>,
        as_key: &str,
        at_key: &str,
        output: Box<dyn TupleSink>,
    ) -> Self {
        EvalFromAt {
            expr,
            as_key: as_key.to_string(),
            at_key: at_key.to_string(),
            output,
            curr: 0,
        }
    }
}

impl TupleSink for EvalFromAt {
    fn push_tuple(&mut self, bindings: Tuple, ctx: &dyn EvalContext) {
        self.push_value(self.expr.evaluate(&bindings, ctx), ctx);
    }
}

impl ValueSink for EvalFromAt {
    #[inline]
    fn push_value(&mut self, value: Value, ctx: &dyn EvalContext) {
        let ordered = value.is_ordered();

        for v in value.into_iter() {
            let at_id = if ordered {
                self.next_at().into()
            } else {
                Missing
            };
            let out = Tuple(HashMap::from([
                (self.as_key.clone(), v.coerce_to_tuple().into()),
                (self.at_key.clone(), at_id),
            ]));
            self.output.push_tuple(out, ctx);
        }
    }
}

impl EvalFromAt {
    #[inline]
    fn next_at(&mut self) -> i64 {
        let at = self.curr;
        self.curr += 1;
        at
    }
}

impl Evaluable for EvalFromAt {
    fn evaluate(&mut self, ctx: &dyn EvalContext) {
        let empty = Tuple(HashMap::new());
        self.push_tuple(empty, ctx);
    }
}

#[derive(Debug)]
pub struct EvalUnpivot {
    expr: Box<dyn EvalExpr>,
    as_key: String,
    at_key: String,
    output: Box<dyn TupleSink>,
}

impl EvalUnpivot {
    pub fn new(
        expr: Box<dyn EvalExpr>,
        as_key: &str,
        at_key: &str,
        output: Box<dyn TupleSink>,
    ) -> Self {
        EvalUnpivot {
            expr,
            as_key: as_key.to_string(),
            at_key: at_key.to_string(),
            output,
        }
    }
}

impl TupleSink for EvalUnpivot {
    fn push_tuple(&mut self, bindings: Tuple, ctx: &dyn EvalContext) {
        let result = self.expr.evaluate(&bindings, ctx);

        let tuple = match result {
            Value::Tuple(tuple) => *tuple,
            other => other.coerce_to_tuple(),
        };

        let unpivoted = tuple.0.into_iter().map(|(k, v)| {
            Tuple::from([(self.as_key.as_str(), v), (self.at_key.as_str(), k.into())])
        });
        for t in unpivoted {
            self.output.push_tuple(t, ctx);
        }
    }
}

impl Evaluable for EvalUnpivot {
    fn evaluate(&mut self, ctx: &dyn EvalContext) {
        let empty = Tuple(HashMap::new());
        self.push_tuple(empty, ctx);
    }
}

#[derive(Debug)]
pub struct EvalWhere {
    pub expr: Box<dyn EvalExpr>,
    pub output: Box<dyn TupleSink>,
}

impl EvalWhere {
    pub fn new(expr: Box<dyn EvalExpr>, output: Box<dyn TupleSink>) -> Self {
        EvalWhere { expr, output }
    }

    #[inline]
    fn eval_filter(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> bool {
        let result = self.expr.evaluate(bindings, ctx);
        match result {
            Boolean(bool_val) => bool_val,
            _ => panic!("invalid filter -- not boolean"),
        }
    }
}

impl TupleSink for EvalWhere {
    fn push_tuple(&mut self, bindings: Tuple, ctx: &dyn EvalContext) {
        if self.eval_filter(&bindings, ctx) {
            self.output.push_tuple(bindings, ctx);
        }
    }
}

#[derive(Debug)]
pub struct EvalSelect {
    pub exprs: HashMap<String, Box<dyn EvalExpr>>,
    pub output: Box<dyn TupleSink>,
}

impl EvalSelect {
    pub fn new(exprs: HashMap<String, Box<dyn EvalExpr>>, output: Box<dyn TupleSink>) -> Self {
        EvalSelect { exprs, output }
    }
}

impl TupleSink for EvalSelect {
    fn push_tuple(&mut self, bindings: Tuple, ctx: &dyn EvalContext) {
        let proj = self
            .exprs
            .iter()
            .map(|(alias, expr)| (alias.to_string(), expr.evaluate(&bindings, ctx)))
            .collect();
        let out = Tuple(proj);
        self.output.push_tuple(out, ctx)
    }
}

#[derive(Debug)]
pub struct EvalDistinct {
    pub seen: HashSet<Tuple>,
    pub output: Box<dyn TupleSink>,
}

impl EvalDistinct {
    pub fn new(output: Box<dyn TupleSink>) -> Self {
        let seen = HashSet::new();
        EvalDistinct { seen, output }
    }
}

impl TupleSink for EvalDistinct {
    fn push_tuple(&mut self, bindings: Tuple, ctx: &dyn EvalContext) {
        let is_new = self.seen.insert(bindings.clone());
        if is_new {
            self.output.push_tuple(bindings, ctx)
        }
    }
}

#[derive(Default, Debug)]
pub struct EvalOutputAccumulator {
    pub output: Bag,
}

impl TupleSink for EvalOutputAccumulator {
    #[inline]
    fn push_tuple(&mut self, bindings: Tuple, ctx: &dyn EvalContext) {
        self.push_value(Value::Tuple(Box::new(bindings)), ctx);
    }
}

impl ValueSink for EvalOutputAccumulator {
    #[inline]
    fn push_value(&mut self, value: Value, _ctx: &dyn EvalContext) {
        self.output.push(value);
    }
}

#[derive(Debug)]
pub struct Output {
    pub output: Rc<RefCell<dyn TupleSink>>,
}

impl TupleSink for Output {
    fn push_tuple(&mut self, bindings: Tuple, ctx: &dyn EvalContext) {
        self.output.borrow_mut().push_tuple(bindings, ctx);
    }
}
