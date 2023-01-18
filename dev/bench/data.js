window.BENCHMARK_DATA = {
  "lastUpdate": 1674083668395,
  "repoUrl": "https://github.com/partiql/partiql-lang-rust",
  "entries": {
    "PartiQL (rust) Benchmark": [
      {
        "commit": {
          "author": {
            "email": "josh@pschorr.dev",
            "name": "Josh Pschorr",
            "username": "jpschorr"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8373662da60a83d1da11870858774831ba1059df",
          "message": "Fix Benchmark GitHub Action to work with Criterion benchmarking (#277)\n\n - Give workflow a name\r\n - Added `bench=false` to `lib`s and `bin`s as per the [criterion faq](https://bheisler.github.io/criterion.rs/book/faq.html#cargo-bench-gives-unrecognized-option-errors-for-valid-command-line-options)",
          "timestamp": "2023-01-18T15:07:47-08:00",
          "tree_id": "365eaf805856675021c2eef370fe5ba14b1d8571",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/8373662da60a83d1da11870858774831ba1059df"
        },
        "date": 1674083667892,
        "tool": "cargo",
        "benches": [
          {
            "name": "join",
            "value": 17392,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5189,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2217,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 144,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 838,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2881,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8705,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23343,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 37075,
            "range": "± 71",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}