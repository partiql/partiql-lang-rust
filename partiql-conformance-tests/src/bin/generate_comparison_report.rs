use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use std::{env, fs};

#[derive(Serialize, Deserialize, Debug)]
struct CTSReport {
    commit_hash: String,
    passing: Vec<String>,
    failing: Vec<String>,
}

// TODO: docs
fn main() {
    let args: Vec<String> = env::args().collect();

    // TODO: add argument checking
    let orig_path = &args[1];
    let new_path = &args[2];

    let orig_results = fs::read_to_string(orig_path).unwrap();
    let new_results = fs::read_to_string(new_path).unwrap();

    let orig_report: CTSReport = serde_json::from_str(&*orig_results).expect("from_str");
    let new_report: CTSReport = serde_json::from_str(&*new_results).expect("from_str");

    let orig_failing: HashSet<String> = HashSet::from_iter(orig_report.failing);
    let new_failing: HashSet<String> = HashSet::from_iter(new_report.failing);

    let orig_passing: HashSet<String> = HashSet::from_iter(orig_report.passing);
    let new_passing: HashSet<String> = HashSet::from_iter(new_report.passing);

    let passing_in_both = orig_passing.intersection(&new_passing);
    let failing_in_both = orig_failing.intersection(&new_failing);
    let passing_orig_failing_new: Vec<&String> =
        orig_passing.intersection(&new_failing).collect();
    let failure_orig_passing_new: Vec<&String> =
        orig_failing.intersection(&new_passing).collect();

    let mut comparison_report_file = File::create("cts-comparison-report.md").expect("File create");

    let num_orig_passing = orig_passing.len() as i32;
    let num_new_passing = new_passing.len() as i32;

    let num_orig_failing = orig_failing.len() as i32;
    let num_new_failing = new_failing.len() as i32;

    let total_orig = num_orig_passing + num_orig_failing as i32;
    let total_new = num_new_passing + num_new_failing as i32;

    let orig_passing = num_orig_passing as f32 / total_orig as f32 * 100.;
    let new_passing = num_new_passing as f32 / total_new as f32 * 100.;

    comparison_report_file
        .write_all(
            format!(
                "### Conformance comparison report
| | main ({}) | {} | +/- |
| --- | ---: | ---: | ---: |
| % Passing | {:.2}% | {:.2}% | {:.2}% |
| :white_check_mark: Passing | {} | {} | {} |
| :x: Failing | {} | {} | {} |
| Total Tests | {} | {} | {} |\n",
                &orig_report.commit_hash,
                &new_report.commit_hash,
                orig_passing,
                new_passing,
                new_passing - orig_passing,
                num_orig_passing,
                num_new_passing,
                num_new_passing - num_orig_passing,
                num_orig_failing,
                num_new_failing,
                num_new_failing - num_orig_failing,
                total_orig,
                total_new,
                total_new - total_orig
            )
            .as_bytes(),
        )
        .expect("write conformance report comparison table");

    comparison_report_file
        .write_all(
            format!(
                "\nNumber passing in both: {}\n
Number failing in both: {}\n
Number passing in main but now fail: {}\n
Number failing in main but now pass: {}
",
                passing_in_both.count(),
                failing_in_both.count(),
                passing_orig_failing_new.len(),
                failure_orig_passing_new.len()
            )
            .as_bytes(),
        )
        .expect("Failure when writing to file");

    if !passing_orig_failing_new.is_empty() {
        comparison_report_file.write_all(
            "\n:interrobang: CONFORMANCE REPORT REGRESSION DETECTED :interrobang:. The following test(s) were previously passing but now fail:\n".as_bytes()
        ).expect("write passing_orig_failing_new heading");
        for test_name in &passing_orig_failing_new {
            comparison_report_file
                .write_all(format!("- {}\n", test_name).as_bytes())
                .expect("write passing_orig_failing_new test case");
        }
    };

    if !failure_orig_passing_new.is_empty() {
        comparison_report_file.write_all(
            "\nThe following test(s) were previously failing but now pass. Before merging, confirm they are intended to pass: \n".as_bytes()
        ).expect("write failure_orig_passing_new heading");
        for test_name in &failure_orig_passing_new {
            comparison_report_file
                .write_all(format!("- {}\n", test_name).as_bytes())
                .expect("write failure_orig_passing_new test case");
        }
    }
}
