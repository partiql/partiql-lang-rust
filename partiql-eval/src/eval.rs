use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Debug;

use thiserror::Error;

use petgraph::algo::toposort;
use petgraph::data::DataMapMut;
use petgraph::prelude::StableGraph;
use petgraph::{Directed, Outgoing};

use partiql_value::Value::{Boolean, Missing, Null};
use partiql_value::{
    partiql_bag, Bag, BinaryAnd, BinaryOr, BindingsName, List, NullableEq, NullableOrd, Tuple,
    UnaryPlus, Value,
};

use crate::env::basic::MapBindings;
use crate::env::Bindings;

#[derive(Debug)]
pub struct EvalPlan(pub StableGraph<Box<dyn Evaluable>, u8, Directed>);

impl Default for EvalPlan {
    fn default() -> Self {
        Self::new()
    }
}

impl EvalPlan {
    fn new() -> Self {
        EvalPlan(StableGraph::<Box<dyn Evaluable>, u8, Directed>::new())
    }
}

pub type EvalResult = Result<Evaluated, EvalErr>;

pub struct Evaluated {
    pub result: Value,
}

pub struct EvalErr {
    pub errors: Vec<EvaluationError>,
}

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("Evaluation Error: malformed evaluation plan detected `{}`", _0)]
    InvalidEvaluationPlan(String),
}

pub trait Evaluable: Debug {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value>;
    fn update_input(&mut self, input: &Value, branch_num: u8);
}

#[derive(Debug)]
pub struct EvalScan {
    pub expr: Box<dyn EvalExpr>,
    pub as_key: String,
    pub at_key: Option<String>,
    pub output: Option<Value>,
}

impl EvalScan {
    pub fn new(expr: Box<dyn EvalExpr>, as_key: &str) -> Self {
        EvalScan {
            expr,
            as_key: as_key.to_string(),
            at_key: None,
            output: None,
        }
    }
    pub fn new_with_at_key(expr: Box<dyn EvalExpr>, as_key: &str, at_key: &str) -> Self {
        EvalScan {
            expr,
            as_key: as_key.to_string(),
            at_key: Some(at_key.to_string()),
            output: None,
        }
    }
}

impl Evaluable for EvalScan {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let mut value = partiql_bag![];
        let v = self.expr.evaluate(&Tuple::new(), ctx);
        let ordered = &v.is_ordered();
        let mut at_index_counter: i64 = 0;
        if let Some(at_key) = &self.at_key {
            for t in v.into_iter() {
                let mut out = Tuple::from([(self.as_key.as_str(), t)]);
                let at_id = if *ordered {
                    at_index_counter.into()
                } else {
                    Missing
                };
                out.insert(at_key, at_id);
                value.push(Value::Tuple(Box::new(out)));
                at_index_counter += 1;
            }
        } else {
            for t in v.into_iter() {
                let out = Tuple::from([(self.as_key.as_str(), t)]);
                value.push(Value::Tuple(Box::new(out)));
            }
        }
        self.output = Some(Value::Bag(Box::new(value)));
        self.output.clone()
    }

    fn update_input(&mut self, _input: &Value, _branch_num: u8) {
        todo!("update_input for Scan")
    }
}

#[derive(Debug)]
pub enum EvalJoinKind {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

#[derive(Debug)]
pub struct EvalJoin {
    pub kind: EvalJoinKind,
    pub on: Option<Box<dyn EvalExpr>>,
    pub input_l: Option<Value>,
    pub input_r: Option<Value>,
    pub output: Option<Value>,
}

impl EvalJoin {
    pub fn new(kind: EvalJoinKind, on: Option<Box<dyn EvalExpr>>) -> Self {
        EvalJoin {
            kind,
            on,
            input_l: None,
            input_r: None,
            output: None,
        }
    }
}

impl Evaluable for EvalJoin {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let l_vals = self.input_l.as_ref().unwrap().iter();
        let r_vals = self.input_r.as_ref().unwrap().iter();

        #[inline]
        fn cross<'a>(
            left: impl Iterator<Item = &'a Value> + Clone + 'a,
            right: impl Iterator<Item = &'a Value> + Clone + 'a,
        ) -> impl Iterator<Item = Tuple> + 'a {
            left.cartesian_product(right).map(|(l_tuple, r_tuple)| {
                l_tuple
                    .as_tuple_ref()
                    .pairs()
                    .chain(r_tuple.as_tuple_ref().pairs())
                    .map(|(a, v)| (a, v.clone()))
                    .collect::<Tuple>()
            })
        }

        // TODO: PartiQL defaults to lateral JOINs (RHS can reference binding tuples defined from the LHS)
        //  https://partiql.org/assets/PartiQL-Specification.pdf#subsection.5.3. Adding this behavior
        //  to be spec-compliant may result in changes to the DAG flows.
        let output: Bag = match self.kind {
            EvalJoinKind::Inner => match &self.on {
                None => cross(l_vals, r_vals).collect(),
                Some(condition) => cross(l_vals, r_vals)
                    .filter(|t| matches!(condition.evaluate(&t, ctx), Value::Boolean(true)))
                    .collect(),
            },
            EvalJoinKind::Left => {
                todo!("Left JOINs")
            }
            EvalJoinKind::Cross => cross(l_vals, r_vals).collect(),
            EvalJoinKind::Full | EvalJoinKind::Right => {
                todo!("Full and Right Joins are not yet implemented for `partiql-lang-rust`")
            }
        };
        self.output = Some(output.into());
        self.output.clone()
    }

    fn update_input(&mut self, input: &Value, branch_num: u8) {
        match branch_num {
            0 => self.input_l = Some(input.clone()),
            1 => self.input_r = Some(input.clone()),
            _ => panic!("EvalJoin nodes only support `0` and `1` for the `branch_num`"),
        };
    }
}

#[derive(Debug)]
pub struct EvalUnpivot {
    pub expr: Box<dyn EvalExpr>,
    pub as_key: String,
    pub at_key: String,
    pub output: Option<Value>,
}

impl EvalUnpivot {
    pub fn new(expr: Box<dyn EvalExpr>, as_key: &str, at_key: &str) -> Self {
        EvalUnpivot {
            expr,
            as_key: as_key.to_string(),
            at_key: at_key.to_string(),
            output: None,
        }
    }
}

impl Evaluable for EvalUnpivot {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let result = self.expr.evaluate(&Tuple::new(), ctx);
        let mut out = vec![];

        let tuple = match result {
            Value::Tuple(tuple) => *tuple,
            other => other.coerce_to_tuple(),
        };

        let unpivoted = tuple.into_iter().map(|(k, v)| {
            Tuple::from([(self.as_key.as_str(), v), (self.at_key.as_str(), k.into())])
        });

        for t in unpivoted {
            out.push(Value::Tuple(Box::new(t)));
        }

        self.output = Some(Value::Bag(Box::new(Bag::from(out))));
        self.output.clone()
    }

    fn update_input(&mut self, _input: &Value, _branch_num: u8) {
        todo!()
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
            // Alike SQL, when the expression of the WHERE clause expression evaluates to
            // absent value or a value that is not a Boolean, PartiQL eliminates the corresponding
            // binding. PartiQL Specification August 1, 2019 Draft, Section 8. `WHERE clause`
            _ => false,
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
    fn update_input(&mut self, input: &Value, _branch_num: u8) {
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

        let ordered = &input_value.is_ordered();
        let mut value = vec![];

        for v in input_value.into_iter() {
            let v_as_tuple = v.coerce_to_tuple();
            let mut t = Tuple::new();

            self.exprs.iter().for_each(|(alias, expr)| {
                let evaluated_val = expr.evaluate(&v_as_tuple, ctx);
                if evaluated_val != Missing {
                    // Per section 2 of PartiQL spec: "value MISSING may not appear as an attribute value
                    t.insert(alias.as_str(), evaluated_val);
                }
            });
            value.push(Value::Tuple(Box::new(t)));
        }

        self.output = match ordered {
            true => Some(Value::List(Box::new(List::from(value)))),
            false => Some(Value::Bag(Box::new(Bag::from(value)))),
        };

        self.output.clone()
    }

    fn update_input(&mut self, input: &Value, _branch_num: u8) {
        self.input = Some(input.clone());
    }
}

#[derive(Debug)]
pub struct EvalProjectValue {
    pub expr: Box<dyn EvalExpr>,
    pub input: Option<Value>,
    pub output: Option<Value>,
}

impl EvalProjectValue {
    pub fn new(expr: Box<dyn EvalExpr>) -> Self {
        EvalProjectValue {
            expr,
            input: None,
            output: None,
        }
    }
}

impl Evaluable for EvalProjectValue {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self
            .input
            .as_ref()
            .expect("Error in retrieving input value")
            .clone();

        let ordered = &input_value.is_ordered();
        let mut value = vec![];

        for v in input_value.into_iter() {
            let out = v.coerce_to_tuple();
            let evaluated = self.expr.evaluate(&out, ctx);
            value.push(evaluated);
        }

        self.output = match ordered {
            true => Some(Value::List(Box::new(List::from(value)))),
            false => Some(Value::Bag(Box::new(Bag::from(value)))),
        };

        self.output.clone()
    }

    fn update_input(&mut self, input: &Value, _branch_num: u8) {
        self.input = Some(input.clone());
    }
}

#[derive(Debug)]
pub struct EvalTupleExpr {
    pub attrs: Vec<Box<dyn EvalExpr>>,
    pub vals: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalTupleExpr {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let mut t = Tuple::new();
        self.attrs
            .iter()
            .filter_map(|attr| {
                let expr = attr.evaluate(bindings, ctx);
                match expr {
                    Value::String(s) => Some(*s),
                    _ => None,
                }
            })
            .zip(self.vals.iter())
            .for_each(|(k, v)| {
                let evaluated = v.evaluate(bindings, ctx);
                // Spec. section 6.1.4
                if evaluated != Value::Missing {
                    t.insert(k.as_str(), evaluated);
                }
            });

        Value::Tuple(Box::new(t))
    }
}

#[derive(Debug)]
pub struct EvalListExpr {
    pub elements: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalListExpr {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let evaluated_elements: Vec<Value> = self
            .elements
            .iter()
            .map(|val| val.evaluate(bindings, ctx))
            .collect();

        Value::List(Box::new(List::from(evaluated_elements)))
    }
}

#[derive(Debug)]
pub struct EvalBagExpr {
    pub elements: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalBagExpr {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let evaluated_elements: Vec<Value> = self
            .elements
            .iter()
            .map(|val| val.evaluate(bindings, ctx))
            .collect();

        Value::Bag(Box::new(Bag::from(evaluated_elements)))
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
                    Value::Tuple(mut tuple) => tuple.remove(s).unwrap_or(Missing),
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

#[derive(Debug, Default)]
pub struct EvalDistinct {
    pub input: Option<Value>,
    pub output: Option<Value>,
}

impl EvalDistinct {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Evaluable for EvalDistinct {
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Option<Value> {
        let out = self.input.clone().unwrap();
        let u: Vec<Value> = out.into_iter().unique().collect();
        self.output = Some(Value::Bag(Box::new(Bag::from(u))));
        self.output.clone()
    }

    fn update_input(&mut self, input: &Value, _branch_num: u8) {
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
    fn update_input(&mut self, input: &Value, _branch_num: u8) {
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
        let value = Bindings::get(bindings, &self.name).or_else(|| ctx.bindings().get(&self.name));
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
pub enum EvalUnaryOp {
    Pos,
    Neg,
    Not,
}

impl EvalExpr for EvalUnaryOpExpr {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let value = self.operand.evaluate(bindings, ctx);
        match self.op {
            EvalUnaryOp::Pos => value.positive(),
            EvalUnaryOp::Neg => -value,
            EvalUnaryOp::Not => !value,
        }
    }
}

// TODO we should replace this enum with some identifier that can be looked up in a symtab/funcregistry
#[derive(Debug)]
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
        #[inline]
        fn short_circuit(op: &EvalBinOp, value: &Value) -> Option<Value> {
            match (op, value) {
                (EvalBinOp::And, Boolean(false)) => Some(false.into()),
                (EvalBinOp::Or, Boolean(true)) => Some(true.into()),
                (_, Missing) => Some(Missing),
                _ => None,
            }
        }

        let lhs = self.lhs.evaluate(bindings, ctx);
        if let Some(propagate) = short_circuit(&self.op, &lhs) {
            return propagate;
        }

        let rhs = self.rhs.evaluate(bindings, ctx);
        match self.op {
            EvalBinOp::And => lhs.and(&rhs),
            EvalBinOp::Or => lhs.or(&rhs),
            EvalBinOp::Concat => {
                // TODO non-naive concat. Also doesn't properly propagate MISSING and NULL
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
            EvalBinOp::Eq => NullableEq::eq(&lhs, &rhs),
            EvalBinOp::Neq => lhs.neq(&rhs),
            EvalBinOp::Gt => NullableOrd::gt(&lhs, &rhs),
            EvalBinOp::Gteq => NullableOrd::gteq(&lhs, &rhs),
            EvalBinOp::Lt => NullableOrd::lt(&lhs, &rhs),
            EvalBinOp::Lteq => NullableOrd::lteq(&lhs, &rhs),
            EvalBinOp::Add => lhs + rhs,
            EvalBinOp::Sub => lhs - rhs,
            EvalBinOp::Mul => lhs * rhs,
            EvalBinOp::Div => lhs / rhs,
            EvalBinOp::Mod => lhs % rhs,
            EvalBinOp::Exp => todo!("Exponentiation"),
        }
    }
}

#[derive(Debug)]
pub struct EvalBetweenExpr {
    pub value: Box<dyn EvalExpr>,
    pub from: Box<dyn EvalExpr>,
    pub to: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalBetweenExpr {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let value = self.value.evaluate(bindings, ctx);
        let from = self.from.evaluate(bindings, ctx);
        let to = self.to.evaluate(bindings, ctx);
        value.gteq(&from).and(&value.lteq(&to))
    }
}

#[derive(Debug)]
pub struct EvalSearchedCaseExpr {
    pub cases: Vec<(Box<dyn EvalExpr>, Box<dyn EvalExpr>)>,
    pub default: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalSearchedCaseExpr {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        for (when_expr, then_expr) in &self.cases {
            let when_expr_evaluated = when_expr.evaluate(bindings, ctx);
            if when_expr_evaluated == Value::Boolean(true) {
                return then_expr.evaluate(bindings, ctx);
            }
        }
        self.default.evaluate(bindings, ctx)
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
                    while let Some((e, n)) = ne.next(&graph) {
                        // use the edge weight to store the `branch_num`
                        let branch_num = *graph
                            .edge_weight(e)
                            .expect("Error in retrieving weight for edge");
                        let dst = graph.node_weight_mut(n).expect("Error in retrieving node");
                        dst.update_input(
                            &result.clone().expect("Error in retrieving source value"),
                            branch_num,
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
}
