window.BENCHMARK_DATA = {
  "lastUpdate": 1674283793763,
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
          "id": "be1070623559bc8dcdb533c016035d9252feb34b",
          "message": "Add benchmarks to parse,compile,plan,eval for 1,15,30 `OR`ed `LIKE`s (#276)",
          "timestamp": "2023-01-19T13:41:33-08:00",
          "tree_id": "221f88a1cdf9dd71783550f40f43161bcad2bf17",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/be1070623559bc8dcdb533c016035d9252feb34b"
        },
        "date": 1674165288035,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7407,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 73077,
            "range": "± 3893",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 145112,
            "range": "± 5281",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 15226,
            "range": "± 686",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 53849,
            "range": "± 2852",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 102976,
            "range": "± 3245",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 25135,
            "range": "± 1769",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 479950,
            "range": "± 18097",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 964258,
            "range": "± 25826",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 33993649,
            "range": "± 1611487",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 177620375,
            "range": "± 3499726",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 335970979,
            "range": "± 6653037",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 20745,
            "range": "± 1137",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6194,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2566,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 164,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 1000,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3441,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 11156,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 28588,
            "range": "± 1205",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 47509,
            "range": "± 2178",
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
          "id": "5928084b07924bf498a0482e800f519b183c8b28",
          "message": "Replace usages of lazy_static with once_cell (#279)",
          "timestamp": "2023-01-20T22:38:29-08:00",
          "tree_id": "eb0676db4c5b1c22c29e19e3c4b9838860dc1710",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/5928084b07924bf498a0482e800f519b183c8b28"
        },
        "date": 1674283793191,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6361,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 61133,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 118813,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 12895,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 44900,
            "range": "± 761",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 82880,
            "range": "± 842",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19212,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 366776,
            "range": "± 3180",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 735761,
            "range": "± 7440",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 28622532,
            "range": "± 1498533",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 142692525,
            "range": "± 305557",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 265532849,
            "range": "± 745644",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17197,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5192,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2180,
            "range": "± 1",
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
            "value": 857,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2893,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8701,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23143,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 36965,
            "range": "± 89",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}