use crate::env::basic::MapBindings;
use crate::eval::expr::EvalExpr;
use crate::eval::{EvalContext, EvalPlan};
use itertools::Itertools;
use partiql_value::Value::{Boolean, Missing, Null};
use partiql_value::{partiql_bag, partiql_tuple, Bag, List, Tuple, Value};
use std::borrow::{Borrow, Cow};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

/// `Evaluable` represents each evaluation operator in the evaluation plan as an evaluable entity.
pub trait Evaluable: Debug {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value>;
    fn update_input(&mut self, input: Value, branch_num: u8);
    fn get_vars(&self) -> Option<&[String]> {
        None
    }
}

/// Represents an evaluation `Scan` operator; `Scan` operator scans the given bindings from its
/// input and and the environment and outputs a bag of binding tuples for tuples/values matching the
/// scan `expr`, e.g. an SQL expression `table1` in SQL expression `FROM table1`.
#[derive(Debug)]
pub struct EvalScan {
    pub expr: Box<dyn EvalExpr>,
    pub as_key: String,
    pub at_key: Option<String>,
    pub input: Option<Value>,

    // cached values
    attrs: Vec<String>,
}

impl EvalScan {
    pub fn new(expr: Box<dyn EvalExpr>, as_key: &str) -> Self {
        let attrs = vec![as_key.to_string()];
        EvalScan {
            expr,
            as_key: as_key.to_string(),
            at_key: None,
            input: None,
            attrs,
        }
    }
    pub fn new_with_at_key(expr: Box<dyn EvalExpr>, as_key: &str, at_key: &str) -> Self {
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().unwrap_or(Missing);

        let bindings = match input_value {
            Value::Bag(t) => *t,
            Value::Tuple(t) => partiql_bag![*t],
            _ => partiql_bag![partiql_tuple![]],
        };

        let mut value = partiql_bag![];
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

        Some(Value::Bag(Box::new(value)))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
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
pub struct EvalJoin {
    pub kind: EvalJoinKind,
    pub on: Option<Box<dyn EvalExpr>>,
    pub input: Option<Value>,
    pub left: Box<dyn Evaluable>,
    pub right: Box<dyn Evaluable>,
}

#[derive(Debug)]
pub enum EvalJoinKind {
    Inner,
    Left,
    Right,
    Full,
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
        #[inline]
        fn tuple_with_null_vals<I, S>(attrs: I) -> Tuple
        where
            S: Into<String>,
            I: IntoIterator<Item = S>,
        {
            attrs.into_iter().map(|k| (k.into(), Null)).collect()
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
                todo!("Full and Right Joins are not yet implemented for `partiql-lang-rust`")
            }
        };
        Some(Value::Bag(Box::new(output_bag)))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `GROUP BY` operator. For `GROUP BY` operational semantics, see section
/// `11` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug)]
pub struct EvalGroupBy {
    pub strategy: EvalGroupingStrategy,
    pub exprs: HashMap<String, Box<dyn EvalExpr>>,
    pub group_as_alias: Option<String>,
    pub input: Option<Value>,
}

/// Represents the grouping qualifier: ALL or PARTIAL.
#[derive(Debug)]
pub enum EvalGroupingStrategy {
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let group_as_alias = &self.group_as_alias;
        let input_value = self.input.take().expect("Error in retrieving input value");

        match self.strategy {
            EvalGroupingStrategy::GroupPartial => todo!(),
            EvalGroupingStrategy::GroupFull => {
                let mut groups: HashMap<Tuple, Vec<Value>> = HashMap::new();
                for v in input_value.into_iter() {
                    let v_as_tuple = v.coerce_to_tuple();
                    groups
                        .entry(self.eval_group(&v_as_tuple, ctx))
                        .or_insert(vec![])
                        .push(Value::Tuple(Box::new(v_as_tuple)));
                }

                let bag = groups
                    .into_iter()
                    .map(|(k, v)| match group_as_alias {
                        None => Value::from(k), // TODO: removing the values here will be insufficient for when aggregations are added since they may have nothing to aggregate over
                        Some(alias) => {
                            let mut tuple_with_group = k;
                            tuple_with_group.insert(alias, Value::Bag(Box::new(Bag::from(v))));
                            Value::from(tuple_with_group)
                        }
                    })
                    .collect::<Bag>();
                Some(Value::from(bag))
            }
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `Pivot` operator; the `Pivot` enables turning a collection into a
/// tuple. For `Pivot` operational semantics, see section `6.2` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug)]
pub struct EvalPivot {
    pub input: Option<Value>,
    pub key: Box<dyn EvalExpr>,
    pub value: Box<dyn EvalExpr>,
}

impl EvalPivot {
    pub fn new(key: Box<dyn EvalExpr>, value: Box<dyn EvalExpr>) -> Self {
        EvalPivot {
            input: None,
            key,
            value,
        }
    }
}

impl Evaluable for EvalPivot {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().expect("Error in retrieving input value");

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
        Some(Value::from(tuple))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `Unpivot` operator; the `Unpivot` enables ranging over the
/// attribute-value pairs of a tuple. For `Unpivot` operational semantics, see section `5.2` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug)]
pub struct EvalUnpivot {
    pub expr: Box<dyn EvalExpr>,
    pub as_key: String,
    pub at_key: Option<String>,
    pub input: Option<Value>,

    // cached values
    attrs: Vec<String>,
}

impl EvalUnpivot {
    pub fn new(expr: Box<dyn EvalExpr>, as_key: &str, at_key: Option<String>) -> Self {
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
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
        Some(Value::from(unpivoted))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().expect("Error in retrieving input value");

        let filtered = input_value
            .into_iter()
            .map(Value::coerce_to_tuple)
            .filter_map(|v| self.eval_filter(&v, ctx).then_some(v));
        Some(Value::from(filtered.collect::<Bag>()))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `Having` operator; for an input bag of binding tuples the `Having`
/// operator filters out the binding tuples that does not meet the condition expressed as `expr`,
/// e.g. `a = 10` in `HAVING a = 10` expression.
#[derive(Debug)]
pub struct EvalHaving {
    pub expr: Box<dyn EvalExpr>,
    pub input: Option<Value>,
}

impl EvalHaving {
    pub fn new(expr: Box<dyn EvalExpr>) -> Self {
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
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().expect("Error in retrieving input value");

        let filtered = input_value
            .into_iter()
            .map(Value::coerce_to_tuple)
            .filter_map(|v| self.eval_having(&v, ctx).then_some(v));
        Some(Value::from(filtered.collect::<Bag>()))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

#[derive(Debug)]
pub struct EvalOrderBySortCondition {
    pub expr: Box<dyn EvalExpr>,
    pub spec: EvalOrderBySortSpec,
}

#[derive(Debug)]
pub enum EvalOrderBySortSpec {
    AscNullsFirst,
    AscNullsLast,
    DescNullsFirst,
    DescNullsLast,
}

/// Represents an evaluation `Order By` operator; e.g. `ORDER BY a DESC NULLS LAST` in `SELECT a FROM t ORDER BY a DESC NULLS LAST`.
#[derive(Debug)]
pub struct EvalOrderBy {
    pub cmp: Vec<EvalOrderBySortCondition>,
    pub input: Option<Value>,
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
                    EvalOrderBySortSpec::AscNullsFirst => l.as_ref().cmp(r.as_ref()),
                    EvalOrderBySortSpec::AscNullsLast => match (l.as_ref(), r.as_ref()) {
                        (Null, Null) => Ordering::Equal,
                        (Null, Missing) => Ordering::Less,
                        (Missing, Missing) => Ordering::Equal,
                        (Missing, Null) => Ordering::Greater,
                        (Null, _) => Ordering::Greater,
                        (Missing, _) => Ordering::Greater,
                        (_, Null) => Ordering::Less,
                        (_, Missing) => Ordering::Less,
                        (l, r) => l.cmp(r),
                    },
                    EvalOrderBySortSpec::DescNullsFirst => match (l.as_ref(), r.as_ref()) {
                        (Null, Null) => Ordering::Equal,
                        (Null, Missing) => Ordering::Less,
                        (Missing, Missing) => Ordering::Equal,
                        (Missing, Null) => Ordering::Greater,
                        (Null, _) => Ordering::Less,
                        (Missing, _) => Ordering::Less,
                        (_, Null) => Ordering::Greater,
                        (_, Missing) => Ordering::Greater,
                        (l, r) => r.cmp(l),
                    },
                    EvalOrderBySortSpec::DescNullsLast => r.as_ref().cmp(l.as_ref()),
                }
            })
            .find_or_last(|o| o != &Ordering::Equal)
            .unwrap_or(Ordering::Equal)
    }
}

impl Evaluable for EvalOrderBy {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().expect("Error in retrieving input value");

        let mut values = input_value.into_iter().collect_vec();
        values.sort_by(|l, r| self.compare(l, r, ctx));
        Some(Value::from(List::from(values)))
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `LIMIT` and/or `OFFSET` operator.
#[derive(Debug)]
pub struct EvalLimitOffset {
    pub limit: Option<Box<dyn EvalExpr>>,
    pub offset: Option<Box<dyn EvalExpr>>,
    pub input: Option<Value>,
}

impl Evaluable for EvalLimitOffset {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().expect("Error in retrieving input value");

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
            Some(n) => Some(collect(offsetted.take(n), ordered)),
            None => Some(collect(offsetted, ordered)),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `SelectValue` operator; `SelectValue` implements PartiQL Core's
/// `SELECT VALUE` clause semantics. For `SelectValue` operational semantics, see section `6.1` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug)]
pub struct EvalSelectValue {
    pub expr: Box<dyn EvalExpr>,
    pub input: Option<Value>,
}

impl EvalSelectValue {
    pub fn new(expr: Box<dyn EvalExpr>) -> Self {
        EvalSelectValue { expr, input: None }
    }
}

impl Evaluable for EvalSelectValue {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().expect("Error in retrieving input value");

        let ordered = input_value.is_ordered();

        let values = input_value.into_iter().map(|v| {
            let v_as_tuple = v.coerce_to_tuple();
            self.expr.evaluate(&v_as_tuple, ctx).into_owned()
        });

        match ordered {
            true => Some(Value::from(values.collect::<List>())),
            false => Some(Value::from(values.collect::<Bag>())),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `Project` operator; for a given bag of input binding tuples as input
/// the `Project` selects attributes as specified by expressions in `exprs`. For `Project`
/// operational semantics, see section `6` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug)]
pub struct EvalSelect {
    pub exprs: HashMap<String, Box<dyn EvalExpr>>,
    pub input: Option<Value>,
}

impl EvalSelect {
    pub fn new(exprs: HashMap<String, Box<dyn EvalExpr>>) -> Self {
        EvalSelect { exprs, input: None }
    }
}

impl Evaluable for EvalSelect {
    fn evaluate(&mut self, ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().expect("Error in retrieving input value");

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
            true => Some(Value::from(values.collect::<List>())),
            false => Some(Value::from(values.collect::<Bag>())),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `ProjectAll` operator; `ProjectAll` implements SQL's `SELECT *`
/// semantics.
#[derive(Debug, Default)]
pub struct EvalSelectAll {
    pub input: Option<Value>,
}

impl EvalSelectAll {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Evaluable for EvalSelectAll {
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Option<Value> {
        let input_value = self.input.take().expect("Error in retrieving input value");

        let ordered = input_value.is_ordered();

        let values = input_value.into_iter().map(|val| {
            val.coerce_to_tuple()
                .into_values()
                .flat_map(|v| v.coerce_to_tuple().into_pairs())
                .collect::<Tuple>()
        });

        match ordered {
            true => Some(Value::from(values.collect::<List>())),
            false => Some(Value::from(values.collect::<Bag>())),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an evaluation `ExprQuery` operator; in PartiQL as opposed to SQL, the following
/// expression by its own is valid: `2 * 2`. Considering this, evaluation plan designates an operator
/// for evaluating such stand-alone expressions.
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
        let input_value = self.input.take().unwrap_or(Value::Null).coerce_to_tuple();

        Some(self.expr.evaluate(&input_value, ctx).into_owned())
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an SQL `DISTINCT` operator, e.g. in `SELECT DISTINCT a FROM t`.
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
        let input_value = self.input.take().expect("Error in retrieving input value");
        let ordered = input_value.is_ordered();

        let values = input_value.into_iter().unique();
        match ordered {
            true => Some(Value::from(values.collect::<List>())),
            false => Some(Value::from(values.collect::<Bag>())),
        }
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an operator that captures the output of a (sub)query in the plan.
#[derive(Debug)]
pub struct EvalSink {
    pub input: Option<Value>,
}

impl Evaluable for EvalSink {
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Option<Value> {
        self.input.take()
    }

    fn update_input(&mut self, input: Value, _branch_num: u8) {
        self.input = Some(input);
    }
}

/// Represents an evaluation operator for sub-queries, e.g. `SELECT a FROM b` in
/// `SELECT b.c, (SELECT a FROM b) FROM books AS b`.
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
