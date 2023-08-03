use crate::env::basic::MapBindings;
use crate::error::EvaluationError;
use crate::eval::expr::EvalExpr;
use crate::eval::{EvalContext, EvalPlan};
use itertools::Itertools;
use partiql_value::Value::{Boolean, Missing, Null};
use partiql_value::{bag, tuple, Bag, List, NullSortedValue, Tuple, Value, ValueIntoIterator};
use std::borrow::{Borrow, Cow};
use std::cell::RefCell;
use std::cmp::{max, min, Ordering};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value;
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
#[derive(Debug)]
pub(crate) struct EvalScan {
    pub(crate) expr: Box<dyn EvalExpr>,
    pub(crate) as_key: String,
    pub(crate) at_key: Option<String>,
    pub(crate) input: Option<Value>,

    // cached values
    attrs: Vec<String>,
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
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
#[derive(Debug)]
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
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
    pub(crate) func: AggFunc,
}

/// Represents an SQL aggregation function computed on a collection of input values.
pub trait AggregateFunction {
    /// Provides the next value for the given `group`.
    fn next_value(&mut self, input_value: &Value, group: &Tuple);
    /// Returns the result of the aggregation function for a given `group`.
    fn compute(&self, group: &Tuple) -> Result<Value, EvaluationError>;
}

#[derive(Debug)]
pub(crate) enum AggFunc {
    // TODO: modeling COUNT(*)
    Avg(Avg),
    Count(Count),
    Max(Max),
    Min(Min),
    Sum(Sum),
}

impl AggregateFunction for AggFunc {
    fn next_value(&mut self, input_value: &Value, group: &Tuple) {
        match self {
            AggFunc::Avg(v) => v.next_value(input_value, group),
            AggFunc::Count(v) => v.next_value(input_value, group),
            AggFunc::Max(v) => v.next_value(input_value, group),
            AggFunc::Min(v) => v.next_value(input_value, group),
            AggFunc::Sum(v) => v.next_value(input_value, group),
        }
    }

    fn compute(&self, group: &Tuple) -> Result<Value, EvaluationError> {
        match self {
            AggFunc::Avg(v) => v.compute(group),
            AggFunc::Count(v) => v.compute(group),
            AggFunc::Max(v) => v.compute(group),
            AggFunc::Min(v) => v.compute(group),
            AggFunc::Sum(v) => v.compute(group),
        }
    }
}

/// Filter values based on the given condition
#[derive(Debug, Default)]
pub(crate) enum AggFilterFn {
    /// Keeps only distinct values in each group
    Distinct(AggFilterDistinct),
    /// Keeps all values
    #[default]
    All,
}

impl AggFilterFn {
    /// Returns true if and only if for the given `group`, `input_value` should be processed
    /// by the aggregation function
    fn filter_value(&mut self, input_value: Value, group: &Tuple) -> bool {
        match self {
            AggFilterFn::Distinct(d) => d.filter_value(input_value, group),
            AggFilterFn::All => true,
        }
    }
}

#[derive(Debug)]
pub(crate) struct AggFilterDistinct {
    seen_vals: HashMap<Tuple, HashSet<Value>>,
}

impl AggFilterDistinct {
    pub(crate) fn new() -> Self {
        AggFilterDistinct {
            seen_vals: HashMap::new(),
        }
    }
}

impl Default for AggFilterDistinct {
    fn default() -> Self {
        Self::new()
    }
}

impl AggFilterDistinct {
    fn filter_value(&mut self, input_value: Value, group: &Tuple) -> bool {
        if let Some(seen_vals_in_group) = self.seen_vals.get_mut(group) {
            seen_vals_in_group.insert(input_value)
        } else {
            let mut new_seen_vals = HashSet::new();
            new_seen_vals.insert(input_value);
            self.seen_vals
                .insert(group.clone(), new_seen_vals)
                .is_none()
        }
    }
}

/// Represents SQL's `AVG` aggregation function
#[derive(Debug)]
pub(crate) struct Avg {
    avgs: HashMap<Tuple, (usize, Value)>,
    aggregator: AggFilterFn,
}

impl Avg {
    pub(crate) fn new_distinct() -> Self {
        Avg {
            avgs: HashMap::new(),
            aggregator: AggFilterFn::Distinct(AggFilterDistinct::new()),
        }
    }

    pub(crate) fn new_all() -> Self {
        Avg {
            avgs: HashMap::new(),
            aggregator: AggFilterFn::default(),
        }
    }
}

impl AggregateFunction for Avg {
    fn next_value(&mut self, input_value: &Value, group: &Tuple) {
        if !input_value.is_null_or_missing()
            && self.aggregator.filter_value(input_value.clone(), group)
        {
            match self.avgs.get_mut(group) {
                None => {
                    self.avgs.insert(group.clone(), (1, input_value.clone()));
                }
                Some((count, sum)) => {
                    *count += 1;
                    *sum = &sum.clone() + input_value;
                }
            }
        }
    }

    fn compute(&self, group: &Tuple) -> Result<Value, EvaluationError> {
        match self.avgs.get(group) {
            None => Err(EvaluationError::IllegalState(
                "Expect group to exist in avgs".to_string(),
            )),
            Some((0, _)) => Ok(Null),
            Some((c, s)) => Ok(s / &Value::from(rust_decimal::Decimal::from(*c))),
        }
    }
}

/// Represents SQL's `COUNT` aggregation function
#[derive(Debug)]
pub(crate) struct Count {
    counts: HashMap<Tuple, usize>,
    aggregator: AggFilterFn,
}

impl Count {
    pub(crate) fn new_distinct() -> Self {
        Count {
            counts: HashMap::new(),
            aggregator: AggFilterFn::Distinct(AggFilterDistinct::new()),
        }
    }

    pub(crate) fn new_all() -> Self {
        Count {
            counts: HashMap::new(),
            aggregator: AggFilterFn::default(),
        }
    }
}

impl AggregateFunction for Count {
    fn next_value(&mut self, input_value: &Value, group: &Tuple) {
        if !input_value.is_null_or_missing()
            && self.aggregator.filter_value(input_value.clone(), group)
        {
            match self.counts.get_mut(group) {
                None => {
                    self.counts.insert(group.clone(), 1);
                }
                Some(count) => {
                    *count += 1;
                }
            };
        }
    }

    fn compute(&self, group: &Tuple) -> Result<Value, EvaluationError> {
        match self.counts.get(group) {
            None => Err(EvaluationError::IllegalState(
                "Expect group to exist in counts".to_string(),
            )),
            Some(val) => Ok(Value::from(val)),
        }
    }
}

/// Represents SQL's `MAX` aggregation function
#[derive(Debug)]
pub(crate) struct Max {
    maxes: HashMap<Tuple, Value>,
    aggregator: AggFilterFn,
}

impl Max {
    pub(crate) fn new_distinct() -> Self {
        Max {
            maxes: HashMap::new(),
            aggregator: AggFilterFn::Distinct(AggFilterDistinct::new()),
        }
    }

    pub(crate) fn new_all() -> Self {
        Max {
            maxes: HashMap::new(),
            aggregator: AggFilterFn::default(),
        }
    }
}

impl AggregateFunction for Max {
    fn next_value(&mut self, input_value: &Value, group: &Tuple) {
        if !input_value.is_null_or_missing()
            && self.aggregator.filter_value(input_value.clone(), group)
        {
            match self.maxes.get_mut(group) {
                None => {
                    self.maxes.insert(group.clone(), input_value.clone());
                }
                Some(m) => {
                    *m = max(m.clone(), input_value.clone());
                }
            }
        }
    }

    fn compute(&self, group: &Tuple) -> Result<Value, EvaluationError> {
        match self.maxes.get(group) {
            None => Err(EvaluationError::IllegalState(
                "Expect group to exist in maxes".to_string(),
            )),
            Some(val) => Ok(val.clone()),
        }
    }
}

/// Represents SQL's `MIN` aggregation function
#[derive(Debug)]
pub(crate) struct Min {
    mins: HashMap<Tuple, Value>,
    aggregator: AggFilterFn,
}

impl Min {
    pub(crate) fn new_distinct() -> Self {
        Min {
            mins: HashMap::new(),
            aggregator: AggFilterFn::Distinct(AggFilterDistinct::new()),
        }
    }

    pub(crate) fn new_all() -> Self {
        Min {
            mins: HashMap::new(),
            aggregator: AggFilterFn::default(),
        }
    }
}

impl AggregateFunction for Min {
    fn next_value(&mut self, input_value: &Value, group: &Tuple) {
        if !input_value.is_null_or_missing()
            && self.aggregator.filter_value(input_value.clone(), group)
        {
            match self.mins.get_mut(group) {
                None => {
                    self.mins.insert(group.clone(), input_value.clone());
                }
                Some(m) => {
                    *m = min(m.clone(), input_value.clone());
                }
            }
        }
    }

    fn compute(&self, group: &Tuple) -> Result<Value, EvaluationError> {
        match self.mins.get(group) {
            None => Err(EvaluationError::IllegalState(
                "Expect group to exist in mins".to_string(),
            )),
            Some(val) => Ok(val.clone()),
        }
    }
}

/// Represents SQL's `SUM` aggregation function
#[derive(Debug)]
pub(crate) struct Sum {
    sums: HashMap<Tuple, Value>,
    aggregator: AggFilterFn,
}

impl Sum {
    pub(crate) fn new_distinct() -> Self {
        Sum {
            sums: HashMap::new(),
            aggregator: AggFilterFn::Distinct(AggFilterDistinct::new()),
        }
    }

    pub(crate) fn new_all() -> Self {
        Sum {
            sums: HashMap::new(),
            aggregator: AggFilterFn::default(),
        }
    }
}

impl AggregateFunction for Sum {
    fn next_value(&mut self, input_value: &Value, group: &Tuple) {
        if !input_value.is_null_or_missing()
            && self.aggregator.filter_value(input_value.clone(), group)
        {
            match self.sums.get_mut(group) {
                None => {
                    self.sums.insert(group.clone(), input_value.clone());
                }
                Some(s) => {
                    *s = &s.clone() + input_value;
                }
            }
        }
    }

    fn compute(&self, group: &Tuple) -> Result<Value, EvaluationError> {
        match self.sums.get(group) {
            None => Err(EvaluationError::IllegalState(
                "Expect group to exist in sums".to_string(),
            )),
            Some(val) => Ok(val.clone()),
        }
    }
}

/// Represents an evaluation `GROUP BY` operator. For `GROUP BY` operational semantics, see section
/// `11` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
/// `aggregate_exprs` represents the set of aggregate expressions to compute.
#[derive(Debug)]
pub(crate) struct EvalGroupBy {
    pub(crate) strategy: EvalGroupingStrategy,
    pub(crate) exprs: HashMap<String, Box<dyn EvalExpr>>,
    pub(crate) aggregate_exprs: Vec<AggregateExpression>,
    pub(crate) group_as_alias: Option<String>,
    pub(crate) input: Option<Value>,
}

/// Represents the grouping qualifier: ALL or PARTIAL.
#[derive(Debug)]
pub(crate) enum EvalGroupingStrategy {
    GroupFull,
    GroupPartial,
}

impl EvalGroupBy {
    #[inline]
    fn eval_group(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> Tuple {
        self.exprs
            .iter()
            .map(
                |(alias, expr)| match expr.evaluate(bindings, ctx).into_owned() {
                    Missing => (alias.as_str(), Value::Null),
                    val => (alias.as_str(), val),
                },
            )
            .collect::<Tuple>()
    }
}

impl Evaluable for EvalGroupBy {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
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
                let mut groups: HashMap<Tuple, Vec<Value>> = HashMap::new();
                for v in input_value.into_iter() {
                    let v_as_tuple = v.coerce_to_tuple();
                    let group = self.eval_group(&v_as_tuple, ctx);
                    // Compute next aggregation result for each of the aggregation expressions
                    for aggregate_expr in self.aggregate_exprs.iter_mut() {
                        let evaluated_val =
                            aggregate_expr.expr.evaluate(&v_as_tuple, ctx).into_owned();
                        aggregate_expr.func.next_value(&evaluated_val, &group);
                    }
                    groups
                        .entry(group)
                        .or_insert(vec![])
                        .push(Value::Tuple(Box::new(v_as_tuple.clone())));
                }

                let bag = groups
                    .into_iter()
                    .map(|(mut k, v)| {
                        // Finalize aggregation computation and include result in output binding
                        // tuple
                        let mut agg_results: Vec<(&str, Value)> = vec![];
                        for aggregate_expr in &self.aggregate_exprs {
                            match aggregate_expr.func.compute(&k) {
                                Ok(agg_result) => {
                                    agg_results.push((aggregate_expr.name.as_str(), agg_result))
                                }
                                Err(err) => {
                                    ctx.add_error(err);
                                    return Missing;
                                }
                            }
                        }
                        agg_results
                            .into_iter()
                            .for_each(|(agg_name, agg_result)| k.insert(agg_name, agg_result));

                        match group_as_alias {
                            None => Value::from(k),
                            Some(alias) => {
                                let mut tuple_with_group = k;
                                tuple_with_group.insert(alias, Value::Bag(Box::new(Bag::from(v))));
                                Value::from(tuple_with_group)
                            }
                        }
                    })
                    .collect::<Bag>();
                Value::from(bag)
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let tuple: Tuple = input_value
            .into_iter()
            .filter_map(|binding| {
                let binding = binding.coerce_to_tuple();
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
        let tuple = match self.expr.evaluate(&Tuple::new(), ctx).into_owned() {
            Value::Tuple(tuple) => *tuple,
            other => other.coerce_to_tuple(),
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
    fn eval_filter(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> bool {
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let filtered = input_value
            .into_iter()
            .map(Value::coerce_to_tuple)
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
    fn eval_having(&self, bindings: &Tuple, ctx: &dyn EvalContext) -> bool {
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let filtered = input_value
            .into_iter()
            .map(Value::coerce_to_tuple)
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
    fn compare(&self, l: &Value, r: &Value, ctx: &dyn EvalContext) -> Ordering {
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let ordered = input_value.is_ordered();

        let values = input_value.into_iter().map(|v| {
            let v_as_tuple = v.coerce_to_tuple();
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
#[derive(Debug)]
pub(crate) struct EvalSelect {
    pub(crate) exprs: Vec<(String, Box<dyn EvalExpr>)>,
    pub(crate) input: Option<Value>,
}

impl EvalSelect {
    pub(crate) fn new(exprs: Vec<(String, Box<dyn EvalExpr>)>) -> Self {
        EvalSelect { exprs, input: None }
    }
}

impl Evaluable for EvalSelect {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let ordered = input_value.is_ordered();

        let values = input_value.into_iter().map(|v| {
            let v_as_tuple = v.coerce_to_tuple();

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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
        let input_value = take_input!(self.input.take(), ctx);

        let ordered = input_value.is_ordered();

        let values = input_value.into_iter().map(|val| {
            val.coerce_to_tuple()
                .into_values()
                .flat_map(|v| v.coerce_to_tuple().into_pairs())
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
        let input_value = self.input.take().unwrap_or(Value::Null).coerce_to_tuple();

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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Value {
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
#[derive(Debug)]
pub(crate) struct EvalSink {
    pub(crate) input: Option<Value>,
}

impl Evaluable for EvalSink {
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Value {
        self.input.take().unwrap_or_else(|| Missing)
    }

    fn update_input(&mut self, input: Value, _branch_num: u8, _ctx: &dyn EvalContext) {
        self.input = Some(input);
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
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, _ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = if let Ok(evaluated) = self
            .plan
            .borrow_mut()
            .execute_mut(MapBindings::from(bindings))
        {
            evaluated.result
        } else {
            Missing
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
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Value {
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
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Value {
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
                        .collect::<HashSet<_>>()
                        .into_iter(),
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
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Value {
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
