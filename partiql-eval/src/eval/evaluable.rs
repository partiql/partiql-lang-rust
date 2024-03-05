use crate::env::basic::MapBindings;
use crate::error::EvaluationError;
use crate::eval::expr::EvalExpr;
use crate::eval::{EvalContext, EvalPlan, NestedContext};
use itertools::Itertools;
use partiql_value::Value::{Boolean, Missing, Null};
use partiql_value::{
    bag, list, tuple, Bag, List, NullSortedValue, Tuple, Value, ValueIntoIterator,
};
use rustc_hash::FxHashMap;
use std::borrow::{Borrow, Cow};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

use std::rc::Rc;

#[macro_export]
macro_rules! take_input {
    ($expr:expr, $ctx:expr) => {
        match $expr {
            None => {
                $ctx.add_error(EvaluationError::IllegalState(
                    "Error in retrieving input value".to_string(),
                ));
                return Missing;
            }
            Some(val) => val,
        }
    };
}

/// Whether an [`Evaluable`] takes input from the plan graph or manages its own iteration.
pub enum EvalType {
    SelfManaged,
    GraphManaged,
}

/// `Evaluable` represents each evaluation operator in the evaluation plan as an evaluable entity.
pub trait Evaluable: Debug {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value;
    fn update_input(&mut self, input: Value, branch_num: u8, ctx: &dyn EvalContext);
    fn get_vars(&self) -> Option<&[String]> {
        None
    }
    fn eval_type(&self) -> EvalType {
        EvalType::GraphManaged
    }
}

/// Represents an evaluation `Scan` operator; `Scan` operator scans the given bindings from its
/// input and and the environment and outputs a bag of binding tuples for tuples/values matching the
/// scan `expr`, e.g. an SQL expression `table1` in SQL expression `FROM table1`.
pub(crate) struct EvalScan {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) as_key: String,
    pub(crate) at_key: Option<String>,
    pub(crate) input: Option<Value>,

    // cached values
    attrs: Vec<String>,
}

impl Debug for EvalScan {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SCAN ")?;
        self.expr.fmt(f)?;

        write!(f, " AS {}", self.as_key)?;

        if let Some(at_key) = &self.at_key {
            write!(f, " AT {}", at_key)?;
        }

        Ok(())
    }
}

impl EvalScan {
    pub(crate) fn new(expr: Box<dyn EvalExpr>, as_key: &str) -> Self {
        let attrs = vec![as_key.to_string()];
        EvalScan {
            expr,
            as_key: as_key.to_string(),
            at_key: None,
            input: None,
            attrs,
        }
    }
    pub(crate) fn new_with_at_key(expr: Box<dyn EvalExpr>, as_key: &str, at_key: &str) -> Self {
        let attrs = vec![as_key.to_string(), at_key.to_string()];
        EvalScan {
            expr,
            as_key: as_key.to_string(),
            at_key: Some(at_key.to_string()),
            input: None,
            attrs,
        }
    }
}

impl Evaluable for EvalScan {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let input_value = self.input.take().unwrap_or(Missing);

        let bindings = match input_value {
            Value::Bag(t) => *t,
            Value::Tuple(t) => bag![*t],
            _ => bag![tuple![]],
        };

        let mut value = bag![];
        bindings.iter().for_each(|binding| {
            let binding_tuple = binding.as_tuple_ref();
            let v = self.expr.evaluate(&binding_tuple, ctx).into_owned();
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

        Value::Bag(Box::new(value))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }

    fn get_vars(&self) -> Option<&[String]> {
        Some(&self.attrs)
    }
}

/// Represents an evaluation `Join` operator; `Join` joins the tuples from its LHS and RHS based on a logic defined
/// by [`EvalJoinKind`]. For semantics of PartiQL joins and their distinction with SQL's see sections
/// 5.3 – 5.7 of [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
pub(crate) struct EvalJoin {
    pub(crate) kind: EvalJoinKind,
    pub(crate) on: Option<Box<dyn EvalExpr>>,
    pub(crate) input: Option<Value>,
    pub(crate) left: Box<dyn Evaluable>,
    pub(crate) right: Box<dyn Evaluable>,
}

#[derive(Debug)]
pub(crate) enum EvalJoinKind {
    Inner,
    Left,
    Right,
    Full,
}

impl Debug for EvalJoin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?} JOIN", &self.kind)?;
        if let Some(on) = &self.on {
            write!(f, "ON ")?;
            on.fmt(f)?;
        }
        Ok(())
    }
}

impl EvalJoin {
    pub(crate) fn new(
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
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        /// Creates a `Tuple` with attributes `attrs`, each with value `Null`
        #[inline]
        fn tuple_with_null_vals<I, S>(attrs: I) -> Tuple
        where
            S: Into<String>,
            I: IntoIterator<Item = S>,
        {
            attrs.into_iter().map(|k| (k.into(), Null)).collect()
        }

        let mut output_bag = bag![];
        let input_env = self.input.take().unwrap_or_else(|| Value::from(tuple![]));
        self.left.update_input(input_env.clone(), 0, ctx);
        let lhs_values = self.left.evaluate(ctx);
        let left_bindings = match lhs_values {
            Value::Bag(t) => *t,
            _ => {
                ctx.add_error(EvaluationError::IllegalState(
                    "Left side of FROM source should result in a bag of bindings".to_string(),
                ));
                return Missing;
            }
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
                    self.right.update_input(Value::from(env_b_l), 0, ctx);
                    let rhs_values = self.right.evaluate(ctx);

                    let right_bindings = match rhs_values {
                        Value::Bag(t) => *t,
                        _ => bag![tuple![]],
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
                                let cond = condition.evaluate(env_b_l_b_r, ctx);
                                if cond.as_ref() == &Value::Boolean(true) {
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
                    let mut output_bag_left = bag![];
                    let env_b_l = input_env
                        .as_tuple_ref()
                        .as_ref()
                        .tuple_concat(b_l.as_tuple_ref().borrow());
                    self.right.update_input(Value::from(env_b_l), 0, ctx);
                    let rhs_values = self.right.evaluate(ctx);

                    let right_bindings = match rhs_values {
                        Value::Bag(t) => *t,
                        _ => bag![tuple![]],
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
                                let cond = condition.evaluate(env_b_l_b_r, ctx);
                                if cond.as_ref() == &Value::Boolean(true) {
                                    output_bag_left.push(Value::Tuple(Box::new(b_l_b_r)));
                                }
                            }
                        }
                    }

                    // if q_r is the empty bag
                    if output_bag_left.is_empty() {
                        let attrs = self.right.get_vars().unwrap_or(&[]);
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
                ctx.add_error(EvaluationError::NotYetImplemented(
                    "FULL and RIGHT JOIN".to_string(),
                ));
                return Missing;
            }
        };
        Value::Bag(Box::new(output_bag))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }

    fn eval_type(&self) -> EvalType {
        EvalType::SelfManaged
    }
}

/// An SQL aggregation function call that has been rewritten to be evaluated with the `GROUP BY`
/// clause. The `[name]` is the string (generated in AST lowering step) that replaces the
/// aggregation call expression. This name will be used as the field in the binding tuple output
/// by `GROUP BY`. `[expr]` corresponds to the expression within the aggregation function. And
/// `[func]` corresponds to the aggregation function that's being called (e.g. sum, count, avg).
///
/// For example, `SELECT a AS a, SUM(b) AS b FROM t GROUP BY a` is rewritten to the following form
///              `SELECT a AS a, $__agg_1 AS b FROM t GROUP BY a`
/// In the above example, `name` corresponds to '$__agg_1', `expr` refers to the expression within
/// the aggregation function, `b`, and `func` corresponds to the sum aggregation function,
/// `[AggSum]`.
#[derive(Debug)]
pub(crate) struct AggregateExpression {
    pub(crate) name: String,
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) func: Box<dyn AggregateFunction>,
}

impl AggregateFunction for AggregateExpression {
    #[inline]
    fn next_distinct(
        &self,
        input_value: &Value,
        state: &mut Option<Value>,
        seen: &mut FxHashMap<Value, ()>,
    ) {
        if input_value.is_present() {
            self.func.next_distinct(input_value, state, seen);
        }
    }

    #[inline]
    fn next_value(&self, input_value: &Value, state: &mut Option<Value>) {
        if input_value.is_present() {
            self.func.next_value(input_value, state);
        }
    }

    #[inline]
    fn finalize(&self, state: Option<Value>) -> Result<Value, EvaluationError> {
        self.func.finalize(state)
    }
}

/// Represents an SQL aggregation function computed on a collection of input values.
pub trait AggregateFunction: Debug {
    #[inline]
    fn next_distinct(
        &self,
        input_value: &Value,
        state: &mut Option<Value>,
        seen: &mut FxHashMap<Value, ()>,
    ) {
        match seen.entry(input_value.clone()) {
            Entry::Occupied(_) => {}
            Entry::Vacant(v) => {
                v.insert(());
                self.next_value(input_value, state);
            }
        }
    }
    /// Provides the next value for the given `group`.
    fn next_value(&self, input_value: &Value, state: &mut Option<Value>);
    /// Returns the result of the aggregation function for a given `group`.
    fn finalize(&self, state: Option<Value>) -> Result<Value, EvaluationError>;
}

/// Represents SQL's `AVG` aggregation function
#[derive(Debug)]
pub(crate) struct Avg {}

impl AggregateFunction for Avg {
    fn next_value(&self, input_value: &Value, state: &mut Option<Value>) {
        match state {
            None => *state = Some(Value::from(list![Value::from(1), input_value.clone()])),
            Some(Value::List(list)) => {
                if let Some(count) = list.get_mut(0) {
                    *count += &Value::from(1);
                }
                if let Some(sum) = list.get_mut(1) {
                    *sum += input_value;
                }
            }
            _ => unreachable!(),
        };
    }

    fn finalize(&self, state: Option<Value>) -> Result<Value, EvaluationError> {
        match state {
            None => Ok(Null),
            Some(Value::List(list)) => {
                let vals = list.to_vec();
                if let [count, sum] = &vals[..] {
                    if let Value::Integer(n) = sum {
                        // Avg does not do integer division; convert to decimal
                        let sum = Value::from(rust_decimal::Decimal::from(*n));
                        Ok(&sum / count)
                    } else {
                        Ok(sum / count)
                    }
                } else {
                    Err(EvaluationError::IllegalState(
                        "Bad finalize state for Avg".to_string(),
                    ))
                }
            }
            _ => unreachable!(),
        }
    }
}

/// Represents SQL's `COUNT` aggregation function
#[derive(Debug)]
pub(crate) struct Count {}

impl AggregateFunction for Count {
    fn next_value(&self, _: &Value, state: &mut Option<Value>) {
        match state {
            None => *state = Some(Value::from(1)),
            Some(Value::Integer(i)) => {
                *i += 1;
            }
            _ => unreachable!(),
        };
    }

    fn finalize(&self, state: Option<Value>) -> Result<Value, EvaluationError> {
        Ok(state.unwrap_or_else(|| Value::from(0)))
    }
}

/// Represents SQL's `MAX` aggregation function
#[derive(Debug)]
pub(crate) struct Max {}

impl AggregateFunction for Max {
    fn next_value(&self, input_value: &Value, state: &mut Option<Value>) {
        match state {
            None => *state = Some(input_value.clone()),
            Some(max) => {
                if &*max < input_value {
                    *max = input_value.clone();
                }
            }
        };
    }

    fn finalize(&self, state: Option<Value>) -> Result<Value, EvaluationError> {
        Ok(state.unwrap_or_else(|| Null))
    }
}

/// Represents SQL's `MIN` aggregation function
#[derive(Debug)]
pub(crate) struct Min {}

impl AggregateFunction for Min {
    fn next_value(&self, input_value: &Value, state: &mut Option<Value>) {
        match state {
            None => *state = Some(input_value.clone()),
            Some(min) => {
                if &*min > input_value {
                    *min = input_value.clone();
                }
            }
        };
    }

    fn finalize(&self, state: Option<Value>) -> Result<Value, EvaluationError> {
        Ok(state.unwrap_or_else(|| Null))
    }
}

/// Represents SQL's `SUM` aggregation function
#[derive(Debug)]
pub(crate) struct Sum {}

impl AggregateFunction for Sum {
    fn next_value(&self, input_value: &Value, state: &mut Option<Value>) {
        match state {
            None => *state = Some(input_value.clone()),
            Some(ref mut sum) => *sum += input_value,
        };
    }

    fn finalize(&self, state: Option<Value>) -> Result<Value, EvaluationError> {
        Ok(state.unwrap_or_else(|| Null))
    }
}

/// Represents SQL's `ANY`/`SOME` aggregation function
#[derive(Debug)]
pub(crate) struct Any {}

impl AggregateFunction for Any {
    fn next_value(&self, input_value: &Value, state: &mut Option<Value>) {
        match state {
            None => {
                *state = Some(match input_value {
                    Boolean(b) => Value::Boolean(*b),
                    _ => Missing,
                })
            }
            Some(ref mut acc) => {
                *acc = match (&acc, input_value) {
                    (Boolean(acc), Boolean(new)) => Boolean(*acc || *new),
                    _ => Missing,
                }
            }
        };
    }

    fn finalize(&self, state: Option<Value>) -> Result<Value, EvaluationError> {
        Ok(state.unwrap_or_else(|| Null))
    }
}

/// Represents SQL's `EVERY` aggregation function
#[derive(Debug)]
pub(crate) struct Every {}

impl AggregateFunction for Every {
    fn next_value(&self, input_value: &Value, state: &mut Option<Value>) {
        match state {
            None => {
                *state = Some(match input_value {
                    Boolean(b) => Value::Boolean(*b),
                    _ => Missing,
                })
            }
            Some(ref mut acc) => {
                *acc = match (&acc, input_value) {
                    (Boolean(acc), Boolean(new)) => Boolean(*acc && *new),
                    _ => Missing,
                }
            }
        };
    }

    fn finalize(&self, state: Option<Value>) -> Result<Value, EvaluationError> {
        Ok(state.unwrap_or_else(|| Null))
    }
}

/// Represents an evaluation `GROUP BY` operator. For `GROUP BY` operational semantics, see section
/// `11` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
/// `aggregate_exprs` represents the set of aggregate expressions to compute.
#[derive(Debug)]
pub(crate) struct EvalGroupBy {
    pub(crate) strategy: EvalGroupingStrategy,
    pub(crate) group: Vec<Box<dyn EvalExpr>>,
    pub(crate) aliases: Vec<String>,
    pub(crate) aggs: Vec<AggregateExpression>,
    pub(crate) distinct_aggs: Vec<AggregateExpression>,
    pub(crate) group_as_alias: Option<String>,
    pub(crate) input: Option<Value>,
}

type GroupKey = Vec<Value>;
type AggState = Vec<Option<Value>>;
type DAggState = Vec<(Option<Value>, FxHashMap<Value, ()>)>;
#[derive(Clone)]
struct CombinedState(AggState, DAggState, Option<Vec<Value>>);

/// Represents the grouping qualifier: ALL or PARTIAL.
#[derive(Debug)]
pub(crate) enum EvalGroupingStrategy {
    GroupFull,
    GroupPartial,
}

impl EvalGroupBy {
    #[inline]
    pub(crate) fn new(
        strategy: EvalGroupingStrategy,
        group: Vec<Box<dyn EvalExpr>>,
        aliases: Vec<String>,
        aggs: Vec<AggregateExpression>,
        distinct_aggs: Vec<AggregateExpression>,
        group_as_alias: Option<String>,
    ) -> Self {
        Self {
            strategy,
            group,
            aliases,
            aggs,
            distinct_aggs,
            group_as_alias,
            input: None,
        }
    }

    #[inline]
    fn group_key<'a, 'c>(&'a self, bindings: &'a Tuple, ctx: &'c dyn EvalContext<'c>) -> GroupKey {
        self.group
            .iter()
            .map(|expr| match expr.evaluate(bindings, ctx).as_ref() {
                Missing => Value::Null,
                val => val.clone(),
            })
            .collect()
    }
}

impl Evaluable for EvalGroupBy {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let group_as_alias = &self.group_as_alias;
        let input_value = take_input!(self.input.take(), ctx);

        match self.strategy {
            EvalGroupingStrategy::GroupPartial => {
                ctx.add_error(EvaluationError::NotYetImplemented(
                    "GROUP PARTIAL".to_string(),
                ));
                Missing
            }
            EvalGroupingStrategy::GroupFull => {
                let mut grouped: FxHashMap<GroupKey, CombinedState> = FxHashMap::default();
                let state = std::iter::repeat(None).take(self.aggs.len()).collect_vec();
                let distinct_state = std::iter::repeat_with(|| (None, FxHashMap::default()))
                    .take(self.distinct_aggs.len())
                    .collect_vec();
                let group_as = group_as_alias.as_ref().map(|_| vec![]);

                let combined = CombinedState(state, distinct_state, group_as);

                for v in input_value.into_iter() {
                    let v_as_tuple = v.coerce_into_tuple();
                    let group_key = self.group_key(&v_as_tuple, ctx);
                    let CombinedState(state, distinct_state, group_as) =
                        grouped.entry(group_key).or_insert_with(|| combined.clone());

                    // Compute next aggregation result for each of the aggregation expressions
                    for (agg_expr, state) in self.aggs.iter().zip(state.iter_mut()) {
                        let evaluated = agg_expr.expr.evaluate(&v_as_tuple, ctx);
                        agg_expr.next_value(evaluated.as_ref(), state);
                    }

                    // Compute next aggregation result for each of the distinct aggregation expressions
                    for (distinct_expr, (state, seen)) in
                        self.distinct_aggs.iter().zip(distinct_state.iter_mut())
                    {
                        let evaluated = distinct_expr.expr.evaluate(&v_as_tuple, ctx);
                        distinct_expr.next_distinct(evaluated.as_ref(), state, seen);
                    }

                    // Add tuple to `GROUP AS` if applicable
                    if let Some(ref mut tuples) = group_as {
                        tuples.push(Value::from(v_as_tuple));
                    }
                }

                let vals = grouped
                    .into_iter()
                    .map(|(group_key, state)| {
                        let CombinedState(agg_state, distinct_state, group_as) = state;
                        let group = self.aliases.iter().cloned().zip(group_key);

                        // finalize all aggregates
                        let aggs_with_state = self.aggs.iter().zip(agg_state);
                        let daggs_with_state = self
                            .distinct_aggs
                            .iter()
                            .zip(distinct_state.into_iter().map(|(state, _)| state));
                        let agg_data = aggs_with_state.chain(daggs_with_state).map(
                            |(aggregate_expr, state)| {
                                let val = match aggregate_expr.finalize(state) {
                                    Ok(agg_result) => agg_result,
                                    Err(err) => {
                                        ctx.add_error(err);
                                        Missing
                                    }
                                };

                                (aggregate_expr.name.to_string(), val)
                            },
                        );

                        let mut tuple = Tuple::from_iter(group.chain(agg_data));

                        // insert `GROUP AS` if applicable
                        if let Some(tuples) = group_as {
                            tuple.insert(
                                group_as_alias.as_ref().unwrap(),
                                Value::from(Bag::from(tuples)),
                            );
                        }

                        Value::from(tuple)
                    })
                    .collect_vec();

                Value::from(Bag::from(vals))
            }
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `Pivot` operator; the `Pivot` enables turning a collection into a
/// tuple. For `Pivot` operational semantics, see section `6.2` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug)]
pub(crate) struct EvalPivot {
    pub(crate) input: Option<Value>,
    pub(crate) key: Box<dyn EvalExpr>,
    pub(crate) value: Box<dyn EvalExpr>,
}

impl EvalPivot {
    pub(crate) fn new(key: Box<dyn EvalExpr>, value: Box<dyn EvalExpr>) -> Self {
        EvalPivot {
            input: None,
            key,
            value,
        }
    }
}

impl Evaluable for EvalPivot {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let tuple: Tuple = input_value
            .into_iter()
            .filter_map(|binding| {
                let binding = binding.coerce_into_tuple();
                let key = self.key.evaluate(&binding, ctx);
                if let Value::String(s) = key.as_ref() {
                    let value = self.value.evaluate(&binding, ctx);
                    Some((s.to_string(), value.into_owned()))
                } else {
                    None
                }
            })
            .collect();
        Value::from(tuple)
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `Unpivot` operator; the `Unpivot` enables ranging over the
/// attribute-value pairs of a tuple. For `Unpivot` operational semantics, see section `5.2` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug)]
pub(crate) struct EvalUnpivot {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) as_key: String,
    pub(crate) at_key: Option<String>,
    pub(crate) input: Option<Value>,

    // cached values
    attrs: Vec<String>,
}

impl EvalUnpivot {
    pub(crate) fn new(expr: Box<dyn EvalExpr>, as_key: &str, at_key: Option<String>) -> Self {
        let attrs = if let Some(at_key) = &at_key {
            vec![as_key.to_string(), at_key.clone()]
        } else {
            vec![as_key.to_string()]
        };

        EvalUnpivot {
            expr,
            as_key: as_key.to_string(),
            at_key,
            input: None,
            attrs,
        }
    }
}

impl Evaluable for EvalUnpivot {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let tuple = match self.expr.evaluate(&Tuple::new(), ctx).into_owned() {
            Value::Tuple(tuple) => *tuple,
            other => other.coerce_into_tuple(),
        };

        let as_key = self.as_key.as_str();
        let pairs = tuple;
        let unpivoted = if let Some(at_key) = &self.at_key {
            pairs
                .map(|(k, v)| Tuple::from([(as_key, v), (at_key.as_str(), k.into())]))
                .collect::<Bag>()
        } else {
            pairs
                .map(|(_, v)| Tuple::from([(as_key, v)]))
                .collect::<Bag>()
        };
        Value::from(unpivoted)
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }

    fn get_vars(&self) -> Option<&[String]> {
        Some(&self.attrs)
    }
}

/// Represents an evaluation `Filter` operator; for an input bag of binding tuples the `Filter`
/// operator filters out the binding tuples that does not meet the condition expressed as `expr`,
/// e.g.`a > 2` in `WHERE a > 2` expression.
#[derive(Debug)]
pub(crate) struct EvalFilter {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) input: Option<Value>,
}

impl EvalFilter {
    pub(crate) fn new(expr: Box<dyn EvalExpr>) -> Self {
        EvalFilter { expr, input: None }
    }

    #[inline]
    fn eval_filter<'a, 'c>(&'a self, bindings: &'a Tuple, ctx: &'c dyn EvalContext<'c>) -> bool {
        let result = self.expr.evaluate(bindings, ctx);
        match result.as_ref() {
            Boolean(bool_val) => *bool_val,
            // Alike SQL, when the expression of the WHERE clause expression evaluates to
            // absent value or a value that is not a Boolean, PartiQL eliminates the corresponding
            // binding. PartiQL Specification August 1, 2019 Draft, Section 8. `WHERE clause`
            _ => false,
        }
    }
}

impl Evaluable for EvalFilter {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let filtered = input_value
            .into_iter()
            .map(Value::coerce_into_tuple)
            .filter_map(|v| self.eval_filter(&v, ctx).then_some(v));
        Value::from(filtered.collect::<Bag>())
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `Having` operator; for an input bag of binding tuples the `Having`
/// operator filters out the binding tuples that does not meet the condition expressed as `expr`,
/// e.g. `a = 10` in `HAVING a = 10` expression.
#[derive(Debug)]
pub(crate) struct EvalHaving {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) input: Option<Value>,
}

impl EvalHaving {
    pub(crate) fn new(expr: Box<dyn EvalExpr>) -> Self {
        EvalHaving { expr, input: None }
    }

    #[inline]
    fn eval_having<'a, 'c>(&'a self, bindings: &'a Tuple, ctx: &'c dyn EvalContext<'c>) -> bool {
        let result = self.expr.evaluate(bindings, ctx);
        match result.as_ref() {
            Boolean(bool_val) => *bool_val,
            // Alike SQL, when the expression of the HAVING clause expression evaluates to
            // absent value or a value that is not a Boolean, PartiQL eliminates the corresponding
            // binding. PartiQL Specification August 1, 2019 Draft, Section 11.1.
            // > HAVING behaves identical to a WHERE, once groups are already formulated earlier
            // See Section 8 on WHERE semantics
            _ => false,
        }
    }
}

impl Evaluable for EvalHaving {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let filtered = input_value
            .into_iter()
            .map(Value::coerce_into_tuple)
            .filter_map(|v| self.eval_having(&v, ctx).then_some(v));
        Value::from(filtered.collect::<Bag>())
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

#[derive(Debug)]
pub(crate) struct EvalOrderBySortCondition {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) spec: EvalOrderBySortSpec,
}

#[derive(Debug)]
pub(crate) enum EvalOrderBySortSpec {
    AscNullsFirst,
    AscNullsLast,
    DescNullsFirst,
    DescNullsLast,
}

/// Represents an evaluation `Order By` operator; e.g. `ORDER BY a DESC NULLS LAST` in `SELECT a FROM t ORDER BY a DESC NULLS LAST`.
#[derive(Debug)]
pub(crate) struct EvalOrderBy {
    pub(crate) cmp: Vec<EvalOrderBySortCondition>,
    pub(crate) input: Option<Value>,
}

impl EvalOrderBy {
    #[inline]
    fn compare<'c>(&self, l: &Value, r: &Value, ctx: &'c dyn EvalContext<'c>) -> Ordering {
        let l = l.as_tuple_ref();
        let r = r.as_tuple_ref();
        self.cmp
            .iter()
            .map(|spec| {
                let l = spec.expr.evaluate(&l, ctx);
                let r = spec.expr.evaluate(&r, ctx);

                match spec.spec {
                    EvalOrderBySortSpec::AscNullsFirst => {
                        let wrap = NullSortedValue::<true, Value>;
                        let (l, r) = (wrap(l.as_ref()), wrap(r.as_ref()));
                        l.cmp(&r)
                    }
                    EvalOrderBySortSpec::AscNullsLast => {
                        let wrap = NullSortedValue::<false, Value>;
                        let (l, r) = (wrap(l.as_ref()), wrap(r.as_ref()));
                        l.cmp(&r)
                    }
                    EvalOrderBySortSpec::DescNullsFirst => {
                        let wrap = NullSortedValue::<false, Value>;
                        let (l, r) = (wrap(l.as_ref()), wrap(r.as_ref()));
                        r.cmp(&l)
                    }
                    EvalOrderBySortSpec::DescNullsLast => {
                        let wrap = NullSortedValue::<true, Value>;
                        let (l, r) = (wrap(l.as_ref()), wrap(r.as_ref()));
                        r.cmp(&l)
                    }
                }
            })
            .find_or_last(|o| o != &Ordering::Equal)
            .unwrap_or(Ordering::Equal)
    }
}

impl Evaluable for EvalOrderBy {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let mut values = input_value.into_iter().collect_vec();
        values.sort_by(|l, r| self.compare(l, r, ctx));
        Value::from(List::from(values))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `LIMIT` and/or `OFFSET` operator.
#[derive(Debug)]
pub(crate) struct EvalLimitOffset {
    pub(crate) limit: Option<Box<dyn EvalExpr>>,
    pub(crate) offset: Option<Box<dyn EvalExpr>>,
    pub(crate) input: Option<Value>,
}

impl Evaluable for EvalLimitOffset {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let empty_bindings = Tuple::new();

        let offset = match &self.offset {
            None => 0,
            Some(expr) => match expr.evaluate(&empty_bindings, ctx).as_ref() {
                Value::Integer(i) => {
                    if *i >= 0 {
                        *i as usize
                    } else {
                        0
                    }
                }
                _ => 0,
            },
        };

        let limit = match &self.limit {
            None => None,
            Some(expr) => match expr.evaluate(&empty_bindings, ctx).as_ref() {
                Value::Integer(i) => {
                    if *i >= 0 {
                        Some(*i as usize)
                    } else {
                        None
                    }
                }
                _ => None,
            },
        };

        let ordered = input_value.is_ordered();
        fn collect(values: impl Iterator<Item = Value>, ordered: bool) -> Value {
            match ordered {
                true => Value::from(values.collect::<List>()),
                false => Value::from(values.collect::<Bag>()),
            }
        }

        let offsetted = input_value.into_iter().skip(offset);
        match limit {
            Some(n) => collect(offsetted.take(n), ordered),
            None => collect(offsetted, ordered),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `SelectValue` operator; `SelectValue` implements PartiQL Core's
/// `SELECT VALUE` clause semantics. For `SelectValue` operational semantics, see section `6.1` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug)]
pub(crate) struct EvalSelectValue {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) input: Option<Value>,
}

impl EvalSelectValue {
    pub(crate) fn new(expr: Box<dyn EvalExpr>) -> Self {
        EvalSelectValue { expr, input: None }
    }
}

impl Evaluable for EvalSelectValue {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let ordered = input_value.is_ordered();

        let values = input_value.into_iter().map(|v| {
            let v_as_tuple = v.coerce_into_tuple();
            self.expr.evaluate(&v_as_tuple, ctx).into_owned()
        });

        match ordered {
            true => Value::from(values.collect::<List>()),
            false => Value::from(values.collect::<Bag>()),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `Project` operator; for a given bag of input binding tuples as input
/// the `Project` selects attributes as specified by expressions in `exprs`. For `Project`
/// operational semantics, see section `6` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
pub(crate) struct EvalSelect {
    pub(crate) exprs: Vec<(String, Box<dyn EvalExpr>)>,
    pub(crate) input: Option<Value>,
}

impl EvalSelect {
    pub(crate) fn new(exprs: Vec<(String, Box<dyn EvalExpr>)>) -> Self {
        EvalSelect { exprs, input: None }
    }
}

impl Debug for EvalSelect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SELECT ")?;
        let mut sep = "";
        for (alias, expr) in &self.exprs {
            write!(f, "{sep}")?;
            expr.fmt(f)?;
            write!(f, " AS {alias}")?;
            sep = ", ";
        }

        Ok(())
    }
}

impl Evaluable for EvalSelect {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let ordered = input_value.is_ordered();

        let values = input_value.into_iter().map(|v| {
            let v_as_tuple = v.coerce_into_tuple();

            let tuple_pairs = self.exprs.iter().filter_map(|(alias, expr)| {
                let evaluated_val = expr.evaluate(&v_as_tuple, ctx);
                match evaluated_val.as_ref() {
                    Missing => None,
                    _ => Some((alias.as_str(), evaluated_val.into_owned())),
                }
            });

            tuple_pairs.collect::<Tuple>()
        });

        match ordered {
            true => Value::from(values.collect::<List>()),
            false => Value::from(values.collect::<Bag>()),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `ProjectAll` operator; `ProjectAll` implements SQL's `SELECT *`
/// semantics.
#[derive(Debug, Default)]
pub(crate) struct EvalSelectAll {
    pub(crate) input: Option<Value>,
}

impl EvalSelectAll {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl Evaluable for EvalSelectAll {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let ordered = input_value.is_ordered();

        let values = input_value.into_iter().map(|val| {
            val.coerce_into_tuple()
                .into_values()
                .flat_map(|v| v.coerce_into_tuple().into_pairs())
                .collect::<Tuple>()
        });

        match ordered {
            true => Value::from(values.collect::<List>()),
            false => Value::from(values.collect::<Bag>()),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `ExprQuery` operator; in PartiQL as opposed to SQL, the following
/// expression by its own is valid: `2 * 2`. Considering this, evaluation plan designates an operator
/// for evaluating such stand-alone expressions.
#[derive(Debug)]
pub(crate) struct EvalExprQuery {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) input: Option<Value>,
}

impl EvalExprQuery {
    pub(crate) fn new(expr: Box<dyn EvalExpr>) -> Self {
        EvalExprQuery { expr, input: None }
    }
}

impl Evaluable for EvalExprQuery {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let input_value = self.input.take().unwrap_or(Value::Null).coerce_into_tuple();

        self.expr.evaluate(&input_value, ctx).into_owned()
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

/// Represents an SQL `DISTINCT` operator, e.g. in `SELECT DISTINCT a FROM t`.
#[derive(Debug, Default)]
pub(crate) struct EvalDistinct {
    pub(crate) input: Option<Value>,
}

impl EvalDistinct {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl Evaluable for EvalDistinct {
    fn evaluate<'a, 'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Value {
        let input_value = take_input!(self.input.take(), ctx);
        let ordered = input_value.is_ordered();

        let values = input_value.into_iter().unique();
        match ordered {
            true => Value::from(values.collect::<List>()),
            false => Value::from(values.collect::<Bag>()),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

/// Represents an operator that captures the output of a (sub)query in the plan.
pub(crate) struct EvalSink {
    pub(crate) input: Option<Value>,
}

impl Evaluable for EvalSink {
    fn evaluate<'a, 'c>(&mut self, _ctx: &'c dyn EvalContext<'c>) -> Value {
        self.input.take().unwrap_or_else(|| Missing)
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
    }
}

impl Debug for EvalSink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SINK")
    }
}

/// Represents an evaluation operator for sub-queries, e.g. `SELECT a FROM b` in
/// `SELECT b.c, (SELECT a FROM b) FROM books AS b`.
#[derive(Debug)]
pub(crate) struct EvalSubQueryExpr {
    pub(crate) plan: Rc<RefCell<EvalPlan>>,
}

impl EvalSubQueryExpr {
    pub(crate) fn new(plan: EvalPlan) -> Self {
        EvalSubQueryExpr {
            plan: Rc::new(RefCell::new(plan)),
        }
    }
}

impl EvalExpr for EvalSubQueryExpr {
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a,
    {
        let bindings = MapBindings::from(bindings);
        let value = {
            let nested_ctx: NestedContext = NestedContext::new(bindings, ctx);

            let mut plan = self.plan.borrow_mut();
            if let Ok(evaluated) = plan.execute_mut(&nested_ctx) {
                evaluated.result
            } else {
                Missing
            }
        };
        Cow::Owned(value)
    }
}

///
/// Coercion function F for bag operators described in RFC-0007
/// - F(absent_value) -> << >>
/// - F(scalar_value) -> << scalar_value >> # singleton bag
/// - F(tuple_value)  -> << tuple_value >>  # singleton bag, see future extensions
/// - F(array_value)  -> bag_value          # discard ordering
/// - F(bag_value)    -> bag_value          # identity
///
#[inline]
fn bagop_iter(v: Value) -> ValueIntoIterator {
    match v {
        Value::Null | Value::Missing => ValueIntoIterator::Single(None),
        other => other.into_iter(),
    }
}

/// Represents the `OUTER UNION` bag operator.
#[derive(Debug, PartialEq)]
pub(crate) struct EvalOuterUnion {
    pub(crate) setq: SetQuantifier,
    pub(crate) l_input: Option<Value>,
    pub(crate) r_input: Option<Value>,
}

impl EvalOuterUnion {
    pub(crate) fn new(setq: SetQuantifier) -> Self {
        EvalOuterUnion {
            setq,
            l_input: None,
            r_input: None,
        }
    }
}

impl Evaluable for EvalOuterUnion {
    fn evaluate<'a, 'c>(&mut self, _ctx: &'c dyn EvalContext<'c>) -> Value {
        let lhs = bagop_iter(self.l_input.take().unwrap_or(Missing));
        let rhs = bagop_iter(self.r_input.take().unwrap_or(Missing));
        let chained = lhs.chain(rhs);
        let vals = match self.setq {
            SetQuantifier::All => chained.collect_vec(),
            SetQuantifier::Distinct => chained.unique().collect_vec(),
        };
        Value::from(Bag::from(vals))
    }

    fn update_input(&mut self, input: Value, branch_num: u8, ctx: &dyn EvalContext) {
        match branch_num {
            0 => self.l_input = Some(input),
            1 => self.r_input = Some(input),
            _ => ctx.add_error(EvaluationError::IllegalState(
                "Invalid branch number".to_string(),
            )),
        }
    }
}

/// Represents the `OUTER INTERSECT` bag operator.
#[derive(Debug, PartialEq)]
pub(crate) struct EvalOuterIntersect {
    pub(crate) setq: SetQuantifier,
    pub(crate) l_input: Option<Value>,
    pub(crate) r_input: Option<Value>,
}

impl EvalOuterIntersect {
    pub(crate) fn new(setq: SetQuantifier) -> Self {
        EvalOuterIntersect {
            setq,
            l_input: None,
            r_input: None,
        }
    }
}

impl Evaluable for EvalOuterIntersect {
    fn evaluate<'a, 'c>(&mut self, _ctx: &'c dyn EvalContext<'c>) -> Value {
        let lhs = bagop_iter(self.l_input.take().unwrap_or(Missing));
        let rhs = bagop_iter(self.r_input.take().unwrap_or(Missing));

        let bag: Bag = match self.setq {
            SetQuantifier::All => {
                let mut lhs = lhs.counts();
                Bag::from_iter(rhs.filter(|elem| match lhs.get_mut(elem) {
                    Some(count) if *count > 0 => {
                        *count -= 1;
                        true
                    }
                    _ => false,
                }))
            }
            SetQuantifier::Distinct => {
                let lhs: HashSet<Value> = lhs.collect();
                Bag::from_iter(
                    rhs.filter(|elem| lhs.contains(elem))
                        .collect::<HashSet<_>>(),
                )
            }
        };
        Value::from(bag)
    }

    fn update_input(&mut self, input: Value, branch_num: u8, ctx: &dyn EvalContext) {
        match branch_num {
            0 => self.l_input = Some(input),
            1 => self.r_input = Some(input),
            _ => ctx.add_error(EvaluationError::IllegalState(
                "Invalid branch number".to_string(),
            )),
        }
    }
}

/// Represents the `OUTER EXCEPT` bag operator.
#[derive(Debug, PartialEq)]
pub(crate) struct EvalOuterExcept {
    pub(crate) setq: SetQuantifier,
    pub(crate) l_input: Option<Value>,
    pub(crate) r_input: Option<Value>,
}

impl EvalOuterExcept {
    pub(crate) fn new(setq: SetQuantifier) -> Self {
        EvalOuterExcept {
            setq,
            l_input: None,
            r_input: None,
        }
    }
}

impl Evaluable for EvalOuterExcept {
    fn evaluate<'a, 'c>(&mut self, _ctx: &'c dyn EvalContext<'c>) -> Value {
        let lhs = bagop_iter(self.l_input.take().unwrap_or(Missing));
        let rhs = bagop_iter(self.r_input.take().unwrap_or(Missing));

        let mut exclude = rhs.counts();
        let excepted = lhs.filter(|elem| match exclude.get_mut(elem) {
            Some(count) if *count > 0 => {
                *count -= 1;
                false
            }
            _ => true,
        });
        let vals = match self.setq {
            SetQuantifier::All => excepted.collect_vec(),
            SetQuantifier::Distinct => excepted.unique().collect_vec(),
        };
        Value::from(Bag::from(vals))
    }

    fn update_input(&mut self, input: Value, branch_num: u8, ctx: &dyn EvalContext) {
        match branch_num {
            0 => self.l_input = Some(input),
            1 => self.r_input = Some(input),
            _ => ctx.add_error(EvaluationError::IllegalState(
                "Invalid branch number".to_string(),
            )),
        }
    }
}

/// Indicates if a set should be reduced to its distinct elements or not.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum SetQuantifier {
    All,
    Distinct,
}
