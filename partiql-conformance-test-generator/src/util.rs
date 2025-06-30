use inflector::Inflector;
use itertools::Itertools;
use quote::__private::TokenStream;

pub trait Escaper {
    /// Escapes a string intended to be used in a file path
    fn escape_path(&self) -> String;

    /// Escapes a string intended to be used as the name of a test
    fn escape_test_name(&self) -> String;

    /// Escapes a string intended to be used as the name of a module
    fn escape_module_name(&self) -> String;
}

fn escape_str(s: &str) -> String {
    let snake_case: String = s
        .split(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .filter(|s| {
            !s.chars()
                .all(|c| c.is_ascii_punctuation() || c.is_whitespace())
        })
        .map(|s| s.to_snake_case())
        .filter(|s| !s.is_empty())
        .join("_");

    match s.chars().next() {
        None => "_".to_string(),
        Some(c) => {
            if c.is_numeric() {
                format!("_{snake_case}")
            } else {
                snake_case
            }
        }
    }
}

impl Escaper for &str {
    fn escape_path(&self) -> String {
        escape_str(self)
    }

    fn escape_test_name(&self) -> String {
        format!("r#{}", escape_str(self))
    }
    fn escape_module_name(&self) -> String {
        format!("r#{}", escape_str(self))
    }
}

impl Escaper for String {
    fn escape_path(&self) -> String {
        self.as_str().escape_path()
    }

    fn escape_test_name(&self) -> String {
        self.as_str().escape_test_name()
    }

    fn escape_module_name(&self) -> String {
        self.as_str().escape_module_name()
    }
}

#[inline]
pub fn escape_fn_code(ts: TokenStream) -> String {
    ts.to_string().replace("\\n", "\n")
}

#[cfg(test)]
mod test {
    use crate::util::Escaper;

    #[test]
    fn escaping_letters_and_whitespace() {
        assert_eq!("a B c \t D \n e_f_G".escape_path(), "a_b_c_d_e_f_g");
        assert_eq!("a B c \t D \n e_f_G".escape_test_name(), "r#a_b_c_d_e_f_g");
        assert_eq!(
            "a B c \t D \n e_f_G".escape_module_name(),
            "r#a_b_c_d_e_f_g"
        );
    }

    #[test]
    fn escaping_letters_numbers_other_chars() {
        assert_eq!(
            "a B c  1 2 3 e f G !?#$%*!(".escape_path(),
            "a_b_c_1_2_3_e_f_g"
        );
        assert_eq!(
            "a B c  1 2 3 e f G !?#$%*!(".escape_test_name(),
            "r#a_b_c_1_2_3_e_f_g"
        );
        assert_eq!(
            "a B c  1 2 3 e f G !?#$%*!(".escape_module_name(),
            "r#a_b_c_1_2_3_e_f_g"
        );
    }

    #[test]
    fn snake_case_uppercase_names() {
        assert_eq!(
            "Example 7 — NULL and MISSING Coercion - 1".escape_path(),
            "example_7_null_and_missing_coercion_1"
        );
    }

    #[test]
    fn snake_case_uppercase_names_without_spaces() {
        assert_eq!(
            "cast to int invalid target type{value:\"`(1 2)`\",target:\"SEXP\"}".escape_path(),
            "cast_to_int_invalid_target_type_value_1_2_target_sexp"
        );
    }

    #[test]
    fn snake_case_camelcased() {
        assert_eq!("rangeOverSexp".escape_path(), "range_over_sexp");
    }
}
