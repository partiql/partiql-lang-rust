# Parser Fuzz Tests

***Note***: This crate is for testing only and is *not* published.

## Setup

You'll need [cargo-fuzz][fuzz], which requires nightly rust:

```shell
rustup toolchain install nightly
cargo install cargo-fuzz
```

If you want to see code coverage of the fuzz tests, you'll also need:

```shell
rustup component add --toolchain nightly llvm-tools-preview rust-src
```

See also the cargo-fuzz [setup instructions][setup].

[fuzz]: https://github.com/rust-fuzz/cargo-fuzz
[setup]: https://rust-fuzz.github.io/book/cargo-fuzz/setup.html

## Fuzz Testing

First, seed the fuzzing corpus with some PartiQL queries:

```shell
mkdir -p fuzz/corpus/fuzz_parse_string
cp -r fuzz/inputs/seed/* fuzz/corpus/fuzz_parse_string 
```

Then, you can run the fuzz test (change the number of jobs for your desired parallelism; a good rule of thumb is either
1x or 2x your number of CPU cores):

```shell
cargo +nightly fuzz run --jobs 4 fuzz_parse_string -- -dict=fuzz/inputs/keywords
 ```

The fuzzing will continue either until you kill the process or until a crash is found.

If you want to limit the fuzzer run time to a target number of seconds, add `-max_total_time <seconds>` like:
```shell
cargo +nightly fuzz run --jobs 4 fuzz_parse_string -- -dict=fuzz/inputs/keywords -max_total_time 30
 ```



### Finding a Crash

### Fuzz Code Coverage

```shell
rustup component add --toolchain nightly llvm-tools-preview rust-src
```

Generate coverage data:

```shell
cargo fuzz coverage fuzz_parse_string
```

```shell
~/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/bin/llvm-cov show fuzz/target/x86_64-apple-darwin/release/fuzz_parse_string -format=html -instr-profile=fuzz/coverage/fuzz_parse_string/coverage.profdata -name-regex "partiql.*" > index.html
```