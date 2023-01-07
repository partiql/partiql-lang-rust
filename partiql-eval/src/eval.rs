use itertools::Itertools;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use thiserror::Error;

use petgraph::algo::toposort;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::StableGraph;
use petgraph::{Directed, Outgoing};

use partiql_value::Value::{Boolean, Missing, Null};
use partiql_value::{
    partiql_bag, partiql_tuple, Bag, BinaryAnd, BinaryOr, BindingsName, List, NullableEq,
    NullableOrd, Tuple, UnaryPlus, Value,
};

use crate::env::basic::MapBindings;
use crate::env::Bindings;
use partiql_logical::Type;

use petgraph::graph::NodeIndex;

use petgraph::visit::EdgeRef;
use regex::{Regex, RegexBuilder};
use std::borrow::Borrow;

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

    pub fn execute_mut(&mut self, bindings: MapBindings<Value>) -> Result<Evaluated, EvalErr> {
        let ctx: Box<dyn EvalContext> = Box::new(BasicContext { bindings });
        // We are only interested in DAGs that can be used as execution plans, which leads to the
        // following definition.
        // A DAG is a directed, cycle-free graph G = (V, E) with a denoted root node v0 ∈ V such
        // that all v ∈ V \{v0} are reachable from v0. Note that this is the definition of trees
        // without the condition |E| = |V | − 1. Hence, all trees are DAGs.
        // Reference: https://link.springer.com/article/10.1007/s00450-009-0061-0
        let sorted_ops = toposort(&self.0, None);
        match sorted_ops {
            Ok(ops) => {
                let plan_graph = &mut self.0;
                let mut result = None;
                for idx in ops.into_iter() {
                    let src = plan_graph
                        .node_weight_mut(idx)
                        .expect("Error in retrieving node");
                    result = src.evaluate(&*ctx);

                    let destinations: Vec<(usize, (u8, NodeIndex))> = plan_graph
                        .edges_directed(idx, Outgoing)
                        .map(|e| (*e.weight(), e.target()))
                        .enumerate()
                        .collect_vec();
                    let branches = destinations.len();
                    for (i, (branch_num, dst_id)) in destinations {
                        let res = if i == branches - 1 {
                            result.take()
                        } else {
                            result.clone()
                        }
                        .expect("Error in retrieving source value");

                        let dst = plan_graph
                            .node_weight_mut(dst_id)
                            .expect("Error in retrieving node");
                        dst.update_input(res, branch_num);
                    }
                }

                let result = result.expect("Error in retrieving eval output");
                Ok(Evaluated { result })
            }
            Err(e) => Err(EvalErr {
                errors: vec![EvaluationError::InvalidEvaluationPlan(format!(
                    "Malformed evaluation plan detected: {:?}",
                    e
                ))],
            }),
        }
    }

    pub fn to_dot_graph(&self) -> String {
        format!("{:?}", Dot::with_config(&self.0, &[Config::EdgeNoLabel]))
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
    fn update_input(&mut self, input: Value, branch_num: u8);
    fn get_vars(&self) -> Vec<String> {
        vec![]
    }
}

#[derive(Debug)]
pub struct EvalScan {
    pub expr: Box<dyn EvalExpr>,
    pub as_key: String,
    pub at_key: Option<String>,
    pub input: Option<Value>,
}

impl EvalScan {
    pub fn new(expr: Box<dyn EvalExpr>, as_key: &str) -> Self {
        EvalScan {
            expr,
            as_key: as_key.to_string(),
            at_key: None,
            input: None,
        }
    }
    pub fn new_with_at_key(expr: Box<dyn EvalExpr>, as_key: &str, at_key: &str) -> Self {
        EvalScan {
            expr,
            as_key: as_key.to_string(),
            at_key: Some(at_key.to_string()),
            input: None,
        }
    }
}

impl Evaluable for EvalScan {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().unwrap_or(Missing);

        let bindings = match input_value {
            Value::Bag(t) => *t,
            Value::Tuple(t) => partiql_bag![*t],
            _ => partiql_bag![partiql_tuple![]],
        };

        let mut value = partiql_bag![];
        bindings.iter().for_each(|binding| {
            let v = self.expr.evaluate(&binding.as_tuple_ref(), ctx);
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
        });

        Some(Value::Bag(Box::new(value)))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }

    fn get_vars(&self) -> Vec<String> {
        match self.at_key.clone() {
            None => vec![self.as_key.clone()],
            Some(at_key) => vec![self.as_key.clone(), at_key],
        }
    }
}

#[derive(Debug)]
pub enum EvalJoinKind {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug)]
pub struct EvalJoin {
    pub kind: EvalJoinKind,
    pub on: Option<Box<dyn EvalExpr>>,
    pub input: Option<Value>,
    pub left: Box<dyn Evaluable>,
    pub right: Box<dyn Evaluable>,
}

impl EvalJoin {
    pub fn new(
        kind: EvalJoinKind,
        left: Box<dyn Evaluable>,
        right: Box<dyn Evaluable>,
        on: Option<Box<dyn EvalExpr>>,
    ) -> Self {
        EvalJoin {
            kind,
            on,
            input: None,
            left,
            right,
        }
    }
}

impl Evaluable for EvalJoin {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        /// Creates a `Tuple` with attributes `attrs`, each with value `Null`
        fn tuple_with_null_vals(attrs: Vec<String>) -> Tuple {
            let mut new_tuple = Tuple::new();
            attrs.iter().for_each(|a| new_tuple.insert(a, Null));
            new_tuple
        }

        let mut output_bag = partiql_bag![];
        let input_env = self
            .input
            .take()
            .unwrap_or_else(|| Value::from(partiql_tuple![]));
        self.left.update_input(input_env.clone(), 0);
        let lhs_values = self.left.evaluate(ctx);
        let left_bindings = match lhs_values {
            Some(Value::Bag(t)) => *t,
            _ => panic!("Left side of FROM source should result in a bag of bindings"),
        };

        // Current implementations follow pseudocode defined in section 5.6 of spec
        // https://partiql.org/assets/PartiQL-Specification.pdf#subsection.5.6
        match self.kind {
            EvalJoinKind::Inner => {
                // for each binding b_l in eval(p0, p, l)
                left_bindings.iter().for_each(|b_l| {
                    let env_b_l = input_env
                        .as_tuple_ref()
                        .as_ref()
                        .tuple_concat(b_l.as_tuple_ref().borrow());
                    self.right.update_input(Value::from(env_b_l), 0);
                    let rhs_values = self.right.evaluate(ctx);

                    let right_bindings = match rhs_values {
                        Some(Value::Bag(t)) => *t,
                        _ => partiql_bag![partiql_tuple![]],
                    };

                    // for each binding b_r in eval (p0, (p || b_l), r)
                    for b_r in right_bindings.iter() {
                        match &self.on {
                            None => {
                                let b_l_b_r = b_l
                                    .as_tuple_ref()
                                    .as_ref()
                                    .tuple_concat(b_r.as_tuple_ref().borrow());
                                output_bag.push(Value::from(b_l_b_r));
                            }
                            // if eval(p0, (p || b_l || b_r), c) is true, add b_l || b_r to output bag
                            Some(condition) => {
                                let b_l_b_r = b_l
                                    .as_tuple_ref()
                                    .as_ref()
                                    .tuple_concat(b_r.as_tuple_ref().borrow());
                                let env_b_l_b_r =
                                    &input_env.as_tuple_ref().as_ref().tuple_concat(&b_l_b_r);
                                if condition.evaluate(env_b_l_b_r, ctx) == Value::Boolean(true) {
                                    output_bag.push(Value::Tuple(Box::new(b_l_b_r)));
                                }
                            }
                        }
                    }
                });
            }
            EvalJoinKind::Left => {
                // for each binding b_l in eval(p0, p, l)
                left_bindings.iter().for_each(|b_l| {
                    // define empty bag q_r
                    let mut output_bag_left = partiql_bag![];
                    let env_b_l = input_env
                        .as_tuple_ref()
                        .as_ref()
                        .tuple_concat(b_l.as_tuple_ref().borrow());
                    self.right.update_input(Value::from(env_b_l), 0);
                    let rhs_values = self.right.evaluate(ctx);

                    let right_bindings = match rhs_values {
                        Some(Value::Bag(t)) => *t,
                        _ => partiql_bag![partiql_tuple![]],
                    };

                    // for each binding b_r in eval (p0, (p || b_l), r)
                    for b_r in right_bindings.iter() {
                        match &self.on {
                            None => {
                                let b_l_b_r = b_l
                                    .as_tuple_ref()
                                    .as_ref()
                                    .tuple_concat(b_r.as_tuple_ref().borrow());
                                output_bag_left.push(Value::from(b_l_b_r));
                            }
                            // if eval(p0, (p || b_l || b_r), c) is true, add b_l || b_r to q_r
                            Some(condition) => {
                                let b_l_b_r = b_l
                                    .as_tuple_ref()
                                    .as_ref()
                                    .tuple_concat(b_r.as_tuple_ref().borrow());
                                let env_b_l_b_r =
                                    &input_env.as_tuple_ref().as_ref().tuple_concat(&b_l_b_r);
                                if condition.evaluate(env_b_l_b_r, ctx) == Value::Boolean(true) {
                                    output_bag_left.push(Value::Tuple(Box::new(b_l_b_r)));
                                }
                            }
                        }
                    }

                    // if q_r is the empty bag
                    if output_bag_left.is_empty() {
                        let attrs = self.right.get_vars();
                        let new_binding = b_l
                            .as_tuple_ref()
                            .as_ref()
                            .tuple_concat(&tuple_with_null_vals(attrs));
                        // add b_l || <v_1_r: NULL, ..., v_n_r: NULL> to output bag
                        output_bag.push(Value::from(new_binding));
                    } else {
                        // otherwise for each binding b_r in q_r, add b_l || b_r to output bag
                        for elem in output_bag_left.into_iter() {
                            output_bag.push(elem)
                        }
                    }
                });
            }
            EvalJoinKind::Full | EvalJoinKind::Right => {
                todo!("Full and Right Joins are not yet implemented for `partiql-lang-rust`")
            }
        };
        Some(Value::Bag(Box::new(output_bag)))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

#[derive(Debug)]
pub struct EvalUnpivot {
    pub expr: Box<dyn EvalExpr>,
    pub as_key: String,
    pub at_key: String,
    pub input: Option<Value>,
}

impl EvalUnpivot {
    pub fn new(expr: Box<dyn EvalExpr>, as_key: &str, at_key: &str) -> Self {
        EvalUnpivot {
            expr,
            as_key: as_key.to_string(),
            at_key: at_key.to_string(),
            input: None,
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

        Some(Value::Bag(Box::new(Bag::from(out))))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }

    fn get_vars(&self) -> Vec<String> {
        vec![self.as_key.clone(), self.at_key.clone()]
    }
}

#[derive(Debug)]
pub struct EvalFilter {
    pub expr: Box<dyn EvalExpr>,
    pub input: Option<Value>,
}

impl EvalFilter {
    pub fn new(expr: Box<dyn EvalExpr>) -> Self {
        EvalFilter { expr, input: None }
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
        let input_value = self.input.take().expect("Error in retrieving input value");

        let mut out = partiql_bag![];
        for v in input_value.into_iter() {
            if self.eval_filter(&v.clone().coerce_to_tuple(), ctx) {
                out.push(v);
            }
        }

        Some(Value::Bag(Box::new(out)))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

#[derive(Debug)]
pub struct EvalProject {
    pub exprs: HashMap<String, Box<dyn EvalExpr>>,
    pub input: Option<Value>,
}

impl EvalProject {
    pub fn new(exprs: HashMap<String, Box<dyn EvalExpr>>) -> Self {
        EvalProject { exprs, input: None }
    }
}

impl Evaluable for EvalProject {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().expect("Error in retrieving input value");

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

        match ordered {
            true => Some(Value::List(Box::new(List::from(value)))),
            false => Some(Value::Bag(Box::new(Bag::from(value)))),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

#[derive(Debug, Default)]
pub struct EvalProjectAll {
    pub input: Option<Value>,
}

impl EvalProjectAll {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Evaluable for EvalProjectAll {
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().expect("Error in retrieving input value");

        let ordered = &input_value.is_ordered();

        let seq = input_value
            .into_iter()
            .map(|val| {
                let mut t = Tuple::new();
                for (_k, val) in val.as_tuple_ref().pairs() {
                    t = t.tuple_concat(&val.as_tuple_ref());
                }
                Value::Tuple(Box::new(t))
            })
            .collect_vec();

        match ordered {
            true => Some(Value::List(Box::new(List::from(seq)))),
            false => Some(Value::Bag(Box::new(Bag::from(seq)))),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

#[derive(Debug)]
pub struct EvalProjectValue {
    pub expr: Box<dyn EvalExpr>,
    pub input: Option<Value>,
}

impl EvalProjectValue {
    pub fn new(expr: Box<dyn EvalExpr>) -> Self {
        EvalProjectValue { expr, input: None }
    }
}

impl Evaluable for EvalProjectValue {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().expect("Error in retrieving input value");

        let ordered = &input_value.is_ordered();
        let mut value = vec![];

        for v in input_value.into_iter() {
            let out = v.coerce_to_tuple();
            let evaluated = self.expr.evaluate(&out, ctx);
            value.push(evaluated);
        }

        match ordered {
            true => Some(Value::List(Box::new(List::from(value)))),
            false => Some(Value::Bag(Box::new(Bag::from(value)))),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

#[derive(Debug)]
pub struct EvalExprQuery {
    pub expr: Box<dyn EvalExpr>,
    pub input: Option<Value>,
}

impl EvalExprQuery {
    pub fn new(expr: Box<dyn EvalExpr>) -> Self {
        EvalExprQuery { expr, input: None }
    }
}

impl Evaluable for EvalExprQuery {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().unwrap_or(Value::Null);
        Some(self.expr.evaluate(&input_value.as_tuple_ref(), ctx))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
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
                if evaluated != Missing {
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
    Key(BindingsName),
    KeyExpr(Box<dyn EvalExpr>),
    Index(i64),
    IndexExpr(Box<dyn EvalExpr>),
}

#[derive(Debug)]
pub struct EvalPath {
    pub expr: Box<dyn EvalExpr>,
    pub components: Vec<EvalPathComponent>,
}

impl EvalExpr for EvalPath {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        #[inline]
        fn path_into(
            value: Value,
            path: &EvalPathComponent,
            bindings: &Tuple,
            ctx: &dyn EvalContext,
        ) -> Value {
            match path {
                EvalPathComponent::Key(k) => match value {
                    Value::Tuple(mut tuple) => tuple.remove(k).unwrap_or(Missing),
                    _ => Missing,
                },
                EvalPathComponent::Index(idx) => match value {
                    Value::List(mut list) if (*idx as usize) < list.len() => {
                        std::mem::take(list.get_mut(*idx).unwrap())
                    }
                    _ => Missing,
                },
                EvalPathComponent::KeyExpr(ke) => {
                    let key = ke.evaluate(bindings, ctx);
                    match (value, key) {
                        (Value::Tuple(mut tuple), Value::String(key)) => tuple
                            .remove(&BindingsName::CaseInsensitive(key.as_ref().clone()))
                            .unwrap_or(Value::Missing),
                        _ => Missing,
                    }
                }
                EvalPathComponent::IndexExpr(ie) => {
                    if let Value::Integer(idx) = ie.evaluate(bindings, ctx) {
                        match value {
                            Value::List(mut list) if (idx as usize) < list.len() => {
                                std::mem::take(list.get_mut(idx).unwrap())
                            }
                            _ => Missing,
                        }
                    } else {
                        Missing
                    }
                }
            }
        }

        let mut value = self.expr.evaluate(bindings, ctx);

        for path in &self.components {
            value = path_into(value, path, bindings, ctx);
        }
        value
    }
}

#[derive(Debug)]
pub struct EvalSubQueryExpr {
    pub plan: Rc<RefCell<EvalPlan>>,
}

impl EvalSubQueryExpr {
    pub fn new(plan: EvalPlan) -> Self {
        EvalSubQueryExpr {
            plan: Rc::new(RefCell::new(plan)),
        }
    }
}

impl EvalExpr for EvalSubQueryExpr {
    fn evaluate(&self, bindings: &Tuple, _ctx: &dyn EvalContext) -> Value {
        return if let Ok(evaluated) = self
            .plan
            .borrow_mut()
            .execute_mut(MapBindings::from(bindings))
        {
            evaluated.result
        } else {
            Missing
        };
    }
}

#[derive(Debug, Default)]
pub struct EvalDistinct {
    pub input: Option<Value>,
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
        Some(Value::Bag(Box::new(Bag::from(u))))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

#[derive(Debug)]
pub struct EvalSink {
    pub input: Option<Value>,
}

impl Evaluable for EvalSink {
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Option<Value> {
        self.input.clone()
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

pub trait EvalExpr: Debug {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value;
}

#[derive(Debug)]
pub struct EvalDynamicLookup {
    pub lookups: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalDynamicLookup {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let result = self
            .lookups
            .iter()
            .map(|lookup| lookup.evaluate(bindings, ctx))
            .find(|res| res != &Value::Missing);
        result.unwrap_or(Value::Missing)
    }
}

#[derive(Debug)]
pub struct EvalVarRef {
    pub name: BindingsName,
}

impl EvalExpr for EvalVarRef {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let value = Bindings::get(bindings, &self.name).or_else(|| ctx.bindings().get(&self.name));
        value.map_or(Missing, |v| v.clone())
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

#[derive(Debug)]
pub struct EvalIsTypeExpr {
    pub expr: Box<dyn EvalExpr>,
    pub is_type: Type,
}

impl EvalExpr for EvalIsTypeExpr {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let expr = self.expr.evaluate(bindings, ctx);
        let result = match self.is_type {
            Type::NullType => matches!(expr, Missing | Null),
            Type::MissingType => matches!(expr, Missing),
            _ => todo!("Implement `IS` for other types"),
        };
        Value::from(result)
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

    // Boolean ops
    In,
}

impl EvalExpr for EvalBinOpExpr {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        #[inline]
        fn short_circuit(op: &EvalBinOp, value: &Value) -> Option<Value> {
            match (op, value) {
                (EvalBinOp::And, Boolean(false)) => Some(false.into()),
                (EvalBinOp::Or, Boolean(true)) => Some(true.into()),
                (EvalBinOp::And, Missing) | (EvalBinOp::Or, Missing) | (EvalBinOp::In, Missing) => {
                    Some(Null)
                }
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
                // TODO non-naive concat (i.e., don't just use debug print for non-strings).
                match (&lhs, &rhs) {
                    (Missing, _) => Missing,
                    (_, Missing) => Missing,
                    (Null, _) => Null,
                    (_, Null) => Null,
                    _ => {
                        let lhs = if let Value::String(s) = lhs {
                            *s
                        } else {
                            format!("{:?}", lhs)
                        };
                        let rhs = if let Value::String(s) = rhs {
                            *s
                        } else {
                            format!("{:?}", rhs)
                        };
                        Value::String(Box::new(format!("{}{}", lhs, rhs)))
                    }
                }
            }
            EvalBinOp::Eq => NullableEq::eq(&lhs, &rhs),
            EvalBinOp::Neq => lhs.neq(&rhs),
            EvalBinOp::Gt => NullableOrd::gt(&lhs, &rhs),
            EvalBinOp::Gteq => NullableOrd::gteq(&lhs, &rhs),
            EvalBinOp::Lt => NullableOrd::lt(&lhs, &rhs),
            EvalBinOp::Lteq => NullableOrd::lteq(&lhs, &rhs),
            EvalBinOp::Add => &lhs + &rhs,
            EvalBinOp::Sub => &lhs - &rhs,
            EvalBinOp::Mul => &lhs * &rhs,
            EvalBinOp::Div => &lhs / &rhs,
            EvalBinOp::Mod => &lhs % &rhs,
            // TODO apply the changes once we clarify the rules of coercion for `IN` RHS.
            // See also:
            // - https://github.com/partiql/partiql-docs/pull/13
            // - https://github.com/partiql/partiql-lang-kotlin/issues/524
            // - https://github.com/partiql/partiql-lang-kotlin/pull/621#issuecomment-1147754213

            // TODO change the Null propagation if required.
            // Current implementation propagates `Null` as described in PartiQL spec section 8 [1]
            // and differs from `partiql-lang-kotlin` impl [2].
            // [1] https://github.com/partiql/partiql-lang-kotlin/issues/896
            // [2] https://partiql.org/assets/PartiQL-Specification.pdf#section.8
            EvalBinOp::In => match rhs.is_bag() || rhs.is_list() {
                true => {
                    let mut has_missing = false;
                    let mut has_null = false;
                    for elem in rhs.into_iter() {
                        // b/c of short_circuiting as we've reached this branch, we know LHS is neither MISSING nor NULL.
                        if elem == lhs {
                            return Boolean(true);
                        } else if elem == Missing {
                            has_missing = true;
                        } else if elem == Null {
                            has_null = true;
                        }
                    }

                    match has_missing | has_null {
                        true => Null,
                        false => Boolean(false),
                    }
                }
                _ => Null,
            },
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
pub struct EvalLikeMatch {
    pub value: Box<dyn EvalExpr>,
    pub pattern: Regex,
}

// TODO make configurable?
// Limit chosen somewhat arbitrarily, but to be smaller than the default of `10 * (1 << 20)`
const RE_SIZE_LIMIT: usize = 1 << 16;

impl EvalLikeMatch {
    pub fn new(value: Box<dyn EvalExpr>, pattern: &str) -> Self {
        let pattern = RegexBuilder::new(pattern)
            .size_limit(RE_SIZE_LIMIT)
            .build()
            .expect("Like Pattern");
        EvalLikeMatch { value, pattern }
    }
}

impl EvalExpr for EvalLikeMatch {
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let value = self.value.evaluate(bindings, ctx);
        match value {
            Null => Value::Null,
            Missing => Value::Missing,
            Value::String(s) => Value::Boolean(self.pattern.is_match(s.as_ref())),
            _ => Value::Boolean(false),
        }
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

pub trait EvalStrExpr: Debug {
    fn evaluate_str(&self, s: &String) -> Value;
    fn input(&self) -> &dyn EvalExpr;
}

impl<T> EvalExpr for T
where
    T: EvalStrExpr,
{
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let value = self.input().evaluate(bindings, ctx);
        match value {
            Null => Value::Null,
            Missing => Value::Missing,
            Value::String(s) => self.evaluate_str(s.as_ref()),
            _ => Value::Null,
        }
    }
}

#[derive(Debug)]
pub struct EvalFnLower {
    pub value: Box<dyn EvalExpr>,
}

impl EvalStrExpr for EvalFnLower {
    #[inline]
    fn evaluate_str(&self, s: &String) -> Value {
        s.to_lowercase().into()
    }

    #[inline]
    fn input(&self) -> &dyn EvalExpr {
        self.value.as_ref()
    }
}

#[derive(Debug)]
pub struct EvalFnUpper {
    pub value: Box<dyn EvalExpr>,
}

impl EvalStrExpr for EvalFnUpper {
    #[inline]
    fn evaluate_str(&self, s: &String) -> Value {
        s.to_uppercase().into()
    }

    #[inline]
    fn input(&self) -> &dyn EvalExpr {
        self.value.as_ref()
    }
}

#[derive(Debug)]
pub struct EvalFnCharLength {
    pub value: Box<dyn EvalExpr>,
}

impl EvalStrExpr for EvalFnCharLength {
    #[inline]
    fn evaluate_str(&self, s: &String) -> Value {
        s.chars().count().into()
    }

    #[inline]
    fn input(&self) -> &dyn EvalExpr {
        self.value.as_ref()
    }
}

#[derive(Debug)]
pub struct EvalFnSubstring {
    pub value: Box<dyn EvalExpr>,
    pub offset: Box<dyn EvalExpr>,
    pub length: Option<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalFnSubstring {
    #[inline]
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let value = match self.value.evaluate(bindings, ctx) {
            Missing => return Value::Missing,
            Value::String(s) => Some(s),
            _ => None,
        };
        let offset = match self.offset.evaluate(bindings, ctx) {
            Missing => return Value::Missing,
            Value::Integer(i) => Some(i),
            _ => None,
        };

        if let Some(length) = &self.length {
            let length = match length.evaluate(bindings, ctx) {
                Missing => return Value::Missing,
                Value::Integer(i) => i as usize,
                _ => return Value::Null,
            };
            if let (Some(value), Some(offset)) = (value, offset) {
                let (offset, length) = if length < 1 {
                    (0, 0)
                } else if offset < 1 {
                    let length = std::cmp::max(offset + (length - 1) as i64, 0) as usize;
                    let offset = std::cmp::max(offset, 0) as usize;
                    (offset, length)
                } else {
                    ((offset - 1) as usize, length)
                };
                return value
                    .chars()
                    .skip(offset)
                    .take(length)
                    .collect::<String>()
                    .into();
            }
        } else if let (Some(value), Some(offset)) = (value, offset) {
            let offset = (std::cmp::max(offset, 1) - 1) as usize;
            return value.chars().skip(offset).collect::<String>().into();
        };
        Value::Null
    }
}

#[derive(Debug)]
pub struct EvalFnBtrim {
    pub value: Box<dyn EvalExpr>,
    pub to_trim: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnBtrim {
    #[inline]
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let value = match self.value.evaluate(bindings, ctx) {
            Missing => return Value::Missing,
            Value::String(s) => Some(s),
            _ => None,
        };
        let to_trim = match self.to_trim.evaluate(bindings, ctx) {
            Missing => return Value::Missing,
            Value::String(s) => s,
            _ => return Value::Null,
        };
        if let Some(s) = value {
            let to_trim = to_trim.chars().collect_vec();
            s.trim_matches(&to_trim[..]).into()
        } else {
            Value::Null
        }
    }
}

#[derive(Debug)]
pub struct EvalFnRtrim {
    pub value: Box<dyn EvalExpr>,
    pub to_trim: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnRtrim {
    #[inline]
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let value = match self.value.evaluate(bindings, ctx) {
            Missing => return Value::Missing,
            Value::String(s) => Some(s),
            _ => None,
        };
        let to_trim = match self.to_trim.evaluate(bindings, ctx) {
            Missing => return Value::Missing,
            Value::String(s) => s,
            _ => return Value::Null,
        };
        if let Some(s) = value {
            let to_trim = to_trim.chars().collect_vec();
            s.trim_end_matches(&to_trim[..]).into()
        } else {
            Value::Null
        }
    }
}

#[derive(Debug)]
pub struct EvalFnLtrim {
    pub value: Box<dyn EvalExpr>,
    pub to_trim: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnLtrim {
    #[inline]
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let value = match self.value.evaluate(bindings, ctx) {
            Missing => return Value::Missing,
            Value::String(s) => Some(s),
            _ => None,
        };
        let to_trim = match self.to_trim.evaluate(bindings, ctx) {
            Missing => return Value::Missing,
            Value::String(s) => s,
            _ => return Value::Null,
        };
        if let Some(s) = value {
            let to_trim = to_trim.chars().collect_vec();
            s.trim_start_matches(&to_trim[..]).into()
        } else {
            Value::Null
        }
    }
}

#[derive(Debug)]
pub struct EvalFnNullif {
    pub test: Box<dyn EvalExpr>,
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnNullif {
    #[inline]
    fn evaluate(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Value {
        let test = self.test.evaluate(bindings, ctx);
        let value = self.value.evaluate(bindings, ctx);
        match (test, value) {
            (Null, Null) => Null,
            (Missing, Missing) => Null,
            (l, r) => match NullableEq::eq(&l, &r) {
                Value::Boolean(true) => Null,
                _ => l,
            },
        }
    }
}
