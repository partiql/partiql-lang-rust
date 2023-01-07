// TODO docs

// TODO consider how to use the appropriate mechanisms to prevent exhaustion of
//      resources by query-written regexes
//      See https://docs.rs/regex/latest/regex/#untrusted-input

// TODO I believe this should be resilient ReDoS, as this should never build
//      an 'Evil Regex' as defined by
//      https://owasp.org/www-community/attacks/Regular_expression_Denial_of_Service_-_ReDoS
pub fn like_to_re_pattern(like_expr: &str, escape: Option<char>) -> String {
    to_re_pattern(like_expr, escape, regex_syntax::is_meta_character)
}

// TODO SIMILAR probably needs to be better thought through for preventing ReDoS
//      A query writer would be able to build an 'Evil Regex' as defined by
//      https://owasp.org/www-community/attacks/Regular_expression_Denial_of_Service_-_ReDoS
//      For now, it is not publically exported for the above reason.
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
                    buf.push_str(".*?")
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
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn like() {
        assert_eq!(like_to_re_pattern("foo", Some('\\')), r#"^foo$"#);
        assert_eq!(like_to_re_pattern("%foo", Some('\\')), r#"^.*?foo$"#);
        assert_eq!(like_to_re_pattern("foo%", Some('\\')), r#"^foo.*?$"#);
        assert_eq!(like_to_re_pattern("foo%bar", Some('\\')), r#"^foo.*?bar$"#);
        assert_eq!(like_to_re_pattern("foo%%bar", Some('\\')), r#"^foo.*?bar$"#);
        assert_eq!(
            like_to_re_pattern("foo%%%bar", Some('\\')),
            r#"^foo.*?bar$"#
        );
        assert_eq!(
            like_to_re_pattern("foo%%%%bar", Some('\\')),
            r#"^foo.*?bar$"#
        );
        assert_eq!(
            like_to_re_pattern("%foo%%%%bar%", Some('\\')),
            r#"^.*?foo.*?bar.*?$"#
        );
        assert_eq!(
            like_to_re_pattern("%foo%%%%bar\\%baz%", Some('\\')),
            r#"^.*?foo.*?bar%baz.*?$"#
        );
        assert_eq!(
            like_to_re_pattern("%foo%%%%bar*%baz%", Some('*')),
            r#"^.*?foo.*?bar%baz.*?$"#
        );
        assert_eq!(like_to_re_pattern("_foo", Some('\\')), r#"^.foo$"#);
        assert_eq!(like_to_re_pattern("foo_", Some('\\')), r#"^foo.$"#);
        assert_eq!(like_to_re_pattern("foo_bar", Some('\\')), r#"^foo.bar$"#);
        assert_eq!(like_to_re_pattern("foo__bar", Some('\\')), r#"^foo..bar$"#);
        assert_eq!(
            like_to_re_pattern("foo_.*?_bar", Some('\\')),
            r#"^foo.\.\*\?.bar$"#
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
        assert_eq!(similar_to_re_pattern("(b|c)%", Some('\\')), r#"^(b|c).*?$"#);
        assert_eq!(
            similar_to_re_pattern("%(b|d)%", Some('\\')),
            r#"^.*?(b|d).*?$"#
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
