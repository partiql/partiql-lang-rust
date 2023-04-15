use crate::env::Bindings;
use crate::eval::expr::pattern_match::like_to_re_pattern;
use crate::eval::EvalContext;
use itertools::Itertools;
use partiql_logical::Type;
use partiql_value::Value::{Boolean, Missing, Null};
use partiql_value::{
    Bag, BinaryAnd, BinaryOr, BindingsName, DateTime, List, NullableEq, NullableOrd, Tuple,
    UnaryPlus, Value,
};
use regex::{Regex, RegexBuilder};
use rust_decimal::prelude::FromPrimitive;
use std::borrow::{Borrow, Cow};
use std::fmt::Debug;

pub(crate) mod pattern_match;

/// A trait for expressions that require evaluation, e.g. `a + b` or `c > 2`.
pub trait EvalExpr: Debug {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value>;
}

/// Represents an evaluation operator for Tuple expressions such as `{t1.a: t1.b * 2}` in
/// `SELECT VALUE {t1.a: t1.b * 2} FROM table1 AS t1`.
#[derive(Debug)]
pub struct EvalTupleExpr {
    pub attrs: Vec<Box<dyn EvalExpr>>,
    pub vals: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalTupleExpr {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let tuple = self
            .attrs
            .iter()
            .zip(self.vals.iter())
            .filter_map(|(attr, val)| {
                let key = attr.evaluate(bindings, ctx);
                match key.as_ref() {
                    Value::String(key) => {
                        let val = val.evaluate(bindings, ctx);
                        match val.as_ref() {
                            Missing => None,
                            _ => Some((key.to_string(), val.into_owned())),
                        }
                    }
                    _ => None,
                }
            })
            .collect::<Tuple>();

        Cow::Owned(Value::from(tuple))
    }
}

/// Represents an evaluation operator for List (ordered array) expressions such as
/// `[t1.a, t1.b * 2]` in `SELECT VALUE [t1.a, t1.b * 2] FROM table1 AS t1`.
#[derive(Debug)]
pub struct EvalListExpr {
    pub elements: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalListExpr {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let values = self
            .elements
            .iter()
            .map(|val| val.evaluate(bindings, ctx).into_owned());

        Cow::Owned(Value::from(values.collect::<List>()))
    }
}

/// Represents an evaluation operator for Bag (unordered array) expressions such as
/// `<<t1.a, t1.b * 2>>` in `SELECT VALUE <<t1.a, t1.b * 2>> FROM table1 AS t1`.
#[derive(Debug)]
pub struct EvalBagExpr {
    pub elements: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalBagExpr {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let values = self
            .elements
            .iter()
            .map(|val| val.evaluate(bindings, ctx).into_owned());

        Cow::Owned(Value::from(values.collect::<Bag>()))
    }
}

/// Represents an evaluation operator for path navigation expressions as outlined in Section `4` of
/// [PartiQL Specification — August 1, 2019](https://partiql.org/assets/PartiQL-Specification.pdf).
#[derive(Debug)]
pub struct EvalPath {
    pub expr: Box<dyn EvalExpr>,
    pub components: Vec<EvalPathComponent>,
}

#[derive(Debug)]
pub enum EvalPathComponent {
    Key(BindingsName),
    KeyExpr(Box<dyn EvalExpr>),
    Index(i64),
    IndexExpr(Box<dyn EvalExpr>),
}

impl EvalExpr for EvalPath {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        #[inline]
        fn path_into<'a>(
            value: &'a Value,
            path: &EvalPathComponent,
            bindings: &'a Tuple,
            ctx: &dyn EvalContext,
        ) -> Option<&'a Value> {
            match path {
                EvalPathComponent::Key(k) => match value {
                    Value::Tuple(tuple) => tuple.get(k),
                    _ => None,
                },
                EvalPathComponent::Index(idx) => match value {
                    Value::List(list) if (*idx as usize) < list.len() => list.get(*idx),
                    _ => None,
                },
                EvalPathComponent::KeyExpr(ke) => {
                    let key = ke.evaluate(bindings, ctx);
                    match (value, key.as_ref()) {
                        (Value::Tuple(tuple), Value::String(key)) => {
                            tuple.get(&BindingsName::CaseInsensitive(key.as_ref().clone()))
                        }
                        _ => None,
                    }
                }
                EvalPathComponent::IndexExpr(ie) => {
                    if let Value::Integer(idx) = ie.evaluate(bindings, ctx).as_ref() {
                        match value {
                            Value::List(list) if (*idx as usize) < list.len() => list.get(*idx),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
            }
        }
        let value = self.expr.evaluate(bindings, ctx);
        self.components
            .iter()
            .fold(Some(value.as_ref()), |v, path| {
                v.and_then(|v| path_into(v, path, bindings, ctx))
            })
            .map_or_else(|| Cow::Owned(Value::Missing), |v| Cow::Owned(v.clone()))
    }
}

/// Represents an operator for dynamic variable name resolution of a (sub)query.
#[derive(Debug)]
pub struct EvalDynamicLookup {
    pub lookups: Vec<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalDynamicLookup {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let mut lookups = self.lookups.iter().filter_map(|lookup| {
            let val = lookup.evaluate(bindings, ctx);
            match val.as_ref() {
                Missing => None,
                _ => Some(val),
            }
        });

        lookups.next().unwrap_or_else(|| Cow::Owned(Value::Missing))
    }
}

/// Represents a variable reference in a (sub)query, e.g. `a` in `SELECT b as a FROM`.
#[derive(Debug)]
pub struct EvalVarRef {
    pub name: BindingsName,
}

impl EvalExpr for EvalVarRef {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = Bindings::get(bindings, &self.name).or_else(|| ctx.bindings().get(&self.name));

        match value {
            None => Cow::Owned(Missing),
            Some(v) => Cow::Borrowed(v),
        }
    }
}

/// Represents a literal in (sub)query, e.g. `1` in `a + 1`.
#[derive(Debug)]
pub struct EvalLitExpr {
    pub lit: Box<Value>,
}

impl EvalExpr for EvalLitExpr {
    fn evaluate<'a>(&'a self, _bindings: &'a Tuple, _ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        Cow::Borrowed(self.lit.as_ref())
    }
}

/// Represents an evaluation unary operator, e.g. `NOT` in `NOT TRUE`.
#[derive(Debug)]
pub struct EvalUnaryOpExpr {
    pub op: EvalUnaryOp,
    pub operand: Box<dyn EvalExpr>,
}

// TODO we should replace this enum with some identifier that can be looked up in a symtab/funcregistry
#[derive(Debug)]
pub enum EvalUnaryOp {
    Pos,
    Neg,
    Not,
}

impl EvalExpr for EvalUnaryOpExpr {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let operand = self.operand.evaluate(bindings, ctx);
        let result = match self.op {
            EvalUnaryOp::Pos => operand.into_owned().positive(),
            EvalUnaryOp::Neg => -operand.as_ref(),
            EvalUnaryOp::Not => !operand.as_ref(),
        };
        Cow::Owned(result)
    }
}

/// Represents a PartiQL evaluation `IS` operator, e.g. `a IS INT`.
#[derive(Debug)]
pub struct EvalIsTypeExpr {
    pub expr: Box<dyn EvalExpr>,
    pub is_type: Type,
}

impl EvalExpr for EvalIsTypeExpr {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let expr = self.expr.evaluate(bindings, ctx);
        let expr = expr.as_ref();
        let result = match self.is_type {
            Type::NullType => matches!(expr, Missing | Null),
            Type::MissingType => matches!(expr, Missing),
            _ => todo!("Implement `IS` for other types"),
        };

        Cow::Owned(result.into())
    }
}

/// Represents an evaluation binary operator, e.g.`a + b`.
#[derive(Debug)]
pub struct EvalBinOpExpr {
    pub op: EvalBinOp,
    pub lhs: Box<dyn EvalExpr>,
    pub rhs: Box<dyn EvalExpr>,
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
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
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
            return Cow::Owned(propagate);
        }

        let rhs = self.rhs.evaluate(bindings, ctx);
        let (lhs, rhs) = (lhs.as_ref(), rhs.as_ref());
        let result = match self.op {
            EvalBinOp::And => lhs.and(rhs),
            EvalBinOp::Or => lhs.or(rhs),
            EvalBinOp::Concat => {
                // TODO non-naive concat (i.e., don't just use debug print for non-strings).
                match (&lhs, &rhs) {
                    (Missing, _) => Missing,
                    (_, Missing) => Missing,
                    (Null, _) => Null,
                    (_, Null) => Null,
                    _ => {
                        let lhs = if let Value::String(s) = lhs {
                            s.as_ref().clone()
                        } else {
                            format!("{lhs:?}")
                        };
                        let rhs = if let Value::String(s) = rhs {
                            s.as_ref().clone()
                        } else {
                            format!("{rhs:?}")
                        };
                        Value::String(Box::new(format!("{lhs}{rhs}")))
                    }
                }
            }
            EvalBinOp::Eq => NullableEq::eq(lhs, rhs),
            EvalBinOp::Neq => lhs.neq(rhs),
            EvalBinOp::Gt => NullableOrd::gt(lhs, rhs),
            EvalBinOp::Gteq => NullableOrd::gteq(lhs, rhs),
            EvalBinOp::Lt => NullableOrd::lt(lhs, rhs),
            EvalBinOp::Lteq => NullableOrd::lteq(lhs, rhs),
            EvalBinOp::Add => lhs + rhs,
            EvalBinOp::Sub => lhs - rhs,
            EvalBinOp::Mul => lhs * rhs,
            EvalBinOp::Div => lhs / rhs,
            EvalBinOp::Mod => lhs % rhs,
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
                    for elem in rhs.iter() {
                        // b/c of short_circuiting as we've reached this branch, we know LHS is neither MISSING nor NULL.
                        if elem == lhs {
                            return Cow::Owned(Boolean(true));
                        } else if elem == &Missing {
                            has_missing = true;
                        } else if elem == &Null {
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
        };
        Cow::Owned(result)
    }
}

/// Represents an evaluation PartiQL `BETWEEN` operator, e.g. `x BETWEEN 10 AND 20`.
#[derive(Debug)]
pub struct EvalBetweenExpr {
    pub value: Box<dyn EvalExpr>,
    pub from: Box<dyn EvalExpr>,
    pub to: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalBetweenExpr {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let from = self.from.evaluate(bindings, ctx);
        let to = self.to.evaluate(bindings, ctx);
        let gteq = value.gteq(from.as_ref());
        let lteq = value.lteq(to.as_ref());
        Cow::Owned(gteq.and(&lteq))
    }
}

/// Represents an evaluation `LIKE` operator, e.g. in `s LIKE 'h%llo'`.
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
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let result = match value.as_ref() {
            Null => Null,
            Missing => Missing,
            Value::String(s) => Boolean(self.pattern.is_match(s.as_ref())),
            _ => Missing,
        };
        Cow::Owned(result)
    }
}

/// Represents an evaluation `LIKE` operator without string literals in the match and/or escape
/// pattern, e.g. in `s LIKE match_str ESCAPE escape_char`.
#[derive(Debug)]
pub struct EvalLikeNonStringNonLiteralMatch {
    pub value: Box<dyn EvalExpr>,
    pub pattern: Box<dyn EvalExpr>,
    pub escape: Box<dyn EvalExpr>,
}

impl EvalLikeNonStringNonLiteralMatch {
    pub fn new(
        value: Box<dyn EvalExpr>,
        pattern: Box<dyn EvalExpr>,
        escape: Box<dyn EvalExpr>,
    ) -> Self {
        EvalLikeNonStringNonLiteralMatch {
            value,
            pattern,
            escape,
        }
    }
}

impl EvalExpr for EvalLikeNonStringNonLiteralMatch {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let pattern = self.pattern.evaluate(bindings, ctx);
        let escape = self.escape.evaluate(bindings, ctx);

        let result = match (value.as_ref(), pattern.as_ref(), escape.as_ref()) {
            (Missing, _, _) => Missing,
            (_, Missing, _) => Missing,
            (_, _, Missing) => Missing,
            (Null, _, _) => Null,
            (_, Null, _) => Null,
            (_, _, Null) => Null,
            (Value::String(v), Value::String(p), Value::String(e)) => {
                assert!(e.chars().count() <= 1);
                let escape = e.chars().next();
                let regex_pattern = RegexBuilder::new(&like_to_re_pattern(p, escape))
                    .size_limit(RE_SIZE_LIMIT)
                    .build()
                    .expect("Like Pattern");
                Boolean(regex_pattern.is_match(v.as_ref()))
            }
            _ => Missing,
        };
        Cow::Owned(result)
    }
}

/// Represents a searched case operator, e.g. CASE [ WHEN <expr> THEN <expr> ]... [ ELSE <expr> ] END.
#[derive(Debug)]
pub struct EvalSearchedCaseExpr {
    pub cases: Vec<(Box<dyn EvalExpr>, Box<dyn EvalExpr>)>,
    pub default: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalSearchedCaseExpr {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        for (when_expr, then_expr) in &self.cases {
            let when_expr_evaluated = when_expr.evaluate(bindings, ctx);
            if when_expr_evaluated.as_ref() == &Value::Boolean(true) {
                return then_expr.evaluate(bindings, ctx);
            }
        }
        self.default.evaluate(bindings, ctx)
    }
}

#[inline]
#[track_caller]
fn string_transform<FnTransform>(value: &Value, transform_fn: FnTransform) -> Value
where
    FnTransform: Fn(&str) -> Value,
{
    match value {
        Null => Value::Null,
        Value::String(s) => transform_fn(s.as_ref()),
        _ => Value::Missing,
    }
}

/// Represents a built-in `lower` string function, e.g. lower('AdBd').
#[derive(Debug)]
pub struct EvalFnLower {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnLower {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let transformed = string_transform(self.value.evaluate(bindings, ctx).as_ref(), |s| {
            s.to_lowercase().into()
        });
        Cow::Owned(transformed)
    }
}

/// Represents a built-in `upper` string function, e.g. upper('AdBd').
#[derive(Debug)]
pub struct EvalFnUpper {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnUpper {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let transformed = string_transform(self.value.evaluate(bindings, ctx).as_ref(), |s| {
            s.to_uppercase().into()
        });
        Cow::Owned(transformed)
    }
}

/// Represents a built-in character length string function, e.g. `char_length('123456789')`.
#[derive(Debug)]
pub struct EvalFnCharLength {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnCharLength {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let transformed = string_transform(self.value.evaluate(bindings, ctx).as_ref(), |s| {
            s.chars().count().into()
        });
        Cow::Owned(transformed)
    }
}

/// Represents a built-in octet length string function, e.g. `octet_length('123456789')`.
#[derive(Debug)]
pub struct EvalFnOctetLength {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnOctetLength {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let transformed = string_transform(self.value.evaluate(bindings, ctx).as_ref(), |s| {
            s.len().into()
        });
        Cow::Owned(transformed)
    }
}

/// Represents a built-in bit length string function, e.g. `bit_length('123456789')`.
#[derive(Debug)]
pub struct EvalFnBitLength {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnBitLength {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let transformed = string_transform(self.value.evaluate(bindings, ctx).as_ref(), |s| {
            (s.len() * 8).into()
        });
        Cow::Owned(transformed)
    }
}

/// Represents a built-in substring string function, e.g. `substring('123456789' FROM 2)`.
#[derive(Debug)]
pub struct EvalFnSubstring {
    pub value: Box<dyn EvalExpr>,
    pub offset: Box<dyn EvalExpr>,
    pub length: Option<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalFnSubstring {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let value = match value.as_ref() {
            Null => None,
            Value::String(s) => Some(s),
            _ => return Cow::Owned(Value::Missing),
        };
        let offset = self.offset.evaluate(bindings, ctx);
        let offset = match offset.as_ref() {
            Null => None,
            Value::Integer(i) => Some(i),
            _ => return Cow::Owned(Value::Missing),
        };

        let result = if let Some(length) = &self.length {
            let length = match length.evaluate(bindings, ctx).as_ref() {
                Value::Integer(i) => *i as usize,
                Value::Null => return Cow::Owned(Value::Null),
                _ => return Cow::Owned(Value::Missing),
            };
            if let (Some(value), Some(&offset)) = (value, offset) {
                let (offset, length) = if length < 1 {
                    (0, 0)
                } else if offset < 1 {
                    let length = std::cmp::max(offset + (length - 1) as i64, 0) as usize;
                    let offset = std::cmp::max(offset, 0) as usize;
                    (offset, length)
                } else {
                    ((offset - 1) as usize, length)
                };
                value
                    .chars()
                    .skip(offset)
                    .take(length)
                    .collect::<String>()
                    .into()
            } else {
                // either value or offset was NULL; return NULL
                Value::Null
            }
        } else if let (Some(value), Some(&offset)) = (value, offset) {
            let offset = (std::cmp::max(offset, 1) - 1) as usize;
            value.chars().skip(offset).collect::<String>().into()
        } else {
            // either value or offset was NULL; return NULL
            Value::Null
        };
        Cow::Owned(result)
    }
}

/// Represents a built-in position string function, e.g. `position('3' IN '123456789')`.
#[derive(Debug)]
pub struct EvalFnPosition {
    pub needle: Box<dyn EvalExpr>,
    pub haystack: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnPosition {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let needle = self.needle.evaluate(bindings, ctx);
        let needle = match needle.as_ref() {
            Null => None,
            Value::String(s) => Some(s),
            _ => return Cow::Owned(Value::Missing),
        };
        let haystack = self.haystack.evaluate(bindings, ctx);
        let haystack = match haystack.as_ref() {
            Value::Null => return Cow::Owned(Value::Null),
            Value::String(s) => s,
            _ => return Cow::Owned(Value::Missing),
        };
        let result = if let Some(needle) = needle {
            haystack
                .find(needle.as_ref())
                .map(|l| l + 1)
                .unwrap_or(0)
                .into()
        } else {
            Value::Null
        };
        Cow::Owned(result)
    }
}

/// Represents a built-in overlay string function, e.g. `OVERLAY('hello' PLACING 'XX' FROM 2 FOR 3)`.
#[derive(Debug)]
pub struct EvalFnOverlay {
    pub value: Box<dyn EvalExpr>,
    pub replacement: Box<dyn EvalExpr>,
    pub offset: Box<dyn EvalExpr>,
    pub length: Option<Box<dyn EvalExpr>>,
}

impl EvalExpr for EvalFnOverlay {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let value = match value.as_ref() {
            Null => None,
            Value::String(s) => Some(s),
            _ => return Cow::Owned(Value::Missing),
        };
        let replacement = self.replacement.evaluate(bindings, ctx);
        let replacement = match replacement.as_ref() {
            Null => None,
            Value::String(s) => Some(s),
            _ => return Cow::Owned(Value::Missing),
        };
        let offset = self.offset.evaluate(bindings, ctx);
        let offset = match offset.as_ref() {
            Null => None,
            Value::Integer(i) => Some(i),
            _ => return Cow::Owned(Value::Missing),
        };

        let length = if let Some(length) = &self.length {
            match length.evaluate(bindings, ctx).as_ref() {
                Value::Integer(i) => *i as usize,
                Value::Null => return Cow::Owned(Value::Null),
                _ => return Cow::Owned(Value::Missing),
            }
        } else if let Some(replacement) = &replacement {
            replacement.len()
        } else {
            // either replacement or length was NULL; return NULL
            return Cow::Owned(Value::Null);
        };

        let result =
            if let (Some(value), Some(replacement), Some(&offset)) = (value, replacement, offset) {
                let mut value = *value.clone();
                let start = std::cmp::max(offset - 1, 0) as usize;
                if start > value.len() {
                    value += replacement;
                } else {
                    let end = std::cmp::min(start + length, value.len());
                    value.replace_range(start..end, replacement);
                }

                Value::from(value)
            } else {
                // either value, replacement, or offset was NULL; return NULL
                Value::Null
            };

        Cow::Owned(result)
    }
}

#[inline]
#[track_caller]
fn trim<'a, FnTrim>(value: &'a Value, to_trim: &'a Value, trim_fn: FnTrim) -> Value
where
    FnTrim: Fn(&'a str, &'a str) -> &'a str,
{
    let value = match value {
        Value::String(s) => Some(s),
        Null => None,
        _ => return Value::Missing,
    };
    let to_trim = match to_trim {
        Value::String(s) => s,
        Null => return Value::Null,
        _ => return Value::Missing,
    };
    if let Some(s) = value {
        let trimmed = trim_fn(s, to_trim);
        Value::from(trimmed)
    } else {
        Value::Null
    }
}

/// Represents a built-in both trim string function, e.g. `trim(both from ' foobar ')`.
#[derive(Debug)]
pub struct EvalFnBtrim {
    pub value: Box<dyn EvalExpr>,
    pub to_trim: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnBtrim {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let to_trim = self.to_trim.evaluate(bindings, ctx);
        let trimmed = trim(value.as_ref(), to_trim.as_ref(), |s, to_trim| {
            let to_trim = to_trim.chars().collect_vec();
            s.trim_matches(&to_trim[..])
        });
        Cow::Owned(trimmed)
    }
}

/// Represents a built-in right trim string function.
#[derive(Debug)]
pub struct EvalFnRtrim {
    pub value: Box<dyn EvalExpr>,
    pub to_trim: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnRtrim {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let to_trim = self.to_trim.evaluate(bindings, ctx);
        let trimmed = trim(value.as_ref(), to_trim.as_ref(), |s, to_trim| {
            let to_trim = to_trim.chars().collect_vec();
            s.trim_end_matches(&to_trim[..])
        });
        Cow::Owned(trimmed)
    }
}

/// Represents a built-in left trim string function.
#[derive(Debug)]
pub struct EvalFnLtrim {
    pub value: Box<dyn EvalExpr>,
    pub to_trim: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnLtrim {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let to_trim = self.to_trim.evaluate(bindings, ctx);
        let trimmed = trim(value.as_ref(), to_trim.as_ref(), |s, to_trim| {
            let to_trim = to_trim.chars().collect_vec();
            s.trim_start_matches(&to_trim[..])
        });
        Cow::Owned(trimmed)
    }
}

/// Represents an `EXISTS` function, e.g. `exists(`(1)`)`.
#[derive(Debug)]
pub struct EvalFnExists {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnExists {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let exists = match value.borrow() {
            Value::Bag(b) => !b.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Tuple(t) => !t.is_empty(),
            _ => false,
        };
        Cow::Owned(Value::Boolean(exists))
    }
}

/// Represents an `ABS` function, e.g. `abs(-1)`.
#[derive(Debug)]
pub struct EvalFnAbs {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnAbs {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let lhs: &Value = value.borrow();
        let rhs = 0.into();
        match NullableOrd::lt(lhs, &rhs) {
            Null => Cow::Owned(Null),
            Missing => Cow::Owned(Missing),
            Value::Boolean(true) => Cow::Owned(-value.into_owned()),
            _ => Cow::Owned(value.into_owned()),
        }
    }
}

/// Represents an `MOD` function, e.g. `MOD(10, 1)`.
#[derive(Debug)]
pub struct EvalFnModulus {
    pub lhs: Box<dyn EvalExpr>,
    pub rhs: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnModulus {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let lhs = self.lhs.evaluate(bindings, ctx);
        let lhs = match lhs.as_ref() {
            Null => None,
            Missing => return Cow::Owned(Value::Missing),
            _ => Some(lhs),
        };
        let rhs = self.rhs.evaluate(bindings, ctx);
        let rhs = match rhs.as_ref() {
            Value::Null => return Cow::Owned(Value::Null),
            Missing => return Cow::Owned(Value::Missing),
            _ => rhs,
        };

        if let Some(lhs) = lhs {
            let lhs: &Value = lhs.borrow();
            let rhs: &Value = rhs.borrow();
            Cow::Owned(lhs % rhs)
        } else {
            Cow::Owned(Value::Null)
        }
    }
}

/// Represents an `CARDINALITY` function, e.g. `cardinality([1,2,3])`.
#[derive(Debug)]
pub struct EvalFnCardinality {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnCardinality {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let result = match value.borrow() {
            Null => Null,
            Missing => Missing,
            Value::List(l) => Value::from(l.len()),
            Value::Bag(b) => Value::from(b.len()),
            Value::Tuple(t) => Value::from(t.len()),
            _ => Missing,
        };
        Cow::Owned(result)
    }
}

/// Represents a year `EXTRACT` function, e.g. `extract(YEAR FROM t)`.
#[derive(Debug)]
pub struct EvalFnExtractYear {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnExtractYear {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let result = match value.borrow() {
            Null => Null,
            Missing => Missing,
            Value::DateTime(dt) => match dt.as_ref() {
                DateTime::Date(d) => Value::from(d.year()),
                DateTime::Timestamp(tstamp) => Value::from(tstamp.year()),
                DateTime::TimestampWithTz(tstamp) => Value::from(tstamp.year()),
                DateTime::Time(_) => Missing,
                DateTime::TimeWithTz(_, _) => Missing,
            },
            _ => Missing,
        };
        Cow::Owned(result)
    }
}

/// Represents a month `EXTRACT` function, e.g. `extract(MONTH FROM t)`.
#[derive(Debug)]
pub struct EvalFnExtractMonth {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnExtractMonth {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let result = match value.borrow() {
            Null => Null,
            Missing => Missing,
            Value::DateTime(dt) => match dt.as_ref() {
                DateTime::Date(d) => Value::from(d.month() as u8),
                DateTime::Timestamp(tstamp) => Value::from(tstamp.month() as u8),
                DateTime::TimestampWithTz(tstamp) => Value::from(tstamp.month() as u8),
                DateTime::Time(_) => Missing,
                DateTime::TimeWithTz(_, _) => Missing,
            },
            _ => Missing,
        };
        Cow::Owned(result)
    }
}

/// Represents a day `EXTRACT` function, e.g. `extract(DAT FROM t)`.
#[derive(Debug)]
pub struct EvalFnExtractDay {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnExtractDay {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let result = match value.borrow() {
            Null => Null,
            Missing => Missing,
            Value::DateTime(dt) => match dt.as_ref() {
                DateTime::Date(d) => Value::from(d.day()),
                DateTime::Timestamp(tstamp) => Value::from(tstamp.day()),
                DateTime::TimestampWithTz(tstamp) => Value::from(tstamp.day()),
                DateTime::Time(_) => Missing,
                DateTime::TimeWithTz(_, _) => Missing,
            },
            _ => Missing,
        };
        Cow::Owned(result)
    }
}

/// Represents an hour `EXTRACT` function, e.g. `extract(HOUR FROM t)`.
#[derive(Debug)]
pub struct EvalFnExtractHour {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnExtractHour {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let result = match value.borrow() {
            Null => Null,
            Missing => Missing,
            Value::DateTime(dt) => match dt.as_ref() {
                DateTime::Time(t) => Value::from(t.hour()),
                DateTime::TimeWithTz(t, _) => Value::from(t.hour()),
                DateTime::Timestamp(tstamp) => Value::from(tstamp.hour()),
                DateTime::TimestampWithTz(tstamp) => Value::from(tstamp.hour()),
                DateTime::Date(_) => Missing,
            },
            _ => Missing,
        };
        Cow::Owned(result)
    }
}

/// Represents a minute `EXTRACT` function, e.g. `extract(MINUTE FROM t)`.
#[derive(Debug)]
pub struct EvalFnExtractMinute {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnExtractMinute {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let result = match value.borrow() {
            Null => Null,
            Missing => Missing,
            Value::DateTime(dt) => match dt.as_ref() {
                DateTime::Time(t) => Value::from(t.minute()),
                DateTime::TimeWithTz(t, _) => Value::from(t.minute()),
                DateTime::Timestamp(tstamp) => Value::from(tstamp.minute()),
                DateTime::TimestampWithTz(tstamp) => Value::from(tstamp.minute()),
                DateTime::Date(_) => Missing,
            },
            _ => Missing,
        };
        Cow::Owned(result)
    }
}

/// Represents a second `EXTRACT` function, e.g. `extract(SECOND FROM t)`.
#[derive(Debug)]
pub struct EvalFnExtractSecond {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnExtractSecond {
    // TODO: doesn't currently extract fractional seconds
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let (second, nanosecond) = match value.borrow() {
            Null => return Cow::Owned(Null),
            Missing => return Cow::Owned(Missing),
            Value::DateTime(dt) => match dt.as_ref() {
                DateTime::Time(t) => (t.second(), t.nanosecond()),
                DateTime::TimeWithTz(t, _) => (t.second(), t.nanosecond()),
                DateTime::Timestamp(tstamp) => (tstamp.second(), tstamp.nanosecond()),
                DateTime::TimestampWithTz(tstamp) => (tstamp.second(), tstamp.nanosecond()),
                DateTime::Date(_) => return Cow::Owned(Missing),
            },
            _ => return Cow::Owned(Missing),
        };
        let result =
            rust_decimal::Decimal::from_f64(((second as f64 * 1e9) + nanosecond as f64) / 1e9)
                .expect("time as decimal");
        Cow::Owned(Value::from(result))
    }
}

/// Represents a timezone hour `EXTRACT` function, e.g. `extract(TIMEZONE_HOUR FROM t)`.
#[derive(Debug)]
pub struct EvalFnExtractTimezoneHour {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnExtractTimezoneHour {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let result = match value.borrow() {
            Null => Null,
            Missing => Missing,
            Value::DateTime(dt) => match dt.as_ref() {
                DateTime::TimeWithTz(_, tz) => Value::from(tz.whole_hours()),
                DateTime::TimestampWithTz(tstamp) => Value::from(tstamp.offset().whole_hours()),
                DateTime::Date(_) => Missing,
                DateTime::Time(_) => Missing,
                DateTime::Timestamp(_) => Missing,
            },
            _ => Missing,
        };
        Cow::Owned(result)
    }
}

/// Represents a timezone minute `EXTRACT` function, e.g. `extract(TIMEZONE_MINUTE FROM t)`.
#[derive(Debug)]
pub struct EvalFnExtractTimezoneMinute {
    pub value: Box<dyn EvalExpr>,
}

impl EvalExpr for EvalFnExtractTimezoneMinute {
    #[inline]
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        let value = self.value.evaluate(bindings, ctx);
        let result = match value.borrow() {
            Null => Null,
            Missing => Missing,
            Value::DateTime(dt) => match dt.as_ref() {
                DateTime::TimeWithTz(_, tz) => Value::from(tz.minutes_past_hour()),
                DateTime::TimestampWithTz(tstamp) => {
                    Value::from(tstamp.offset().minutes_past_hour() % 60)
                }
                DateTime::Date(_) => Missing,
                DateTime::Time(_) => Missing,
                DateTime::Timestamp(_) => Missing,
            },
            _ => Missing,
        };
        Cow::Owned(result)
    }
}
