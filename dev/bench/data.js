window.BENCHMARK_DATA = {
  "lastUpdate": 1686084238597,
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
          "id": "ec93ff263b0714068e13cd6b445d39c3cf2da611",
          "message": "Add `ABS`, `MOD`, `CARDINALITY`, and `OVERLAY` built-ins (#312)",
          "timestamp": "2023-03-01T09:34:31-08:00",
          "tree_id": "049a88f6608901abe24fdf57dcc20b07ba7cb102",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/ec93ff263b0714068e13cd6b445d39c3cf2da611"
        },
        "date": 1677692881642,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7590,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 79999,
            "range": "± 3587",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 157155,
            "range": "± 7762",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 30193,
            "range": "± 1268",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 74004,
            "range": "± 2770",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 126985,
            "range": "± 5492",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 25231,
            "range": "± 1597",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 471432,
            "range": "± 17706",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 947483,
            "range": "± 41481",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 25574002,
            "range": "± 2143942",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 136180293,
            "range": "± 5672138",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 277514437,
            "range": "± 12278075",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 18080,
            "range": "± 529",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6304,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2725,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 172,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 972,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3876,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 12386,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 32444,
            "range": "± 982",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 49838,
            "range": "± 1618",
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
          "id": "0eb3ef07e0bb453afd2efbee50ddcf476f06ad9b",
          "message": "Update to latest `partiql-tests` (#313)",
          "timestamp": "2023-03-09T14:31:14-08:00",
          "tree_id": "f9d3e94c77152cd0c524e27d0d367b1a7aa3feb6",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/0eb3ef07e0bb453afd2efbee50ddcf476f06ad9b"
        },
        "date": 1678401720643,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5898,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 58485,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 115400,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 24717,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 57900,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 100564,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 18896,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 353416,
            "range": "± 658",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 712437,
            "range": "± 1291",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 24616151,
            "range": "± 548542",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 119182162,
            "range": "± 454209",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 226882267,
            "range": "± 469631",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14307,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5145,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2234,
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
            "value": 642,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2415,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8322,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22507,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34599,
            "range": "± 82",
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
          "id": "dcb83b81365ccaf2060763c3f8a57bbd622b7582",
          "message": "Add `LIMIT` and `OFFSET` operators to evaluator (#314)",
          "timestamp": "2023-03-09T14:31:49-08:00",
          "tree_id": "1722245140d26e5ec4de44e6addd7b4b594ab3d2",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/dcb83b81365ccaf2060763c3f8a57bbd622b7582"
        },
        "date": 1678401820785,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6478,
            "range": "± 322",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 64591,
            "range": "± 4858",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 129470,
            "range": "± 8493",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 27576,
            "range": "± 1926",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 66290,
            "range": "± 2903",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 118766,
            "range": "± 6589",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 22315,
            "range": "± 1042",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 434299,
            "range": "± 24205",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 877380,
            "range": "± 44119",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 24643588,
            "range": "± 1244921",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 143224323,
            "range": "± 5720191",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 263846877,
            "range": "± 7088809",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15776,
            "range": "± 703",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5593,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2296,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 150,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 736,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2688,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9169,
            "range": "± 417",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 25749,
            "range": "± 1323",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 40305,
            "range": "± 1751",
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
          "id": "f9a0e15c392fb038f835617217e5c15752fc5cd2",
          "message": "Update to latest `partiql-tests` (#315)",
          "timestamp": "2023-03-10T10:33:19-08:00",
          "tree_id": "29e310c89cf51413a04d51d6147b4eb7ff98a89c",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/f9a0e15c392fb038f835617217e5c15752fc5cd2"
        },
        "date": 1678473904188,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6986,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 68269,
            "range": "± 1551",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 136505,
            "range": "± 1849",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 28910,
            "range": "± 428",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 68825,
            "range": "± 1137",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 119155,
            "range": "± 1793",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 22632,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 427920,
            "range": "± 1483",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 863276,
            "range": "± 1711",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27122720,
            "range": "± 393078",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 141676344,
            "range": "± 688764",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 269309910,
            "range": "± 1245059",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16939,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6147,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2609,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 173,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 789,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3006,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 10108,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 27247,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 41934,
            "range": "± 91",
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
          "id": "59e78f12b09f487d9db70e851adf0682cca98a56",
          "message": "Behavior-preserving refactor; `List`, `Bag`, and `Tuple` in own files (#316)",
          "timestamp": "2023-03-10T11:24:29-08:00",
          "tree_id": "d6cde6b9261acf1cc79af7baa3b70c07427982ce",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/59e78f12b09f487d9db70e851adf0682cca98a56"
        },
        "date": 1678476896371,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6371,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 61599,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 122110,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 24686,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 58171,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 100484,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19464,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 363917,
            "range": "± 746",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 734526,
            "range": "± 3629",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21663123,
            "range": "± 263057",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122438667,
            "range": "± 353264",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239836744,
            "range": "± 416737",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14155,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5390,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2293,
            "range": "± 2",
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
            "value": 701,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2753,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9022,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23509,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 35821,
            "range": "± 114",
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
          "id": "349d4fc4275b23699a997211b21701ac06c66b20",
          "message": "Only construct FnSymTab once, using `OnceCell` (#318)",
          "timestamp": "2023-03-10T19:26:22-08:00",
          "tree_id": "567aa2ea60246f2d82d254d4a882064fa3b23fe5",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/349d4fc4275b23699a997211b21701ac06c66b20"
        },
        "date": 1678505832212,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5865,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 58522,
            "range": "± 345",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 115425,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4663,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 34800,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 71721,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 18983,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 354654,
            "range": "± 567",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 716087,
            "range": "± 1254",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23401336,
            "range": "± 564523",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 120039363,
            "range": "± 465669",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 228019621,
            "range": "± 339893",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14173,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5153,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2249,
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
            "value": 696,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2600,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8552,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22900,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 35205,
            "range": "± 62",
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
          "id": "0ecb71d1da549189a5993aae6c5dcacb1b79f02f",
          "message": "Fix Tuple value duplicate equality and hashing (#320)",
          "timestamp": "2023-03-20T14:47:41-07:00",
          "tree_id": "6a1a1cc2e1784cfb16d0a5544700021bc5dbfdf2",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/0ecb71d1da549189a5993aae6c5dcacb1b79f02f"
        },
        "date": 1679349502406,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6861,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 67234,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 133362,
            "range": "± 544",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5083,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36001,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 74724,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19021,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 357062,
            "range": "± 1157",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 723821,
            "range": "± 4479",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21528921,
            "range": "± 660066",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122803228,
            "range": "± 2376867",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 241124643,
            "range": "± 546483",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14194,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5392,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2258,
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
            "value": 761,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3209,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 11151,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 28639,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 41601,
            "range": "± 1425",
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
          "id": "86c0199de67706bfbbeb9a212bcf54ad633e549e",
          "message": "Add timestamp value and parsing from ion (#319)",
          "timestamp": "2023-03-21T11:24:53-07:00",
          "tree_id": "f3aef883d1e5fe312ce0f1fc2b78d44c42bf0c55",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/86c0199de67706bfbbeb9a212bcf54ad633e549e"
        },
        "date": 1679423805100,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7136,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 68384,
            "range": "± 7364",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 128002,
            "range": "± 15572",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4846,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 37089,
            "range": "± 1987",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 73909,
            "range": "± 6237",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20625,
            "range": "± 1302",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 396138,
            "range": "± 25011",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 803404,
            "range": "± 49817",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22026403,
            "range": "± 1284211",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 116209448,
            "range": "± 7363452",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 218793260,
            "range": "± 7492364",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 13894,
            "range": "± 1332",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 4919,
            "range": "± 1079",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2138,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 135,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 676,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2616,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8858,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 24468,
            "range": "± 1272",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 38703,
            "range": "± 2342",
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
          "id": "05ccb642050992d47b4cf13df0b8829d289cb71d",
          "message": "Clean up some clippy warnings (#321)",
          "timestamp": "2023-03-21T14:15:23-07:00",
          "tree_id": "79f183921fc27bb56af635459db2f3fe1b7f5f12",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/05ccb642050992d47b4cf13df0b8829d289cb71d"
        },
        "date": 1679433971329,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6448,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 62260,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 123155,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4958,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36640,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 75370,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19359,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 362019,
            "range": "± 728",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 730693,
            "range": "± 1155",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21550702,
            "range": "± 142081",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122487119,
            "range": "± 408040",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239759829,
            "range": "± 396891",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14193,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5428,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2343,
            "range": "± 10",
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
            "value": 711,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2680,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9098,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23573,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34964,
            "range": "± 95",
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
          "id": "e3806a055e0f89093ed56cb5129f010f62b66178",
          "message": "Change codecov action Rust version to '2023-03-09' (#322)",
          "timestamp": "2023-03-21T14:20:53-07:00",
          "tree_id": "cd8b0fc97618fbaf0caae57ccbb7a92cd78a031f",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/e3806a055e0f89093ed56cb5129f010f62b66178"
        },
        "date": 1679434311515,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6330,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 60802,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 119981,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4984,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36563,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 75655,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19160,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 359050,
            "range": "± 1068",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 726714,
            "range": "± 2963",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21684392,
            "range": "± 116927",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122586325,
            "range": "± 302746",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239906243,
            "range": "± 366255",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14356,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5546,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2444,
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
            "value": 705,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2676,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9134,
            "range": "± 19",
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
            "value": 35334,
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
          "id": "993348bfa9e9a024951c229d284fca552dd23906",
          "message": "Handle test/mod names that start with a digit (#325)",
          "timestamp": "2023-03-22T15:44:39-07:00",
          "tree_id": "8d698b198aca6719a49a0eb7d2495e8e08d30c20",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/993348bfa9e9a024951c229d284fca552dd23906"
        },
        "date": 1679525730215,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6451,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 61766,
            "range": "± 345",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 122448,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4986,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36443,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 75223,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19598,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 363269,
            "range": "± 746",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 732255,
            "range": "± 1375",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21729994,
            "range": "± 223678",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 123463943,
            "range": "± 443243",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 242596645,
            "range": "± 513130",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14203,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5417,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2364,
            "range": "± 2",
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
            "value": 702,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2707,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8866,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23382,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 35396,
            "range": "± 114",
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
          "id": "b493f2227ab9c7f64f1fc13e0fa79634dc4769d3",
          "message": "Add initial GROUP BY implementation (#301)",
          "timestamp": "2023-03-29T11:47:26-07:00",
          "tree_id": "f236047a8f24dae333b311600084345ae1e860d5",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/b493f2227ab9c7f64f1fc13e0fa79634dc4769d3"
        },
        "date": 1680116398007,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7275,
            "range": "± 508",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 70107,
            "range": "± 5111",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 133193,
            "range": "± 7248",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5266,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 40548,
            "range": "± 1965",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 86291,
            "range": "± 3477",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 23553,
            "range": "± 1444",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 452970,
            "range": "± 20202",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 887418,
            "range": "± 38180",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23751815,
            "range": "± 988416",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 125012054,
            "range": "± 4369520",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239984833,
            "range": "± 7618946",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15083,
            "range": "± 711",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5544,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2510,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 150,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 820,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2928,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9726,
            "range": "± 391",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 27584,
            "range": "± 1449",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 40564,
            "range": "± 3395",
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
          "id": "e3e1627a16984718ef91e30814f4abf779866f73",
          "message": "Parse `TABLE <id>` references (#333)",
          "timestamp": "2023-04-03T21:01:24-07:00",
          "tree_id": "18197cb3797a1d6a021c7177756c989446059f28",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/e3e1627a16984718ef91e30814f4abf779866f73"
        },
        "date": 1680581556185,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6051,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 58813,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 118196,
            "range": "± 609",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5022,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 37065,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 75373,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19480,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 361242,
            "range": "± 881",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 728815,
            "range": "± 1988",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21713373,
            "range": "± 361830",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121711558,
            "range": "± 429383",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239092396,
            "range": "± 714497",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14163,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5494,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2397,
            "range": "± 24",
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
            "value": 686,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2641,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9167,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22644,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 33899,
            "range": "± 181",
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
          "id": "a6d47b50669db65d5d7a185a29f697388379574b",
          "message": "Properly skip comments when parsing (#332)",
          "timestamp": "2023-04-04T10:51:21-07:00",
          "tree_id": "8d162338a6e22d2056f898c7c100ec3700f810eb",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/a6d47b50669db65d5d7a185a29f697388379574b"
        },
        "date": 1680631467754,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7652,
            "range": "± 856",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 74689,
            "range": "± 3639",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 153126,
            "range": "± 6628",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5944,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 43556,
            "range": "± 2158",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 89988,
            "range": "± 7474",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 24757,
            "range": "± 1227",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 485762,
            "range": "± 34239",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 973352,
            "range": "± 39550",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27874194,
            "range": "± 1896134",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 139238922,
            "range": "± 6297889",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 273135384,
            "range": "± 10673895",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17742,
            "range": "± 850",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6469,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2813,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 174,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 854,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3428,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 11159,
            "range": "± 638",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 29467,
            "range": "± 1303",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 45692,
            "range": "± 2828",
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
          "id": "75185ddf2f63a876b6a64dcc486e4daac72514b1",
          "message": "Update `partiql-tests` (#334)",
          "timestamp": "2023-04-04T11:08:20-07:00",
          "tree_id": "b377df41f1230085063c559fa7455b7ba28a9288",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/75185ddf2f63a876b6a64dcc486e4daac72514b1"
        },
        "date": 1680632487589,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7568,
            "range": "± 791",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 72578,
            "range": "± 3836",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 145653,
            "range": "± 10095",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5951,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 45585,
            "range": "± 2736",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 96385,
            "range": "± 6363",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 25891,
            "range": "± 1829",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 497379,
            "range": "± 57917",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 960216,
            "range": "± 48055",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27671166,
            "range": "± 1901446",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 139617883,
            "range": "± 7858751",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 267582322,
            "range": "± 10083622",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 18012,
            "range": "± 1015",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6391,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2829,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 171,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 878,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3335,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 11535,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 30624,
            "range": "± 1709",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 46295,
            "range": "± 2559",
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
          "id": "11e5623b68921cda193e09a568db913120cb5501",
          "message": "Implements `ORDER BY` (#337)",
          "timestamp": "2023-04-10T10:12:56-07:00",
          "tree_id": "4955d463c5d3e00bd98c0d641148cfb3beb53e70",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/11e5623b68921cda193e09a568db913120cb5501"
        },
        "date": 1681147503398,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6625,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 63404,
            "range": "± 2371",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 131598,
            "range": "± 4452",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5102,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 38730,
            "range": "± 1709",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 80080,
            "range": "± 4153",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20802,
            "range": "± 925",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 392209,
            "range": "± 19124",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 812290,
            "range": "± 23887",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 24446318,
            "range": "± 794110",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 133570961,
            "range": "± 2766081",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 255661369,
            "range": "± 4870536",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16677,
            "range": "± 473",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5979,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2518,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 159,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 779,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2829,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9935,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 25353,
            "range": "± 997",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 38659,
            "range": "± 1383",
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
          "id": "17fd6e73c1789bdf2cb3aa945f5ed2986220d41f",
          "message": "Add initial sql aggregation implementation (#335)",
          "timestamp": "2023-04-10T15:22:32-07:00",
          "tree_id": "6144a142d45dfec7cdb0123a3013ab2a5be0d425",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/17fd6e73c1789bdf2cb3aa945f5ed2986220d41f"
        },
        "date": 1681166075923,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6959,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 67651,
            "range": "± 902",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 137684,
            "range": "± 2259",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5618,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 40914,
            "range": "± 586",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 81308,
            "range": "± 1989",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 22056,
            "range": "± 533",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 419242,
            "range": "± 4841",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 839189,
            "range": "± 19369",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 26300062,
            "range": "± 650598",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 136098701,
            "range": "± 2210380",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 255607807,
            "range": "± 4204209",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16780,
            "range": "± 502",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5885,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2459,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 168,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 770,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2896,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9997,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 25598,
            "range": "± 421",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 40720,
            "range": "± 555",
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
          "id": "ccee459d4b9b464c8a1cbc0edc16b40e0e755699",
          "message": "Update CHANGELOG ahead of 0.3.0 release (#338)",
          "timestamp": "2023-04-11T10:31:59-07:00",
          "tree_id": "586ce8974ce1ceb6aee4be6215e7e41876b99355",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/ccee459d4b9b464c8a1cbc0edc16b40e0e755699"
        },
        "date": 1681234993859,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6241,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 60181,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 120538,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5118,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36925,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 75899,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19442,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 361533,
            "range": "± 732",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 733630,
            "range": "± 1122",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21441586,
            "range": "± 133055",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122726231,
            "range": "± 431153",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 240696104,
            "range": "± 403089",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14324,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5511,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2397,
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
            "value": 706,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2735,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8966,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22500,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34062,
            "range": "± 112",
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
          "id": "2cb413a5b3c2f67444448754634af0faf361d9cb",
          "message": "chore: Release (#339)",
          "timestamp": "2023-04-11T14:53:16-07:00",
          "tree_id": "2d398670fe6c4ce313ab87b769970268d6eb8825",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/2cb413a5b3c2f67444448754634af0faf361d9cb"
        },
        "date": 1681250662652,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6295,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 60897,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 120959,
            "range": "± 440",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5187,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36956,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 76159,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19834,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 363140,
            "range": "± 949",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 737260,
            "range": "± 1587",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22145134,
            "range": "± 224355",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122278409,
            "range": "± 413341",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239722063,
            "range": "± 377737",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14707,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5451,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2324,
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
            "value": 700,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2653,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9051,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22462,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 33389,
            "range": "± 99",
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
          "id": "e63267acf3682fd6a44b148a273b0f9a21a92fe9",
          "message": "Add EXTRACT builtin function; update `partiql-tests` (#340)",
          "timestamp": "2023-04-17T16:34:42-07:00",
          "tree_id": "4c025fa9a56eb5f4f56f6bd328e869509b288001",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/e63267acf3682fd6a44b148a273b0f9a21a92fe9"
        },
        "date": 1681775158275,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6009,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 59511,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 117124,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4746,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35249,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 72488,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19141,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 356062,
            "range": "± 810",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 719534,
            "range": "± 1211",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23756020,
            "range": "± 602467",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 119588157,
            "range": "± 375164",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 226931928,
            "range": "± 379982",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14435,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 5136,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 2264,
            "range": "± 6",
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
            "value": 705,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2599,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8693,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21972,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 35056,
            "range": "± 83",
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
          "id": "c276b0d5f5be50f389a9ecaa0bdfd26ebf70084f",
          "message": "Change partiql-eval type visibilities (#342)",
          "timestamp": "2023-04-18T14:37:02-07:00",
          "tree_id": "0ddc97039e04b13523926899cc13db2f0f82fcc9",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/c276b0d5f5be50f389a9ecaa0bdfd26ebf70084f"
        },
        "date": 1681854467522,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6120,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 62313,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 122828,
            "range": "± 276",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5219,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36763,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 75859,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19600,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 358497,
            "range": "± 810",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 729668,
            "range": "± 1517",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21119571,
            "range": "± 81713",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121465910,
            "range": "± 543514",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 238587461,
            "range": "± 478116",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14417,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7176,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 636,
            "range": "± 0",
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
            "value": 732,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2706,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8937,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21854,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34450,
            "range": "± 109",
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
          "id": "380f5265feab34da02e933dff47b7845945f0699",
          "message": "Fix parsing of EXTRACT to allow keywords after the FROM (#344)",
          "timestamp": "2023-04-19T11:41:41-07:00",
          "tree_id": "b7ee38ff0e0a97d5d4ff0d41fac4545d78017650",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/380f5265feab34da02e933dff47b7845945f0699"
        },
        "date": 1681930433841,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6964,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 71203,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 140899,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5757,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 42092,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 85191,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 23317,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 434967,
            "range": "± 4659",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 872789,
            "range": "± 1855",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 26411375,
            "range": "± 721629",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 141055752,
            "range": "± 658685",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 268288230,
            "range": "± 8240336",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 18142,
            "range": "± 850",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8031,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 733,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 173,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 829,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3079,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 10263,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 26473,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 41552,
            "range": "± 152",
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
          "id": "aa107b605f1a61054a989a5fc4bc3211e54d77b6",
          "message": "Upgrade ion-rs from 0.16 to 0.17 (#348)",
          "timestamp": "2023-04-25T12:22:05-07:00",
          "tree_id": "81e3cd55eb93d155f26cf50c576e1eb4f30a7c12",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/aa107b605f1a61054a989a5fc4bc3211e54d77b6"
        },
        "date": 1682451157499,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6240,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 56979,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 113802,
            "range": "± 641",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4959,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35429,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 71496,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20310,
            "range": "± 236",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 366803,
            "range": "± 825",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 752191,
            "range": "± 1350",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21812699,
            "range": "± 145609",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 123396623,
            "range": "± 382899",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 240792858,
            "range": "± 319827",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14216,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7208,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 679,
            "range": "± 40",
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
            "value": 701,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2653,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8590,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21922,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 33654,
            "range": "± 99",
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
          "id": "ad9b5d27a19218990f849d0a7e84e3bdbed987d2",
          "message": "Fix bag,list,tuple mcaros to fully qualify type `Bag`,`List`,`Tuple` (#359)",
          "timestamp": "2023-05-08T17:13:22-07:00",
          "tree_id": "e93e0e1a24242123f0572b4a774432d81c29084d",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/ad9b5d27a19218990f849d0a7e84e3bdbed987d2"
        },
        "date": 1683591845992,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6132,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 55316,
            "range": "± 442",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 109887,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4839,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35400,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 71997,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20245,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 366432,
            "range": "± 1330",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 747190,
            "range": "± 1463",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22369460,
            "range": "± 207891",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 123786492,
            "range": "± 473108",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 241121492,
            "range": "± 541064",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14068,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7222,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 635,
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
            "value": 701,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2600,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8478,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21715,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 33471,
            "range": "± 117",
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
          "id": "72c5ae9d3665be6bf81cf7c802faba437a1a7f0b",
          "message": "Add `partiql-extension-ion` extension for encoding/decoding Ion from/into PartiQL `Value`s (#362)\n\n- Extracts ion decoding into `Value` from the various places it was in the codebase and centralizes it into `partiql-extension-ion`\r\n- Updates the decoding to remove panics and instead return `Result` types\r\n- Adds encoding of `Value` back into Ion\r\n- Supports both 'unlifted' Ion to Value as well as the 'partiql encoded in ion' used by both the conformance tests and the partiql-lang-kotlin implementation\r\n- Abstracts encoding/decoding to/from IonWriter/IonReader",
          "timestamp": "2023-05-16T17:25:19-07:00",
          "tree_id": "cb0b7de76c3259fadf8e40c8a21d6c0d93eeb5f4",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/72c5ae9d3665be6bf81cf7c802faba437a1a7f0b"
        },
        "date": 1684283771368,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6483,
            "range": "± 622",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 55397,
            "range": "± 4778",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 113625,
            "range": "± 11218",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4577,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35973,
            "range": "± 3041",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 71133,
            "range": "± 7706",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 23618,
            "range": "± 2105",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 404804,
            "range": "± 39939",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 826707,
            "range": "± 54614",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21570786,
            "range": "± 1553055",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121322681,
            "range": "± 10381113",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 235943878,
            "range": "± 16741227",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15194,
            "range": "± 1839",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6795,
            "range": "± 646",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 637,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 141,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 884,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3363,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 11524,
            "range": "± 1170",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 30279,
            "range": "± 2223",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 47774,
            "range": "± 3359",
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
          "id": "d18d7baa030e22a9d1730769795394d4234551a4",
          "message": "Change `lower` to return `Result` rather than `panic` (#361)",
          "timestamp": "2023-05-17T14:29:34-07:00",
          "tree_id": "f6d96769609661ade5f26e221ef03fb96c4df2aa",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/d18d7baa030e22a9d1730769795394d4234551a4"
        },
        "date": 1684359720411,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 8930,
            "range": "± 665",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 76516,
            "range": "± 4421",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 154644,
            "range": "± 9526",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6874,
            "range": "± 750",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 49260,
            "range": "± 4957",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 95045,
            "range": "± 6729",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 28625,
            "range": "± 1261",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 522281,
            "range": "± 25178",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 1037943,
            "range": "± 59694",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27105077,
            "range": "± 949853",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 144460994,
            "range": "± 3965104",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 278580620,
            "range": "± 7410063",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17429,
            "range": "± 1043",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8351,
            "range": "± 511",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 809,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 175,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 952,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3368,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 10637,
            "range": "± 605",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 29687,
            "range": "± 1112",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 45360,
            "range": "± 1465",
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
          "id": "4bdd42283357af85e6e4176015465d913e369add",
          "message": "Modify `parse_embedded_ion_str` to return `Result`, not `expect()` (#366)",
          "timestamp": "2023-05-17T15:19:37-07:00",
          "tree_id": "ebc3ee069515e0d8312345464e54ccc8257a8e22",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/4bdd42283357af85e6e4176015465d913e369add"
        },
        "date": 1684362659603,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6607,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 63798,
            "range": "± 1334",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 126175,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5681,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 40557,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 80767,
            "range": "± 757",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 23662,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 433724,
            "range": "± 3825",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 878498,
            "range": "± 4203",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27780159,
            "range": "± 378559",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 142228594,
            "range": "± 654011",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 269525587,
            "range": "± 891021",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17054,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8292,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 790,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 173,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 790,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2864,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9484,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 25284,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 41737,
            "range": "± 95",
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
          "id": "e0badca605a5d079001d643712f94be498fdb03a",
          "message": "Change AST to logical plan function argument lowering to return Result (#367)",
          "timestamp": "2023-05-17T21:25:11-07:00",
          "tree_id": "079eeb89954c3fa60bc7fd57a21d84ea62cdc6f7",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/e0badca605a5d079001d643712f94be498fdb03a"
        },
        "date": 1684384527250,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6061,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 55370,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 109848,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5176,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35564,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 71948,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20338,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 371476,
            "range": "± 937",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 754477,
            "range": "± 1621",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23555365,
            "range": "± 383520",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 123178448,
            "range": "± 667828",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 240271459,
            "range": "± 531132",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14243,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7305,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 625,
            "range": "± 0",
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
            "value": 708,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2546,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8644,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22484,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34207,
            "range": "± 140",
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
          "id": "8dbe7a46ad76df9cd61f1d825777553adfec056b",
          "message": "Update lalrpop to v0.20 (#369)",
          "timestamp": "2023-05-18T13:41:28-07:00",
          "tree_id": "0008a71a8cea4fc1e83ac4b4de9fd5b72895ad5d",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/8dbe7a46ad76df9cd61f1d825777553adfec056b"
        },
        "date": 1684443093740,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6142,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 55143,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 109108,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4943,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35857,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 72222,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20408,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 371511,
            "range": "± 1016",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 757607,
            "range": "± 1460",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21671854,
            "range": "± 173356",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 123211496,
            "range": "± 457918",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 240886507,
            "range": "± 334522",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 13997,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7352,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 694,
            "range": "± 0",
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
            "value": 704,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2634,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8667,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22168,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 33774,
            "range": "± 84",
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
          "id": "425d3cccbb14dc5879b78d070f2441087e9bf891",
          "message": "Preliminary work to create a Catalog, plumb it through lowering and evaluation with a `read_ion` extension function. (#375)\n\nMVP for a Catalog for eventually holding Functions, Types, Schemas, etc. For now, it is able to hold a BaseTableFunction that is used to allow scanning of arbitrary data sources via an 'extension' plug-in interface.\r\n\r\nThe new partiql-extension-ion-functions crate provides an example an extension in IonExtension, which registers a ReadIonFunction.",
          "timestamp": "2023-05-24T19:05:40-07:00",
          "tree_id": "9ff322cf39568b9c892e86b79c55aa4da6a1f0fb",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/425d3cccbb14dc5879b78d070f2441087e9bf891"
        },
        "date": 1684981017107,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6556,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 61896,
            "range": "± 2639",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 120027,
            "range": "± 3622",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5479,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 39933,
            "range": "± 1516",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 78845,
            "range": "± 2600",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 23247,
            "range": "± 956",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 422144,
            "range": "± 13026",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 833501,
            "range": "± 29380",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23491583,
            "range": "± 961139",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 136653164,
            "range": "± 4191411",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 255348535,
            "range": "± 7394218",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16559,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7640,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 686,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 163,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 767,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2772,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9316,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 24205,
            "range": "± 856",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 38564,
            "range": "± 1503",
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
          "id": "637fdab7f45545c5eaebc8d7d0ac57755fb62c8a",
          "message": "Adds the COLL_* functions (COLL_AVG, COLL_COUNT, COLL_MAX, COLL_MIN, COLL_SUM) (#353)",
          "timestamp": "2023-05-24T19:59:19-07:00",
          "tree_id": "a779e82ceeadb43603efa1f68f9b3cec7c564aba",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/637fdab7f45545c5eaebc8d7d0ac57755fb62c8a"
        },
        "date": 1684984307551,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7458,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 71998,
            "range": "± 3624",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 141398,
            "range": "± 4833",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6094,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 46197,
            "range": "± 1763",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 94178,
            "range": "± 2745",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 28685,
            "range": "± 1407",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 499776,
            "range": "± 25115",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 996037,
            "range": "± 33623",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 25723129,
            "range": "± 1514078",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 144582397,
            "range": "± 6746094",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 261155781,
            "range": "± 11056306",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17192,
            "range": "± 1296",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7788,
            "range": "± 528",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 828,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 166,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 926,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3067,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 10548,
            "range": "± 562",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 28347,
            "range": "± 1909",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 45026,
            "range": "± 2008",
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
          "id": "7e56972ab3b8a15ca914239e461a1ecd0b2f3984",
          "message": "Changes logical plan to eval plan API to return `Result` type (#363)",
          "timestamp": "2023-05-24T20:46:06-07:00",
          "tree_id": "6d3a190b32820bfa45df69496cfb2f8912e809de",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/7e56972ab3b8a15ca914239e461a1ecd0b2f3984"
        },
        "date": 1684986993928,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5514,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 54816,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 108335,
            "range": "± 629",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4761,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 34997,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 69759,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20141,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 359609,
            "range": "± 776",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 729894,
            "range": "± 1110",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 20765921,
            "range": "± 401978",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 118247204,
            "range": "± 265923",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 225807187,
            "range": "± 842717",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14225,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6726,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 687,
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
            "value": 699,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2362,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8034,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21151,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34326,
            "range": "± 72",
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
          "id": "b2d86dba807092971da85fdb8e0b48c71ddb8f9d",
          "message": "chore: Release v0.4.0 (#376)",
          "timestamp": "2023-05-24T22:38:15-07:00",
          "tree_id": "cd3e1d0068f667a3d6d0d159f244ebdefdaee269",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/b2d86dba807092971da85fdb8e0b48c71ddb8f9d"
        },
        "date": 1684993707550,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5952,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 55718,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 113396,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5058,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36163,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 73653,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20773,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 373948,
            "range": "± 1029",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 757205,
            "range": "± 975",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 20199926,
            "range": "± 188401",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122440308,
            "range": "± 307503",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 240104652,
            "range": "± 397280",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14149,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7468,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 659,
            "range": "± 7",
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
            "value": 732,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2684,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8800,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22476,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34636,
            "range": "± 117",
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
          "id": "fe2dbee030f3b7b8a023ef648a3b30f01c3264f8",
          "message": "IonExtension is intended to be `pub` (#377)",
          "timestamp": "2023-05-25T15:21:22-07:00",
          "tree_id": "3fe9c1b386a01041fc6866428cd7f16a3b182999",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/fe2dbee030f3b7b8a023ef648a3b30f01c3264f8"
        },
        "date": 1685053908777,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5867,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 57554,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 113674,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4752,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 34468,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 69533,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20467,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 367453,
            "range": "± 836",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 743078,
            "range": "± 2860",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 20079716,
            "range": "± 400188",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 119319652,
            "range": "± 456709",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 228499225,
            "range": "± 365558",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14296,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6789,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 622,
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
            "value": 667,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2345,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8039,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21628,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34656,
            "range": "± 61",
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
          "id": "37bce5e7b47f247826f86ebec3d6b0e3e20d3a71",
          "message": "chore: Release v0.4.1 (#378)",
          "timestamp": "2023-05-25T15:57:00-07:00",
          "tree_id": "746c967658d317fecde660cb6460853dbf279158",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/37bce5e7b47f247826f86ebec3d6b0e3e20d3a71"
        },
        "date": 1685056081740,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6059,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 60970,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 121029,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4714,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 34708,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 70210,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19834,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 363269,
            "range": "± 640",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 731501,
            "range": "± 4874",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21826487,
            "range": "± 1556441",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121743532,
            "range": "± 2939271",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 230370579,
            "range": "± 2204752",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14340,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6973,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 661,
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
            "value": 663,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2342,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7915,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20935,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34025,
            "range": "± 140",
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
          "id": "114e1d5a79fc24fff1aacece925e6c1d9f43ab1f",
          "message": "Remove panics in `EvalExpr`, `Evaluable`, and `execute_mut` (#374)",
          "timestamp": "2023-06-01T16:13:51-07:00",
          "tree_id": "7bcaa427192f543cfce5f6273da8328d4250bd97",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/114e1d5a79fc24fff1aacece925e6c1d9f43ab1f"
        },
        "date": 1685661868569,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5533,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 53748,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 106548,
            "range": "± 2519",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4827,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35319,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 71106,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19849,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 360687,
            "range": "± 1179",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 725634,
            "range": "± 5207",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 20949322,
            "range": "± 337042",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 119602703,
            "range": "± 319411",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 228076207,
            "range": "± 295767",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14439,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6878,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 645,
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
            "value": 670,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2402,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8053,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21755,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34535,
            "range": "± 73",
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
          "id": "f2bdd18fcf251c219b186ed324063fe75092d7b8",
          "message": "Update codecov CI to stable version of Rust; change conformance test GH Action to use nightly (#383)\n\n* Update codecov CI to stable version of Rust\r\n\r\n* Try updating conformance test report generation rust toolchains to nightly",
          "timestamp": "2023-06-02T15:30:33-07:00",
          "tree_id": "0017f0329b6c73e7fa9554b024e5147204df6003",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/f2bdd18fcf251c219b186ed324063fe75092d7b8"
        },
        "date": 1685745662449,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5496,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 53665,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 106788,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4785,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35271,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 71477,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19851,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 359616,
            "range": "± 2109",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 725686,
            "range": "± 2566",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 19882945,
            "range": "± 250612",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 119073372,
            "range": "± 395906",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 227881963,
            "range": "± 344716",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14533,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6794,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 697,
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
            "value": 697,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2434,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8086,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21676,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34807,
            "range": "± 58",
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
          "id": "dc1edc4a648d20a23e2363178603d80cce57ee7b",
          "message": "Add `operators_by_id` to Logical Plan to get Nodes and their OpId (#382)",
          "timestamp": "2023-06-02T16:54:09-07:00",
          "tree_id": "54b85a11b889cde95df35c4e6873bf8a57a7f123",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/dc1edc4a648d20a23e2363178603d80cce57ee7b"
        },
        "date": 1685750683167,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6016,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 57378,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 115345,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5056,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36260,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 73789,
            "range": "± 2129",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20463,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 365842,
            "range": "± 1150",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 740782,
            "range": "± 1555",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 20629280,
            "range": "± 284479",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 123244696,
            "range": "± 401642",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 241014215,
            "range": "± 1775508",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14409,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7206,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 648,
            "range": "± 0",
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
            "value": 711,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2626,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8741,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22777,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 35683,
            "range": "± 356",
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
          "id": "024620a2d3b26374897350af373ffb9f6c21f429",
          "message": "Translate collection literals to Value when lowering (#380)",
          "timestamp": "2023-06-03T18:02:50-07:00",
          "tree_id": "5b6f2d13d62e8be150ed861de9d9b38d3909190a",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/024620a2d3b26374897350af373ffb9f6c21f429"
        },
        "date": 1685841209857,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6028,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 57453,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 115181,
            "range": "± 708",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5096,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36466,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 74352,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20578,
            "range": "± 647",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 367561,
            "range": "± 1287",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 745146,
            "range": "± 1620",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 20161461,
            "range": "± 271804",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121602615,
            "range": "± 304646",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 238276133,
            "range": "± 3529775",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14337,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7189,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 704,
            "range": "± 11",
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
            "value": 695,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2728,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8671,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22174,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 35008,
            "range": "± 112",
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
          "id": "85995935ac7026d2fc7a0201e29c8669932ef7ae",
          "message": "Add plan flows for `JOIN`. Add 'self managed' graph nodes. (#385)",
          "timestamp": "2023-06-05T14:17:45-07:00",
          "tree_id": "8d88efa3da8d42cc4f7651f83e4d8709f5f79fc3",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/85995935ac7026d2fc7a0201e29c8669932ef7ae"
        },
        "date": 1686000637299,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7744,
            "range": "± 459",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 74508,
            "range": "± 3109",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 145739,
            "range": "± 6524",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6283,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 48984,
            "range": "± 2206",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 101284,
            "range": "± 4182",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 27872,
            "range": "± 2440",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 526660,
            "range": "± 33205",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 1071481,
            "range": "± 64031",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27906859,
            "range": "± 1342101",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 151754110,
            "range": "± 4065915",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 286354827,
            "range": "± 6707477",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17884,
            "range": "± 560",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8557,
            "range": "± 329",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 792,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 175,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 1043,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 4023,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 12741,
            "range": "± 391",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 34103,
            "range": "± 1355",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 54291,
            "range": "± 22163",
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
          "id": "8bdb756f98153819af69cf380ffdb154a9fdf064",
          "message": "v0.5.0 Release prep.",
          "timestamp": "2023-06-06T13:32:21-07:00",
          "tree_id": "f3ac62b458b98ccf5f7d5788c558b8eea366bd5d",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/8bdb756f98153819af69cf380ffdb154a9fdf064"
        },
        "date": 1686084237200,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6449,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 62035,
            "range": "± 2008",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 124431,
            "range": "± 3675",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5530,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 39468,
            "range": "± 1444",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 80215,
            "range": "± 2499",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 22635,
            "range": "± 616",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 415913,
            "range": "± 10187",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 836447,
            "range": "± 18095",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 24701122,
            "range": "± 875027",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 133401376,
            "range": "± 3020484",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 256324216,
            "range": "± 4857182",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16348,
            "range": "± 1582",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7576,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 692,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 168,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 827,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2887,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9188,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 24304,
            "range": "± 584",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 39085,
            "range": "± 1276",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}