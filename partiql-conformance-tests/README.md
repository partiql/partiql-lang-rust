## Running the Tests

Running `cargo test` from the `partiql-lang-rust` root will not run the conformance tests by default.

To run all the tests (including conformance tests), you will need to run `cargo test` with the "conformance_test" `--features` flag:

```shell
cargo test --features "conformance_test"
```

Or to run just the conformance tests:

```shell
cargo test --package partiql-conformance-tests --features "conformance_test"
```

---

Conformance tests are generated from the [PartiQL Test Data](partiql-tests/README.md).

### Default Tests
The default tests can be run with:
```shell
cargo test --package partiql-conformance-tests --features "conformance_test" 
```

Which is equivalent to
```shell
cargo test --package partiql-conformance-tests --no-default-features --features "base,conformance_test" 
```

### Test Categories
It is also possible to run subsets of the tests. See the set of test categories in [Cargo.toml](Cargo.toml)

To run only `semantic` analysis tests:
```shell
cargo test --package partiql-conformance-tests --no-default-features --features "semantic,conformance_test" 
```


To run only `strict` tests:
```shell
cargo test --package partiql-conformance-tests --no-default-features --features "strict,conformance_test" 
```

To run `experimental` tests in addition to all default tests:
```shell
cargo test --package partiql-conformance-tests --features "experimental, conformance_test" 
```


### Individual Tests

Running an individual test (or subset of tests) can vary by the IDE being used. Using CLion, you may need to first edit
the test run configuration and enable the "Use all features in test" checkbox or explicitly add the 
`--features "conformance_test"` test option.

Using the command line, you can run an individual test with the following:
```shell
cargo test --package partiql-conformance-tests --test <test name or full mod path> --features "conformance_test" -- --exact
```
