use crate::error::PlanningError;

use crate::eval::eval_expr_wrapper::{TernaryValueExpr, UnaryValueExpr};
use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};
use partiql_types::{type_string, DummyShapeBuilder};
use partiql_value::Value;
use partiql_value::Value::Missing;
use regex::{Regex, RegexBuilder};

// TODO make configurable?
// Limit chosen somewhat arbitrarily, but to be smaller than the default of `10 * (1 << 20)`
const RE_SIZE_LIMIT: usize = 1 << 16;

/// Represents an evaluation `LIKE` operator, e.g. in `s LIKE 'h%llo'`.
#[derive(Debug)]
pub(crate) struct EvalLikeMatch {
    pub(crate) pattern: Regex,
}

impl EvalLikeMatch {
    fn new(pattern: Regex) -> Self {
        EvalLikeMatch { pattern }
    }

    pub(crate) fn create(pattern: &str, escape: &str) -> Result<Self, PlanningError> {
        if escape.chars().count() > 1 {
            return Err(PlanningError::IllegalState(format!(
                "Invalid LIKE expression pattern: {escape}"
            )));
        }

        let escape = escape.chars().next();
        let regex = like_to_re_pattern(pattern, escape);
        let regex_pattern = RegexBuilder::new(&regex).size_limit(RE_SIZE_LIMIT).build();
        match regex_pattern {
            Ok(pattern) => Ok(EvalLikeMatch::new(pattern)),
            Err(err) => Err(PlanningError::IllegalState(format!(
                "Invalid LIKE expression pattern: {regex}. Regex error: {err}"
            ))),
        }
    }
}

impl BindEvalExpr for EvalLikeMatch {
    fn bind<const STRICT: bool>(
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        // use DummyShapeBuilder, as we don't care about shape Ids for evaluation dispatch
        let mut bld = DummyShapeBuilder::default();
        let pattern = self.pattern.clone();
        UnaryValueExpr::create_typed::<{ STRICT }, _>([type_string!(bld)], args, move |value| {
            match value {
                Value::String(s) => Value::Boolean(pattern.is_match(s.as_ref())),
                _ => Missing,
            }
        })
    }
}

/// Represents an evaluation `LIKE` operator without string literals in the match and/or escape
/// pattern, e.g. in `s LIKE match_str ESCAPE escape_char`.
#[derive(Debug)]
pub(crate) struct EvalLikeNonStringNonLiteralMatch {}

impl BindEvalExpr for EvalLikeNonStringNonLiteralMatch {
    fn bind<const STRICT: bool>(
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        // use DummyShapeBuilder, as we don't care about shape Ids for evaluation dispatch
        let mut bld = DummyShapeBuilder::default();
        let types = [type_string!(bld), type_string!(bld), type_string!(bld)];
        TernaryValueExpr::create_typed::<{ STRICT }, _>(
            types,
            args,
            |value, pattern, escape| match (value, pattern, escape) {
                (Value::String(v), Value::String(p), Value::String(e)) => {
                    if e.chars().count() > 1 {
                        // TODO re-instate once eval closures can generate errors for STRICT mode
                        /*
                        ctx.add_error(EvaluationError::IllegalState(
                            "escape longer than 1 character".to_string(),
                        ));
                         */
                    }
                    let escape = e.chars().next();
                    let regex_pattern = RegexBuilder::new(&like_to_re_pattern(p, escape))
                        .size_limit(RE_SIZE_LIMIT)
                        .build();
                    match regex_pattern {
                        Ok(pattern) => Value::Boolean(pattern.is_match(v.as_ref())),
                        Err(_err) => {
                            // TODO re-instate once eval closures can generate errors for STRICT mode
                            //ctx.add_error(EvaluationError::IllegalState(err.to_string()));
                            Missing
                        }
                    }
                }
                _ => Missing,
            },
        )
    }
}

// TODO docs

// TODO consider how to use the appropriate mechanisms to prevent exhaustion of
//      resources by query-written regexes
//      See https://docs.rs/regex/latest/regex/#untrusted-input

// TODO I believe this should be resilient ReDoS, as this should never build
//      an 'Evil Regex' as defined by
//      https://owasp.org/www-community/attacks/Regular_expression_Denial_of_Service_-_ReDoS
fn like_to_re_pattern(like_expr: &str, escape: Option<char>) -> String {
    to_re_pattern(like_expr, escape, regex_syntax::is_meta_character)
}

// TODO SIMILAR probably needs to be better thought through for preventing ReDoS
//      A query writer would be able to build an 'Evil Regex' as defined by
//      https://owasp.org/www-community/attacks/Regular_expression_Denial_of_Service_-_ReDoS
//      For now, it is not publicly exported for the above reason.
#[allow(dead_code)]
fn similar_to_re_pattern(similar_expr: &str, escape: Option<char>) -> String {
    to_re_pattern(similar_expr, escape, is_similar_meta_character)
}

#[inline]
#[allow(dead_code)]
fn is_similar_meta_character(c: char) -> bool {
    match c {
        // pass these through to be interpreted as regex meta characters
        '|' | '*' | '+' | '?' | '{' | '}' | '(' | ')' | '[' | ']' => false,
        // everything else, defer
        _ => regex_syntax::is_meta_character(c),
    }
}

#[inline]
fn to_re_pattern<F>(expr: &str, escape: Option<char>, is_meta_character: F) -> String
where
    F: Fn(char) -> bool,
{
    let mut pattern = String::from("^");
    write_re_pattern(expr, escape, is_meta_character, &mut pattern);
    pattern += "$";
    pattern
}

#[inline]
fn write_re_pattern<F>(
    like_expr: &str,
    escape_ch: Option<char>,
    is_meta_character: F,
    buf: &mut String,
) where
    F: Fn(char) -> bool,
{
    buf.reserve(like_expr.len() + 6);
    let mut escaped = false;
    let mut wildcard = false;

    for ch in like_expr.chars() {
        let is_any = std::mem::replace(&mut wildcard, false);
        let is_escaped = std::mem::replace(&mut escaped, false);
        match (ch, is_escaped) {
            (_, false) if Some(ch) == escape_ch => escaped = true,
            ('%', false) => {
                if !is_any {
                    buf.push_str(".*?");
                }
                wildcard = true;
            }
            ('_', false) => buf.push('.'),
            _ => {
                if is_meta_character(ch) {
                    buf.push('\\'); // regex-escape the next character
                }
                buf.push(ch);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn like() {
        assert_eq!(like_to_re_pattern("foo", Some('\\')), r"^foo$");
        assert_eq!(like_to_re_pattern("%foo", Some('\\')), r"^.*?foo$");
        assert_eq!(like_to_re_pattern("foo%", Some('\\')), r"^foo.*?$");
        assert_eq!(like_to_re_pattern("foo%bar", Some('\\')), r"^foo.*?bar$");
        assert_eq!(like_to_re_pattern("foo%%bar", Some('\\')), r"^foo.*?bar$");
        assert_eq!(like_to_re_pattern("foo%%%bar", Some('\\')), r"^foo.*?bar$");
        assert_eq!(like_to_re_pattern("foo%%%%bar", Some('\\')), r"^foo.*?bar$");
        assert_eq!(
            like_to_re_pattern("%foo%%%%bar%", Some('\\')),
            r"^.*?foo.*?bar.*?$"
        );
        assert_eq!(
            like_to_re_pattern("%foo%%%%bar\\%baz%", Some('\\')),
            r"^.*?foo.*?bar%baz.*?$"
        );
        assert_eq!(
            like_to_re_pattern("%foo%%%%bar*%baz%", Some('*')),
            r"^.*?foo.*?bar%baz.*?$"
        );
        assert_eq!(like_to_re_pattern("_foo", Some('\\')), r"^.foo$");
        assert_eq!(like_to_re_pattern("foo_", Some('\\')), r"^foo.$");
        assert_eq!(like_to_re_pattern("foo_bar", Some('\\')), r"^foo.bar$");
        assert_eq!(like_to_re_pattern("foo__bar", Some('\\')), r"^foo..bar$");
        assert_eq!(
            like_to_re_pattern("foo_.*?_bar", Some('\\')),
            r"^foo.\.\*\?.bar$"
        );
    }

    #[test]
    fn like_match() {
        let pat = like_to_re_pattern("foo_.*?_bar", Some('\\'));
        let re = Regex::new(&pat).unwrap();

        assert!(re.is_match("foos.*?qbar"));
    }

    #[test]
    fn similar() {
        assert_eq!(similar_to_re_pattern("(b|c)%", Some('\\')), r"^(b|c).*?$");
        assert_eq!(
            similar_to_re_pattern("%(b|d)%", Some('\\')),
            r"^.*?(b|d).*?$"
        );
    }

    #[test]
    fn similar_match() {
        let pat = similar_to_re_pattern("(b|c)%", Some('\\'));
        let re = Regex::new(&pat).unwrap();
        assert!(!re.is_match("abc"));

        let pat = similar_to_re_pattern("%(b|d)%", Some('\\'));
        let re = Regex::new(&pat).unwrap();
        assert!(re.is_match("abc"));
    }
}
