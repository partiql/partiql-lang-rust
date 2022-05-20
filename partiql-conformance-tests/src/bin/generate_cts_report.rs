use serde_json::json;
use serde_json::{to_string_pretty, Value};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

// TODO: docs
fn main() {
    let args: Vec<String> = env::args().collect();

    // TODO: add argument checking
    let cargo_test_source_file = &args[1];
    let commit_hash = &args[2];
    let output_file_name = &args[3];

    let file = File::open(cargo_test_source_file).expect("open");
    let reader = BufReader::new(file);

    let mut all_passing_test_names: Vec<Value> = Vec::new();
    let mut all_failing_test_names: Vec<Value> = Vec::new();

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let v: Value = serde_json::from_str(&*line).expect("from_str");
                if v["type"] == "test" {
                    let event = v["event"].as_str().expect("as_str");
                    if event == "ok" {
                        all_passing_test_names.push(v["name"].to_owned());
                    } else if event == "failed" {
                        all_failing_test_names.push(v["name"].to_owned());
                    }
                }
            }
            Err(e) => println!("Error reading line: {}", e),
        }
    }

    let report_as_json = json!({
        "commit_hash": commit_hash,
        "passing": all_passing_test_names,
        "failing": all_failing_test_names
    });

    File::create(output_file_name)
        .expect("File create")
        .write_all(
            to_string_pretty(&report_as_json)
                .expect("to_string_pretty")
                .as_bytes(),
        )
        .expect("Failure when writing to file")
}
