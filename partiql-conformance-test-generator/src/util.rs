use inflector::Inflector;

pub trait StringExt {
    /// Converts a string slice into an escaped `String`
    fn escaped_snake_case(&self) -> String;
}

impl StringExt for &str {
    fn escaped_snake_case(&self) -> String {
        // TODO: currently non-text, non-numeric tokens are ignored and whitespace is converted to
        //  underscores. More complicated escaping can be done to preserve more tokens
        //  (e.g. punctuation)
        self.to_snake_case()
    }
}

impl StringExt for String {
    fn escaped_snake_case(&self) -> String {
        self.as_str().escaped_snake_case()
    }
}

#[cfg(test)]
mod test {
    use crate::util::StringExt;

    #[test]
    fn escaping_letters_and_whitespace() {
        assert_eq!("a B c \t D \n e_f_G".escaped_snake_case(), "a_b_c_d_e_f_g")
    }

    #[test]
    fn escaping_letters_numbers_other_chars() {
        assert_eq!(
            "a B c  1 2 3 e f G !?#$%*!(".escaped_snake_case(),
            "a_b_c_1_2_3_e_f_g"
        )
    }
}
