use ion_rs::value::reader::{element_reader, ElementReader};
use partiql_conformance_test_generator::generator::Generator;
use partiql_conformance_test_generator::ion_data_to_test_document;
use partiql_conformance_test_generator::util::{all_ion_files_in, dir_to_mods, StringExt};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

/// TODO: once APIs are more stable, include documentation on what this build script is doing
fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=partiql-tests");

    let tests_dir = "tests/";
    let tests_path = Path::new(tests_dir);

    // TODO: consider first moving directory and deleting once test generation is successful
    if tests_path.exists() {
        fs::remove_dir_all(tests_dir).expect("removal of tests/ before test generation");
    }

    let file_dir = "partiql-tests";
    let all_files = all_ion_files_in(file_dir).expect("test files");

    for file in &all_files {
        let all_data = fs::read(file).unwrap();
        let all_ion_data = element_reader().read_all(&all_data).unwrap();

        let test_document = ion_data_to_test_document(all_ion_data);
        let test_generator = Generator { test_document };
        let scope = test_generator.generate_scope().to_string();

        let dest_path_non_escaped = Path::new("tests").join(file.with_extension(""));
        let dest_path_escaped: PathBuf = dest_path_non_escaped
            .iter()
            .map(|path_part| path_part.to_str().expect("to_str").escaped_snake_case())
            .collect();
        let dest_path = dest_path_escaped.with_extension("rs");

        let dest_dir = dest_path.parent().expect("parent of dest_path");
        std::fs::create_dir_all(dest_dir).expect("recursively created directory");
        File::create(dest_path)
            .expect("File creation failed")
            .write_all(scope.as_bytes())
            .expect("Failure when writing to file");
    }

    let sub_tests_dir = "partiql_tests";
    let sub_tests_path = Path::new(sub_tests_dir);
    dir_to_mods(&tests_path.join(sub_tests_path));
    // Creates the top-level mod.rs file in the tests/ directory and adds the "conformance_test"
    // flag to this mod
    File::create(&tests_path.join("mod.rs"))
        .expect("mod.rs created in root test folder")
        .write_all(
            format!(
                "#[cfg(feature = \"conformance_test\")]\nmod {};\n",
                sub_tests_dir
            )
            .as_bytes(),
        )
        .expect("Failure when writing to file");

    Command::new("cargo")
        .arg("fmt")
        .arg("--verbose")
        .arg("--")
        .spawn()
        .expect("cargo fmt of tests/ failed");
    Ok(())
}
