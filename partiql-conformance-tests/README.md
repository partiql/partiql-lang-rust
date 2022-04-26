## Running the Tests

Running `cargo test` from the `partiql-lang-rust` root will not run the conformance tests by default.

To run the tests, you will need to run `cargo test` with the "conformance_test" `--features` flag:

```shell
cargo test --package partiql-conformance-tests --features "conformance_test"
```

---

Running an individual test (or subset of tests) can vary by the IDE being used. Using CLion, you may need to first edit
the test run configuration and enable the "Use all features in test" checkbox or explicitly add the 
`--features "conformance_test"` test option.

Using the command line, you can run an individual test with the following:
```shell
cargo test --package partiql-conformance-tests --test <test name or full mod path> --features "conformance_test" -- --exact
```
