use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use std::process::exit;
use std::{env, fs};

#[derive(Serialize, Deserialize, Debug)]
struct CTSReport {
    commit_hash: String,
    passing: Vec<String>,
    failing: Vec<String>,
    ignored: Vec<String>,
}

/// Compares two conformance reports generated from [`generate_cts_report`], generating a comparison
/// report as markdown. The markdown report will contain the following:
/// - a table showing the number and percent of passing/failing tests in both reports
/// - number of tests passing in both reports
/// - number of tests failing in both reports
/// - number of tests passing in the first report but now fail in the second report (i.e. tests with
/// regressed behavior)
///   - also lists out these tests and gives a warning
/// - number of tests failing in the first report but now pass in the second report
///   - also lists out these tests
///
/// Requires the 3 following arguments
/// 1. path to first conformance report (will most commonly refer to the target branch's report)
/// 2. path to second conformance report
/// 3. path to output comparison report
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        println!("Requires passing in 3 arguments to `generate_comparison_report`. Usage:");
        println!("    generate_comparison_report <path to first conformance report> <path to second conformance report> <output comparison report path>");
        exit(1);
    }
    let orig_path = &args[1];
    let new_path = &args[2];
    let output_comparison_report_path = &args[3];

    let orig_results = fs::read_to_string(orig_path).expect("read to string orig_results");
    let new_results = fs::read_to_string(new_path).expect("read to string new_results");

    let orig_report: CTSReport = serde_json::from_str(&orig_results).expect("from_str");
    let new_report: CTSReport = serde_json::from_str(&new_results).expect("from_str");

    let orig_failing: HashSet<String> = HashSet::from_iter(orig_report.failing);
    let new_failing: HashSet<String> = HashSet::from_iter(new_report.failing);

    let orig_passing: HashSet<String> = HashSet::from_iter(orig_report.passing);
    let new_passing: HashSet<String> = HashSet::from_iter(new_report.passing);

    let orig_ignored: HashSet<String> = HashSet::from_iter(orig_report.ignored);
    let new_ignored: HashSet<String> = HashSet::from_iter(new_report.ignored);

    let passing_in_both = orig_passing.intersection(&new_passing);
    let failing_in_both = orig_failing.intersection(&new_failing);
    let passing_orig_failing_new: Vec<&String> = orig_passing.intersection(&new_failing).collect();
    let failure_orig_passing_new: Vec<&String> = orig_failing.intersection(&new_passing).collect();

    let mut comparison_report_file =
        File::create(output_comparison_report_path).expect("File create");

    let num_orig_passing = orig_passing.len() as i32;
    let num_new_passing = new_passing.len() as i32;

    let num_orig_failing = orig_failing.len() as i32;
    let num_new_failing = new_failing.len() as i32;

    let num_orig_ignored = orig_ignored.len() as i32;
    let num_new_ignored = new_ignored.len() as i32;

    let total_orig = num_orig_passing + num_orig_failing + num_orig_ignored;
    let total_new = num_new_passing + num_new_failing + num_new_ignored;

    let orig_passing = num_orig_passing as f32 / total_orig as f32 * 100.;
    let new_passing = num_new_passing as f32 / total_new as f32 * 100.;

    comparison_report_file
        .write_all(
            format!(
                "### Conformance comparison report
| | Base ({}) | {} | +/- |
| --- | ---: | ---: | ---: |
| % Passing | {:.2}% | {:.2}% | {:.2}% |
| :white_check_mark: Passing | {} | {} | {} |
| :x: Failing | {} | {} | {} |
| :large_orange_diamond: Ignored | {} | {} | {} |
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
                num_orig_ignored,
                num_new_ignored,
                num_new_ignored - num_orig_ignored,
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
Number passing in Base ({}) but now fail: {}\n
Number failing in Base ({}) but now pass: {}
",
                passing_in_both.count(),
                failing_in_both.count(),
                &orig_report.commit_hash,
                passing_orig_failing_new.len(),
                &orig_report.commit_hash,
                failure_orig_passing_new.len()
            )
            .as_bytes(),
        )
        .expect("Failure when writing to file");

    if !passing_orig_failing_new.is_empty() {
        comparison_report_file.write_all(
            "\n:interrobang: CONFORMANCE REPORT REGRESSION DETECTED :interrobang:. The following test(s) were previously passing but now fail:\n<details><summary>Click here to see</summary>\n\n".as_bytes()
        ).expect("write passing_orig_failing_new heading");
        for test_name in &passing_orig_failing_new {
            comparison_report_file
                .write_all(format!("- {test_name}\n").as_bytes())
                .expect("write passing_orig_failing_new test case");
        }
        comparison_report_file
            .write_all("\n</details>".as_bytes())
            .expect("write passing_orig_failing_new closing");
    };

    if !failure_orig_passing_new.is_empty() {
        comparison_report_file.write_all(
            "\nThe following test(s) were previously failing but now pass. Before merging, confirm they are intended to pass: \n<details><summary>Click here to see</summary>\n\n".as_bytes()
        ).expect("write failure_orig_passing_new heading");
        for test_name in &failure_orig_passing_new {
            comparison_report_file
                .write_all(format!("- {test_name}\n").as_bytes())
                .expect("write failure_orig_passing_new test case");
        }
        comparison_report_file
            .write_all("\n</details>".as_bytes())
            .expect("write failure_orig_passing_new closing");
    }
}
