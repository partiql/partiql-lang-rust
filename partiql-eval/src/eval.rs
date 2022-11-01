use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Debug;

use petgraph::algo::toposort;
use petgraph::prelude::StableGraph;
use petgraph::{Directed, Incoming, Outgoing};

use partiql_value::Value::{Boolean, Missing, Null};
use partiql_value::{partiql_bag, Bag, BindingsName, Tuple, Value};

use crate::env::basic::MapBindings;
use crate::env::Bindings;

#[derive(Debug)]
pub struct EvalPlan(pub StableGraph<Box<dyn Evaluable>, (), Directed>);

impl Default for EvalPlan {
    fn default() -> Self {
        Self::new()
    }
}

impl EvalPlan {
    fn new() -> Self {
        EvalPlan(StableGraph::<Box<dyn Evaluable>, (), Directed>::new())
    }
}

pub type EvalResult = Result<Evaluated, EvalErr>;

pub struct Evaluated {
    pub result: Value,
}

pub struct EvalErr {
    pub errors: Vec<EvaluationError>,
}

pub enum EvaluationError {
    InvalidEvaluationPlan(String),
}

pub trait Evaluable: Debug {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value>;
    fn update_input(&mut self, input: &Value);
}

#[derive(Debug)]
pub struct EvalScan {
    pub expr: Box<dyn EvalExpr>,
    pub as_key: String,
    pub at_key: String,
    pub output: Option<Value>,
}

impl EvalScan {
    pub fn new(expr: Box<dyn EvalExpr>, as_key: &str, at_key: &str) -> Self {
        EvalScan {
            expr,
            as_key: as_key.to_string(),
            at_key: at_key.to_string(),
            output: None,
        }
    }
}

impl Evaluable for EvalScan {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let mut value = partiql_bag![];
        let v = self.expr.evaluate(&Tuple(HashMap::new()), ctx);
        let ordered = &v.is_ordered();
        let mut c: i64 = 0;
        let at_key = &self.at_key;
        for t in v.into_iter() {
            let mut out = HashMap::from([(self.as_key.clone(), t)]);
            if !at_key.is_empty() {
                let at_id = if *ordered { c.into() } else { Missing };
                out.insert(at_key.clone(), at_id.into());
                c += 1;
            }

            value.push(Value::Tuple(Box::new(Tuple(out))));
        }

        self.output = Some(Value::Bag(Box::new(value)));
        self.output.clone()
    }

    fn update_input(&mut self, _input: &Value) {
        todo!("update_input for Scan")
    }
}

#[derive(Debug)]
pub struct EvalUnpivot {
    pub expr: Box<dyn EvalExpr>,
    pub as_key: String,
    pub at_key: String,
    pub input: Option<Value>,
    pub output: Option<Value>,
}

impl EvalUnpivot {
    pub fn new(expr: Box<dyn EvalExpr>, as_key: &str, at_key: &str) -> Self {
        EvalUnpivot {
            expr,
            as_key: as_key.to_string(),
            at_key: at_key.to_string(),
            input: None,
            output: None,
        }
    }
}

impl Evaluable for EvalUnpivot {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let result = self.expr.evaluate(&Tuple(HashMap::new()), ctx);
        let mut out = vec![];

        let tuple = match result {
            Value::Tuple(tuple) => *tuple,
            other => other.coerce_to_tuple(),
        };

        let unpivoted = tuple.0.into_iter().map(|(k, v)| {
            Tuple::from([(self.as_key.as_str(), v), (self.at_key.as_str(), k.into())])
        });

        for t in unpivoted {
            out.push(Value::Tuple(Box::new(t)));
        }

        self.output = Some(Value::Bag(Box::new(Bag::from(out))));
        self.output.clone()
    }

    fn update_input(&mut self, input: &Value) {
        self.input = Some(input.clone());
    }
}

#[derive(Debug)]
pub struct EvalFilter {
    pub expr: Box<dyn EvalExpr>,
    pub input: Option<Value>,
    pub output: Option<Value>,
}

impl EvalFilter {
    pub fn new(expr: Box<dyn EvalExpr>) -> Self {
        EvalFilter {
            expr,
            input: None,
            output: None,
        }
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

impl Evaluable for EvalFilter {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self
            .input
            .as_ref()
            .expect("Error in retrieving input value")
            .clone();

        let mut out = partiql_bag![];
        for v in input_value.into_iter() {
            if self.eval_filter(&v.clone().coerce_to_tuple(), ctx) {
                out.push(v);
            }
        }

        self.output = Some(Value::Bag(Box::new(out)));
        self.output.clone()
    }
    fn update_input(&mut self, input: &Value) {
        self.input = Some(input.clone())
    }
}

#[derive(Debug)]
pub struct EvalProject {
    pub exprs: HashMap<String, Box<dyn EvalExpr>>,
    pub input: Option<Value>,
    pub output: Option<Value>,
}

impl EvalProject {
    pub fn new(exprs: HashMap<String, Box<dyn EvalExpr>>) -> Self {
        EvalProject {
            exprs,
            input: None,
            output: None,
        }
    }
}

impl Evaluable for EvalProject {
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

        self.output = Some(Value::Bag(Box::new(value)));
        self.output.clone()
    }
    fn update_input(&mut self, input: &Value) {
        self.input = Some(input.clone());
    }
}

#[derive(Debug)]
pub enum EvalPathComponent {
    Key(String),
    Index(i64),
}

#[derive(Debug)]
pub struct EvalPath {
    pub expr: Box<dyn EvalExpr>,
    pub components: Vec<EvalPathComponent>,
}

impl EvalExpr for EvalPath {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        #[inline]
        fn path_into(value: Value, path: &EvalPathComponent) -> Value {
            match path {
                EvalPathComponent::Key(s) => match value {
                    Value::Tuple(mut tuple) => tuple.0.remove(s).unwrap_or(Missing),
                    _ => Missing,
                },
                EvalPathComponent::Index(idx) => match value {
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
pub struct EvalDistinct {
    pub input: Option<Value>,
    pub output: Option<Value>,
}

impl EvalDistinct {
    pub fn new() -> Self {
        EvalDistinct {
            input: None,
            output: None,
        }
    }
}

impl Evaluable for EvalDistinct {
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Option<Value> {
        let out = self.input.clone().unwrap();
        let u: Vec<Value> = out.into_iter().unique().collect();
        self.output = Some(Value::Bag(Box::new(Bag::from(u))));
        self.output.clone()
    }

    fn update_input(&mut self, input: &Value) {
        self.input = Some(input.clone());
    }
}

#[derive(Debug)]
pub struct EvalSink {
    pub input: Option<Value>,
    pub output: Option<Value>,
}

impl Evaluable for EvalSink {
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Option<Value> {
        self.input.clone()
    }
    fn update_input(&mut self, input: &Value) {
        self.input = Some(input.clone());
    }
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
pub struct EvalBinOpExpr {
    pub op: EvalBinop,
    pub lhs: Box<dyn EvalExpr>,
    pub rhs: Box<dyn EvalExpr>,
}

// TODO we should replace this enum with some identifier that can be looked up in a symtab/funcregistry
#[derive(Debug)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum EvalBinop {
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
        // Missing and Null propagation. Missing has precedence over Null
        if lhs == Value::Missing || rhs == Value::Missing {
            Value::Missing
        } else if lhs == Value::Null || rhs == Value::Null {
            Value::Null
        } else {
            match self.op {
                EvalBinop::And => todo!(),
                EvalBinop::Or => todo!(),
                EvalBinop::Concat => {
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
                EvalBinop::Eq => todo!(),
                EvalBinop::Neq => todo!(),
                EvalBinop::Gt => Boolean(lhs > rhs),
                EvalBinop::Gteq => Boolean(lhs >= rhs),
                EvalBinop::Lt => Boolean(lhs < rhs),
                EvalBinop::Lteq => Boolean(lhs <= rhs),
                EvalBinop::Add => lhs + rhs,
                EvalBinop::Sub => lhs - rhs,
                EvalBinop::Mul => lhs * rhs,
                EvalBinop::Div => lhs / rhs,
                EvalBinop::Mod => lhs % rhs,
                EvalBinop::Exp => todo!("Exponentiation"),
            }
        }
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
    ctx: Box<dyn EvalContext>,
}

impl Evaluator {
    pub fn new(bindings: MapBindings<Value>) -> Self {
        let ctx: Box<dyn EvalContext> = Box::new(BasicContext { bindings });
        Evaluator { ctx }
    }

    pub fn execute(&mut self, plan: EvalPlan) -> Result<Evaluated, EvalErr> {
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
                        let evaluated = Evaluated {
                            result: result.expect("Error in retrieving eval output"),
                        };
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
