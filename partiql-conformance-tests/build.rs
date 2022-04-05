use ion_rs::value::reader::{element_reader, ElementReader};
use partiql_conformance_test_generator::ion_data_to_test_document;
use partiql_conformance_test_generator::util::{all_ion_files_in, to_full_file_name, StringExt};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

/// TODO: once APIs are more stable, include documentation on what this build script is doing
fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=partiql-tests/*");

    let file_dir = "partiql-tests";
    let all_files = all_ion_files_in(file_dir).expect("test files");

    for file in &all_files {
        let file_parent_path = file
            .parent()
            .expect("parent")
            .to_str()
            .expect("to_str")
            .escaped_snake_case();
        let full_file_name = to_full_file_name(&file_parent_path, file);
        let dest_path: PathBuf = ["tests", full_file_name.as_str()].iter().collect();

        let all_data = fs::read(file).unwrap();
        let all_ion_data = element_reader().read_all(&all_data).unwrap();

        let test_doc = ion_data_to_test_document(all_ion_data);
        let scope = test_doc.generate_scope();

        File::create(dest_path)
            .expect("File creation failed")
            .write_all(scope.to_string().as_bytes())
            .unwrap_or_else(|error| panic!("Failure when writing to file: {:?}", error));
    }

    Ok(())
}
