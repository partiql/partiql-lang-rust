use inflector::Inflector;
use std::ffi::OsStr;
use std::ops::Add;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

/// Returns a file name with prefix prepended and 'rs' as the extension
pub fn to_full_file_name(prefix: &str, ion_file: &Path) -> String {
    let ion_file = ion_file
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .escaped_snake_case()
        .add(".rs");
    format!("{}_{}", prefix, ion_file)
}

/// Returns a vector of all .ion files in the directory `dir`
pub fn all_ion_files_in(dir: &str) -> walkdir::Result<Vec<PathBuf>> {
    let ion_file_extension = OsStr::new("ion");
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|result| match result {
            Ok(entry) => {
                if entry.path().extension() == Some(ion_file_extension) {
                    Some(Ok(entry.into_path()))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(e)),
        })
        .collect()
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
