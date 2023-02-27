window.BENCHMARK_DATA = {
  "lastUpdate": 1677537830375,
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
          "id": "d49566e29b38694276b79ca51470ad40c3e89df4",
          "message": "Add cargo-deny config and CI action to enforce it (#281)",
          "timestamp": "2023-01-23T09:19:01-08:00",
          "tree_id": "7ce5f78937283cd2190a610044a5321226ece521",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/d49566e29b38694276b79ca51470ad40c3e89df4"
        },
        "date": 1674495013972,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6491,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 62720,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 122095,
            "range": "± 714",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 12740,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 44094,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 81839,
            "range": "± 373",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19763,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 374307,
            "range": "± 680",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 755815,
            "range": "± 1215",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27077498,
            "range": "± 153033",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 145268492,
            "range": "± 3478600",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 276123093,
            "range": "± 519471",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17292,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5398,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2367,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 107,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 863,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3092,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9339,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23771,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 37284,
            "range": "± 98",
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
          "id": "670fd2598eeb95e2ef179e2516bf092cdd13b63b",
          "message": "Add built-in functions: position, ocetet_len, bit_len (#282)\n\n- Add implementations\r\n- Update partiql-tests",
          "timestamp": "2023-01-23T11:30:17-08:00",
          "tree_id": "0a648e9e514b371d60d816294ef5195d9a0bba0a",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/670fd2598eeb95e2ef179e2516bf092cdd13b63b"
        },
        "date": 1674502891898,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6593,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 63442,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 122282,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 14992,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 45745,
            "range": "± 525",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 83207,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20021,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 375989,
            "range": "± 816",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 756527,
            "range": "± 1246",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27890001,
            "range": "± 213966",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 145696852,
            "range": "± 815158",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 277123483,
            "range": "± 720546",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17343,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5494,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2372,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 107,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 864,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3045,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9224,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23717,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 37433,
            "range": "± 122",
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
          "id": "4a49f5beb58aced185975dd02e114fb239294245",
          "message": "Add config for cargo-about (#283)",
          "timestamp": "2023-01-23T13:14:55-08:00",
          "tree_id": "ba941e84b4c3a44f5ba8ce5aa9c7597af527c122",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/4a49f5beb58aced185975dd02e114fb239294245"
        },
        "date": 1674509179168,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6187,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 60790,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 118577,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 15061,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 46023,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 83460,
            "range": "± 539",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19233,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 366612,
            "range": "± 506",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 739457,
            "range": "± 813",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 29964622,
            "range": "± 793225",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 144126034,
            "range": "± 365150",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 268415999,
            "range": "± 674236",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17361,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5235,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2240,
            "range": "± 2",
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
            "value": 829,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2890,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8733,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23032,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 37147,
            "range": "± 101",
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
          "id": "50d2c202bbc28a6838cea054c4b82645e2a4a23d",
          "message": "Improve performance by removing extraneous `clone`s and tweaking initial buffer sizes. (#284)\n\n* Remove some extraneous `clone`s.\r\n* Increate initial parse node location buffer.",
          "timestamp": "2023-01-24T11:35:43-08:00",
          "tree_id": "95917e0508c9beaf3cc2b478c7a6f11c3b62b8f8",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/50d2c202bbc28a6838cea054c4b82645e2a4a23d"
        },
        "date": 1674589704114,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7520,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 71016,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 140093,
            "range": "± 708",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 17775,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 54699,
            "range": "± 569",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 99245,
            "range": "± 664",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 22876,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 425952,
            "range": "± 2341",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 850544,
            "range": "± 6573",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27221617,
            "range": "± 338963",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 163337283,
            "range": "± 1653688",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 310879992,
            "range": "± 2217285",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 19576,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6213,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2602,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 171,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 1009,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3477,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 10636,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 27570,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 43162,
            "range": "± 239",
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
          "id": "55a62fe3fbbeb07e222eea6233c1fe95400de708",
          "message": "Update CI actions (#285)\n\n* Upgrade actions/checkout: v2 -> v3\r\n* Upgrade actions/cache: v2 -> v3\r\n* Update `Create or update comment` step to use `body-file`\r\n* Switch to dtolnay/rust-toolchain\r\n* Use stable for benchmarking",
          "timestamp": "2023-01-24T17:55:56-08:00",
          "tree_id": "5ea1d5fba064643d428da55c529804a57fc86431",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/55a62fe3fbbeb07e222eea6233c1fe95400de708"
        },
        "date": 1674612411955,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5531,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 53594,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 104761,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 15858,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 47938,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 85851,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 18599,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 349639,
            "range": "± 1107",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 704069,
            "range": "± 1553",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23604013,
            "range": "± 1115758",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 140285113,
            "range": "± 731586",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 267115442,
            "range": "± 500746",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16240,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5201,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2153,
            "range": "± 0",
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
            "value": 784,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2458,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7718,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19730,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 30927,
            "range": "± 51",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "caialan@amazon.com",
            "name": "Alan Cai",
            "username": "alancai98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4253c37a6ab0193b534d14a41288fda2f14e7b89",
          "message": "Implement `PIVOT` in evaluator (#286)",
          "timestamp": "2023-01-24T18:33:52-08:00",
          "tree_id": "dfe27649b0fa57cf7125acd8bfd21351d03c491f",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/4253c37a6ab0193b534d14a41288fda2f14e7b89"
        },
        "date": 1674614802380,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6597,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 63682,
            "range": "± 2124",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 128146,
            "range": "± 4522",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 19492,
            "range": "± 1633",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 60252,
            "range": "± 2109",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 111357,
            "range": "± 7860",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 24657,
            "range": "± 3185",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 467259,
            "range": "± 13933",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 937933,
            "range": "± 25138",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 28834045,
            "range": "± 933755",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 177208151,
            "range": "± 5033799",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 340452591,
            "range": "± 7701219",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 19777,
            "range": "± 886",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6328,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2597,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 166,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 952,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2945,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9339,
            "range": "± 593",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 24931,
            "range": "± 1269",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 39660,
            "range": "± 1597",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "caialan@amazon.com",
            "name": "Alan Cai",
            "username": "alancai98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "747f41abab84951e427c889739a72b6c53b8fb19",
          "message": "Fix clippy warnings following GH Actions Rust version bump (#288)",
          "timestamp": "2023-01-27T13:14:21-08:00",
          "tree_id": "6ab331717f1a233d145813a71f27299bc501683d",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/747f41abab84951e427c889739a72b6c53b8fb19"
        },
        "date": 1674854787362,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6049,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 59600,
            "range": "± 988",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 119262,
            "range": "± 1917",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 18222,
            "range": "± 319",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 56078,
            "range": "± 694",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 98857,
            "range": "± 1240",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 22389,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 418776,
            "range": "± 5226",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 846015,
            "range": "± 10631",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27242423,
            "range": "± 611058",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 159795753,
            "range": "± 1692378",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 307317650,
            "range": "± 2770306",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 19049,
            "range": "± 302",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6137,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2480,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 169,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 758,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2574,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8734,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22782,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 36073,
            "range": "± 530",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "caialan@amazon.com",
            "name": "Alan Cai",
            "username": "alancai98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e36e4e13dfd9f09c05e108ffa506db2ff6ddb19f",
          "message": "Implement `LIKE` for non-string, non-literals (#287)\n\nCo-authored-by: Josh Pschorr <josh@pschorr.dev>",
          "timestamp": "2023-01-30T17:05:21-08:00",
          "tree_id": "37889d69c5a76e6c1ea4cd1c009e7edcf7a9ee4e",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/e36e4e13dfd9f09c05e108ffa506db2ff6ddb19f"
        },
        "date": 1675127769090,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5165,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50698,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 101354,
            "range": "± 457",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 15575,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 47444,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 87859,
            "range": "± 915",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 18967,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 354022,
            "range": "± 1246",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 715375,
            "range": "± 1002",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22713997,
            "range": "± 313934",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 137614593,
            "range": "± 579053",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 262717266,
            "range": "± 453205",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16237,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5159,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2088,
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
            "value": 677,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2201,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7209,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19136,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 30144,
            "range": "± 69",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "27716912+am357@users.noreply.github.com",
            "name": "Arash Maymandi",
            "username": "am357"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5cbe79e0cda989cb1dd3271c2f9072bd956e0cb1",
          "message": "Add Serde json to `partiql-value` (#289)\n\nAdds the serde-json's Serialize and Deseralize to partiql_value::Value as a feature.",
          "timestamp": "2023-02-01T15:23:12-08:00",
          "tree_id": "161e7d607188084650c0263a7a7aee834f903278",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/5cbe79e0cda989cb1dd3271c2f9072bd956e0cb1"
        },
        "date": 1675294442335,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5477,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 52421,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 105743,
            "range": "± 481",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 16310,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 49141,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 89277,
            "range": "± 794",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19557,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 360397,
            "range": "± 933",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 729454,
            "range": "± 1774",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22942070,
            "range": "± 165102",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 141281069,
            "range": "± 488177",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 275202804,
            "range": "± 397784",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16433,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5373,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2224,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 107,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 657,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2242,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7588,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 18919,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29935,
            "range": "± 124",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "27716912+am357@users.noreply.github.com",
            "name": "Arash Maymandi",
            "username": "am357"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "48b6d896c889c1e547f387e5ea85565bc3f9237c",
          "message": "Add serde to partiql-logical (#290)\n\nAdds serde w/ Serialize and Deserialize traits to partiql-logical.",
          "timestamp": "2023-02-01T16:29:55-08:00",
          "tree_id": "b89dc94acee56f52c37157f600faa4efcd0ea112",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/48b6d896c889c1e547f387e5ea85565bc3f9237c"
        },
        "date": 1675298563546,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6330,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 62053,
            "range": "± 5035",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 121487,
            "range": "± 8726",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 17812,
            "range": "± 1318",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 57873,
            "range": "± 4431",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 108388,
            "range": "± 7139",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 24265,
            "range": "± 2063",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 465518,
            "range": "± 24073",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 939163,
            "range": "± 49309",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27399860,
            "range": "± 1613212",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 161789612,
            "range": "± 7817515",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 307445863,
            "range": "± 9584027",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 18769,
            "range": "± 1248",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5789,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2424,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 160,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 754,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2627,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8862,
            "range": "± 565",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 24433,
            "range": "± 1859",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 37685,
            "range": "± 1941",
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
          "id": "5f349a5bf16927c8315495c59ec040d1547eaa02",
          "message": "Change EvalExpr.evalute to return `Cow`s to avoid unnecessary clones (#291)\n\nRefactors `EvalExpr`'s `evaluate` function to return a [`Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html#).\r\n\r\nExpressions like paths and variable reference return a `Cow::Borrowed` value and value-generating expressions (e.g., `+`, `trim`, ||`, etc) return a `Cow::Owned` value.\r\n\r\nLocally, this results in a ~15% to ~30% speedup on evaluation benchmarks and ~40% speedup of the run of the entire conformance test suite.",
          "timestamp": "2023-02-07T15:13:03-08:00",
          "tree_id": "2725003435ba116f6f506ea6f6981f32b9596677",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/5f349a5bf16927c8315495c59ec040d1547eaa02"
        },
        "date": 1675812232818,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5457,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51359,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 105040,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 16543,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 49325,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 88243,
            "range": "± 521",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19224,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 364635,
            "range": "± 1051",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 735597,
            "range": "± 3682",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21515084,
            "range": "± 150198",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 125042001,
            "range": "± 508153",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 244972393,
            "range": "± 506925",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 13960,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5409,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2313,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 683,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2304,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7558,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19894,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 30960,
            "range": "± 110",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "caialan@amazon.com",
            "name": "Alan Cai",
            "username": "alancai98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9e2dbf79821a97f6258d61e9f8b8983e8a8a59ea",
          "message": "Update GitHub Actions conformance comment when target branch conformance report unavailable (#292)",
          "timestamp": "2023-02-08T11:07:31-08:00",
          "tree_id": "4a447eb9830b01d7195406d9d3a9b69cc2b58f4c",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/9e2dbf79821a97f6258d61e9f8b8983e8a8a59ea"
        },
        "date": 1675883918390,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5363,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51148,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 101470,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 15242,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 47697,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 86670,
            "range": "± 951",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 18838,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 355005,
            "range": "± 982",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 717449,
            "range": "± 1096",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23042300,
            "range": "± 437448",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 120855751,
            "range": "± 596722",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 228879143,
            "range": "± 378923",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 13935,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5066,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2121,
            "range": "± 2",
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
            "value": 676,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2309,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8068,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21180,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 32585,
            "range": "± 2141",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "27716912+am357@users.noreply.github.com",
            "name": "Arash Maymandi",
            "username": "am357"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7e543111c6954f92ae1221e108a0e1a6e0142808",
          "message": "Add `serde` and `Display` for `partiql-logical` and `serde` for `partiql-value` (#293)\n\n1- Adds the remaining serde features.\r\n2- Adds an initial Display for LogicalPlan.",
          "timestamp": "2023-02-08T15:33:42-08:00",
          "tree_id": "6db7f5fb96a36d09e1ae87e04375ddba036c3e27",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/7e543111c6954f92ae1221e108a0e1a6e0142808"
        },
        "date": 1675899878665,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5622,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 53084,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 110263,
            "range": "± 394",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 16510,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 48665,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 88964,
            "range": "± 443",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19477,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 362655,
            "range": "± 857",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 731293,
            "range": "± 4368",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21806108,
            "range": "± 211601",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 124854202,
            "range": "± 532431",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 244342899,
            "range": "± 416269",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 13945,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5330,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2296,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 643,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2238,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7625,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19325,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29701,
            "range": "± 67",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "27716912+am357@users.noreply.github.com",
            "name": "Arash Maymandi",
            "username": "am357"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b2ce0ce739e69380e821c94936b7f79f60ed043c",
          "message": "Make partiql_value::parse_ion  public (#294)",
          "timestamp": "2023-02-09T13:31:22-08:00",
          "tree_id": "9dc30f6512be35e535d4f436131781f7a7929406",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/b2ce0ce739e69380e821c94936b7f79f60ed043c"
        },
        "date": 1675978936885,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5606,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 52413,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 103485,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 16517,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 48863,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 89172,
            "range": "± 485",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19254,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 364078,
            "range": "± 674",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 734342,
            "range": "± 1627",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21462446,
            "range": "± 158943",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 125592744,
            "range": "± 494778",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 245551563,
            "range": "± 984010",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14033,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5373,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2297,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 107,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 646,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2237,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7663,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19188,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29990,
            "range": "± 106",
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
          "id": "c63109e95765f3a4f98844bafd23b08fcb15c7fd",
          "message": "Fix off by one error when checking preconditions to lower join `ON` (#295)",
          "timestamp": "2023-02-10T11:43:41-08:00",
          "tree_id": "2ed0aaca0794f83047e10ecad6d1dd5511484d68",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/c63109e95765f3a4f98844bafd23b08fcb15c7fd"
        },
        "date": 1676058882717,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5451,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 52686,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 104196,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 16457,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 49285,
            "range": "± 426",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 89639,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19392,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 362807,
            "range": "± 908",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 732695,
            "range": "± 1789",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21616170,
            "range": "± 178405",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 125023271,
            "range": "± 538132",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 244749685,
            "range": "± 387919",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 13960,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5413,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2355,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 107,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 648,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2255,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7594,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19916,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 30015,
            "range": "± 130",
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
          "id": "f3d0e5dc9d2b5d487c1e93f3e3e2110373888744",
          "message": "Add some convenience methods and cleanup of `Value` (#297)\n\n- Add `Extend` implementations for `List` and `Bag`\r\n- Add methods to iterate a `Tuple`'s values without zipping its names\r\n- Allow `collect()` into a `Tuple` with any `Into<String>`\r\n- other minor cleanup",
          "timestamp": "2023-02-10T19:31:04-08:00",
          "tree_id": "79d8b9abffa49c2cdcdbfeea4f6d3af326e7e1d7",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/f3d0e5dc9d2b5d487c1e93f3e3e2110373888744"
        },
        "date": 1676086985674,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6334,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 63677,
            "range": "± 2146",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 126557,
            "range": "± 4724",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 17390,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 54365,
            "range": "± 1706",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 95625,
            "range": "± 3817",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 21212,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 400173,
            "range": "± 16073",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 795484,
            "range": "± 25894",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 24628028,
            "range": "± 918434",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 136629795,
            "range": "± 3772909",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 258180147,
            "range": "± 6508248",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15747,
            "range": "± 553",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5776,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2384,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 162,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 747,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2510,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8173,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21638,
            "range": "± 892",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34846,
            "range": "± 1196",
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
          "id": "0e87eeed3005707ea5b142ba59facd572f8301f2",
          "message": "Cache `Evaluable`'s \"attrs\" at construct time, and return by ref (#298)",
          "timestamp": "2023-02-10T19:53:25-08:00",
          "tree_id": "331973390114d34e476505880b3e2cc0faac9f66",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/0e87eeed3005707ea5b142ba59facd572f8301f2"
        },
        "date": 1676088272659,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5182,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50886,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 100462,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 15290,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 47727,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 86803,
            "range": "± 658",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19063,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 354320,
            "range": "± 1283",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 716587,
            "range": "± 947",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22603898,
            "range": "± 455430",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121970589,
            "range": "± 657090",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 232140951,
            "range": "± 565967",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14151,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5155,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2235,
            "range": "± 4",
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
            "value": 638,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2192,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7334,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19069,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 30783,
            "range": "± 55",
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
          "id": "1d77b27c5fbd25bb010b9f3cc1ec80df71a1de86",
          "message": "Cleanup evaluator to more idiomatically use `.collect()` throughout (#299)",
          "timestamp": "2023-02-10T20:37:57-08:00",
          "tree_id": "cfce773b1bb09c6bdc39acc1a84373fe48e0caed",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/1d77b27c5fbd25bb010b9f3cc1ec80df71a1de86"
        },
        "date": 1676090949414,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5372,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51750,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 103832,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 15453,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 47375,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 88581,
            "range": "± 381",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19211,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 355366,
            "range": "± 775",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 713149,
            "range": "± 1209",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22221794,
            "range": "± 454867",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121174596,
            "range": "± 579371",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 231127430,
            "range": "± 363060",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14326,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5158,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2198,
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
            "value": 640,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2192,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7353,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19242,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 30604,
            "range": "± 52",
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
          "id": "7a6d7ad28a62436f30103510570e4922fb65cfd7",
          "message": "Refactor evaluator structure; no behavior changes (#300)\n\nBehavior-preserving refactor of the evaluator to bring some structure to the code, especially splitting `EvalExpr`s from `Evaluable`s.",
          "timestamp": "2023-02-15T10:53:45-08:00",
          "tree_id": "b553690cc4df39ad8cad3c898eb689dad67d36f2",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/7a6d7ad28a62436f30103510570e4922fb65cfd7"
        },
        "date": 1676487914753,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5196,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51048,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 101356,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 15449,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 47671,
            "range": "± 669",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 86333,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 18950,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 354963,
            "range": "± 715",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 713899,
            "range": "± 1265",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23691335,
            "range": "± 852018",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 119477283,
            "range": "± 588499",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 227195024,
            "range": "± 606511",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14254,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5144,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2175,
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
            "value": 699,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2501,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8267,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21594,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 33512,
            "range": "± 69",
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
          "id": "320c6297315465f6a97297b6d356a55bbadc4894",
          "message": "Recognize aggregate fn names in parser (#302)",
          "timestamp": "2023-02-16T10:19:34-08:00",
          "tree_id": "e4b593ae196df01ea8f71a444a135fcb7b939431",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/320c6297315465f6a97297b6d356a55bbadc4894"
        },
        "date": 1676572252334,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 115178,
            "range": "± 1746",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 180173,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 235896,
            "range": "± 2125",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 16294,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 49355,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 89549,
            "range": "± 403",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19261,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 360981,
            "range": "± 970",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 728004,
            "range": "± 1777",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22263170,
            "range": "± 190990",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121810519,
            "range": "± 378248",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 238475214,
            "range": "± 417895",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14324,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5780,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2631,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 99531,
            "range": "± 1077",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 114249,
            "range": "± 853",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 124391,
            "range": "± 2677",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 145777,
            "range": "± 3247",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 159555,
            "range": "± 1297",
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
          "id": "92c6b40119f3e8c5ebb91d86f5762fbe6f8aa390",
          "message": "Build Aggregate function recognizer only once, using OnceCell (#304)",
          "timestamp": "2023-02-16T15:16:34-08:00",
          "tree_id": "ff521ddb7e59e4eaaba3e10f360660eed0d8c85b",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/92c6b40119f3e8c5ebb91d86f5762fbe6f8aa390"
        },
        "date": 1676590061310,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5740,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 54738,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 111581,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 16473,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 48635,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 87867,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19355,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 364076,
            "range": "± 898",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 735590,
            "range": "± 1497",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21414758,
            "range": "± 226933",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121753098,
            "range": "± 386673",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239066377,
            "range": "± 574287",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14491,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5467,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2339,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 643,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2250,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7753,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19826,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 30056,
            "range": "± 108",
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
          "id": "40158c274ea3a2ec9f1425c12f85c4c461be651a",
          "message": "Pass-through comments when preprocessing special forms (#305)",
          "timestamp": "2023-02-17T10:54:56-08:00",
          "tree_id": "4de9b9b2255ae424e29222785774b9d858681d79",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/40158c274ea3a2ec9f1425c12f85c4c461be651a"
        },
        "date": 1676660764897,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5509,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 52423,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 109029,
            "range": "± 196",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 15462,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 47588,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 87176,
            "range": "± 897",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 18956,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 356617,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 716681,
            "range": "± 1280",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22275400,
            "range": "± 429339",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 118498598,
            "range": "± 549760",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 225742032,
            "range": "± 464029",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14106,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5142,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2128,
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
            "value": 604,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2224,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7438,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19762,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 30884,
            "range": "± 79",
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
          "id": "a6586f0a6e265e949229efb6517b20d7e69f8ca0",
          "message": "Fix `JOIN` parsing: Default to `INNER`; Allow ellision of keywords. (#308)",
          "timestamp": "2023-02-17T10:55:21-08:00",
          "tree_id": "bb00d142111fb36b407202ae1a5e7628bac34ecc",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/a6586f0a6e265e949229efb6517b20d7e69f8ca0"
        },
        "date": 1676660811634,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5734,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 54907,
            "range": "± 474",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 111565,
            "range": "± 681",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 16603,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 48233,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 86543,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19490,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 363808,
            "range": "± 946",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 732220,
            "range": "± 1248",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22108124,
            "range": "± 226403",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122272206,
            "range": "± 337532",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239301296,
            "range": "± 441572",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14484,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5611,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2403,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 627,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2327,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7700,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20044,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 30010,
            "range": "± 104",
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
          "id": "cda2b17991f3a13ee711eec1ae401f767932554b",
          "message": "Make `BY <x>` optional in `GROUP` clause (#307)",
          "timestamp": "2023-02-17T10:55:12-08:00",
          "tree_id": "8e64fef1ad74fdfbb4df793f3b6de56e3d8bd2c6",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/cda2b17991f3a13ee711eec1ae401f767932554b"
        },
        "date": 1676660859010,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7047,
            "range": "± 409",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 58414,
            "range": "± 3023",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 126104,
            "range": "± 6315",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 18251,
            "range": "± 1481",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 57388,
            "range": "± 3590",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 104128,
            "range": "± 7603",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 23172,
            "range": "± 1052",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 434977,
            "range": "± 24671",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 871779,
            "range": "± 43897",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 25391360,
            "range": "± 1532492",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 142824624,
            "range": "± 5751448",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 275272777,
            "range": "± 8177995",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16406,
            "range": "± 951",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5896,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2555,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 158,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 690,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2514,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9021,
            "range": "± 531",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 24575,
            "range": "± 1064",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 37904,
            "range": "± 2769",
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
          "id": "70d14b36d373df482e80517fcd37814afb585b0e",
          "message": "Allow un-parenthesized subquery as the only argument of a function (#309)",
          "timestamp": "2023-02-17T10:55:44-08:00",
          "tree_id": "d0d8ddb4c6ae314991538d8aab35a6604794c080",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/70d14b36d373df482e80517fcd37814afb585b0e"
        },
        "date": 1676660951706,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7135,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 60891,
            "range": "± 3974",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 134387,
            "range": "± 10344",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 18259,
            "range": "± 1319",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 59223,
            "range": "± 3802",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 109885,
            "range": "± 28621",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 25338,
            "range": "± 1900",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 472705,
            "range": "± 20918",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 956261,
            "range": "± 43127",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27781599,
            "range": "± 1286130",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 151821027,
            "range": "± 6761203",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 299975376,
            "range": "± 7303073",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17568,
            "range": "± 847",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6229,
            "range": "± 538",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2727,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 167,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 739,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2638,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9081,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 25222,
            "range": "± 3244",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 38055,
            "range": "± 1993",
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
          "id": "e340635fe8919a19147e7d0fc05169a267ed7087",
          "message": "Fix handling of List/Bag/Tuple in keyword argument preprocessing (#311)",
          "timestamp": "2023-02-17T13:33:54-08:00",
          "tree_id": "7ef7281f73626eb648868f710abcb7dfc9aa0e0d",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/e340635fe8919a19147e7d0fc05169a267ed7087"
        },
        "date": 1676670330663,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5767,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 53007,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 110300,
            "range": "± 494",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 16307,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 48541,
            "range": "± 768",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 89736,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19484,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 360722,
            "range": "± 859",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 728033,
            "range": "± 2170",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21802717,
            "range": "± 190111",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122524347,
            "range": "± 1000842",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 240048273,
            "range": "± 386662",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14291,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5536,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2413,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 634,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2180,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7573,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19582,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 30173,
            "range": "± 199",
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
          "id": "8fb0fddbd7df88dfadecd6e7dfb1f175013e526a",
          "message": "Parse `OUTER` `UNION`/`INTERSECT`/`EXCEPT` (#306)",
          "timestamp": "2023-02-17T17:18:28-08:00",
          "tree_id": "893fca6d67608aea8f2ec117c56d007d43dbd6f6",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/8fb0fddbd7df88dfadecd6e7dfb1f175013e526a"
        },
        "date": 1676683799436,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5799,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 53742,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 107850,
            "range": "± 299",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 16367,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 48355,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 89000,
            "range": "± 424",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19398,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 358911,
            "range": "± 665",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 723400,
            "range": "± 2251",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21462905,
            "range": "± 189138",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121389861,
            "range": "± 412932",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239103910,
            "range": "± 392366",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14516,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5449,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2284,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 602,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2156,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7712,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19647,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29771,
            "range": "± 110",
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
          "id": "57dc203ebd7bc417808bf49f4e256ec782fefdb4",
          "message": "Add parsing of `WITH` clause (`SEARCH` & `CYCLE` still TODO) (#310)",
          "timestamp": "2023-02-27T14:32:36-08:00",
          "tree_id": "9605674ebb3b05eda01d8ae5ef361e72d6ee6054",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/57dc203ebd7bc417808bf49f4e256ec782fefdb4"
        },
        "date": 1677537829802,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6306,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 66119,
            "range": "± 602",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 131655,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 16441,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 49157,
            "range": "± 336",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 88785,
            "range": "± 580",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19024,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 359794,
            "range": "± 926",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 728211,
            "range": "± 1813",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21091173,
            "range": "± 92994",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121635138,
            "range": "± 388243",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239355713,
            "range": "± 368154",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14190,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5462,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2283,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 668,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2811,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8912,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22760,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 35136,
            "range": "± 123",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}