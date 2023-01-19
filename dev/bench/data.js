window.BENCHMARK_DATA = {
  "lastUpdate": 1674157407100,
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
      },
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
          "id": "589b0c87b3ff209925b02eeebffa728d8b446933",
          "message": "Cleanup some minor crate version mismatches (#278)",
          "timestamp": "2023-01-19T11:35:25-08:00",
          "tree_id": "dcdb5449c77d5167f98e51fc61ae29bfe2539de2",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/589b0c87b3ff209925b02eeebffa728d8b446933"
        },
        "date": 1674157405877,
        "tool": "cargo",
        "benches": [
          {
            "name": "join",
            "value": 21263,
            "range": "± 1507",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6278,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2686,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 165,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 1008,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3463,
            "range": "± 390",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 10529,
            "range": "± 784",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 28451,
            "range": "± 1329",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 44689,
            "range": "± 5813",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}