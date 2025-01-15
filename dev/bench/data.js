window.BENCHMARK_DATA = {
  "lastUpdate": 1736984309043,
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
          "id": "15fe939a35bf508d8f66fa7f4edd4b087a5c4578",
          "message": "Move `name_resolver` to a different crate (#387)\n\nThis PR is in preparation of upcomming work for adding an AST Typer.\r\nThe name resolver seems to be required by the `typer` but it being in\r\n`partiql-logical-planner` implies its use only in planner.\r\n\r\nIn addition, this PR defines a new crate call `partiql-ast-passes`\r\n(could pick a better name and open to ideas)  that\r\nintends to include passes on AST transformation. Arguably `lower` in\r\n`partiql-logical-planner` can also be part of this new crate but leaving\r\nit out for now, as logical planning is a major transform and perhaps deserves\r\nits own crate.\r\n\r\n* This PR also updates the `partiql-tests` sub-module.",
          "timestamp": "2023-06-07T15:43:47-07:00",
          "tree_id": "6599b6cb0ba34b9b0ca42a9cb7bcdcc3ed46cbf3",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/15fe939a35bf508d8f66fa7f4edd4b087a5c4578"
        },
        "date": 1686178470068,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6080,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 55784,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 112905,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5314,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36802,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 74525,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20232,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 364387,
            "range": "± 1103",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 739409,
            "range": "± 1554",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22104702,
            "range": "± 162526",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121983473,
            "range": "± 533637",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 238539502,
            "range": "± 516303",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14340,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7351,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 634,
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
            "value": 710,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2638,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8644,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22227,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34834,
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
          "id": "a2e6bde3d3bd95d76f2a73055bee2160db1e33e8",
          "message": "Upgrade ion-rs to 0.18 (#390)",
          "timestamp": "2023-06-12T11:34:10-07:00",
          "tree_id": "4f4139b613c46f4aa6c65a5f3232f5b32b80e19d",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/a2e6bde3d3bd95d76f2a73055bee2160db1e33e8"
        },
        "date": 1686595573141,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6229,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 58816,
            "range": "± 3210",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 126349,
            "range": "± 6685",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5268,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 41735,
            "range": "± 1991",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 88240,
            "range": "± 9123",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 24190,
            "range": "± 1570",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 474705,
            "range": "± 28008",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 948299,
            "range": "± 40485",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 24985405,
            "range": "± 1548746",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 126878607,
            "range": "± 7686760",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 237218353,
            "range": "± 9472421",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14649,
            "range": "± 800",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7046,
            "range": "± 478",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 735,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 145,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 768,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2774,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9263,
            "range": "± 562",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 25088,
            "range": "± 1698",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 42060,
            "range": "± 3227",
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
          "id": "67777c2978d9d683312fab6a318deded6d53d903",
          "message": "Add encoding/decoding to/from Ion Element (#391)",
          "timestamp": "2023-06-12T13:10:48-07:00",
          "tree_id": "1987602fb7c3a9adb301a065ddd4d6176564947f",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/67777c2978d9d683312fab6a318deded6d53d903"
        },
        "date": 1686601283252,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6140,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 57836,
            "range": "± 1200",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 115780,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5132,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36303,
            "range": "± 1136",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 73402,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20075,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 361972,
            "range": "± 2738",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 736682,
            "range": "± 1905",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21926548,
            "range": "± 156802",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122779721,
            "range": "± 460689",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 240222097,
            "range": "± 566566",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14092,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7271,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 712,
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
            "value": 714,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2681,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8816,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23207,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 36234,
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
          "id": "8b6acd8c6dd2e5e6b4877b58fb91a60a4700db94",
          "message": "Update test generation to run strict eval and static analysis tests (#392)",
          "timestamp": "2023-06-13T10:00:25-07:00",
          "tree_id": "ecf2a641fc3179ce99bd986314b2b1e9e2b3855c",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/8b6acd8c6dd2e5e6b4877b58fb91a60a4700db94"
        },
        "date": 1686676338501,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6842,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 65874,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 130601,
            "range": "± 387",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5963,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 43103,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 88503,
            "range": "± 10245",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 24323,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 441480,
            "range": "± 7048",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 890185,
            "range": "± 3675",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27760325,
            "range": "± 670891",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 144544074,
            "range": "± 253657",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 273903391,
            "range": "± 5853941",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 18040,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8050,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 754,
            "range": "± 13",
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
            "value": 807,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2926,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9971,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 26575,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 42677,
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
          "id": "ff3e950645f76ac93c985446f11ce4cf7527810c",
          "message": "Add feature flags to conformance tests based on test type (#393)",
          "timestamp": "2023-06-13T21:07:42-07:00",
          "tree_id": "ad310bf83e37f12f2db0573aff73c4ca756f921d",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/ff3e950645f76ac93c985446f11ce4cf7527810c"
        },
        "date": 1686716300140,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5980,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 56132,
            "range": "± 341",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 113067,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5277,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36765,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 74446,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20083,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 365764,
            "range": "± 870",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 742654,
            "range": "± 1077",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21715733,
            "range": "± 719301",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122458027,
            "range": "± 499204",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239598074,
            "range": "± 537477",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14209,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7170,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 632,
            "range": "± 0",
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
            "value": 707,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2664,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8678,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22316,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 35171,
            "range": "± 90",
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
          "id": "babd7ac1eaae23d89a614193d47990a748f45b64",
          "message": "Add `partiql-types` and literals typing (#389)",
          "timestamp": "2023-06-14T08:20:54-07:00",
          "tree_id": "e2d9c651eaac7584a8e33b297cbc083f111e0406",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/babd7ac1eaae23d89a614193d47990a748f45b64"
        },
        "date": 1686756806444,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6933,
            "range": "± 847",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 65659,
            "range": "± 5519",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 135387,
            "range": "± 8910",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6051,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 44369,
            "range": "± 5790",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 95681,
            "range": "± 21741",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 26157,
            "range": "± 2358",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 502372,
            "range": "± 30894",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 1001031,
            "range": "± 68498",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27122381,
            "range": "± 1817512",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 142875254,
            "range": "± 5879075",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 270865397,
            "range": "± 19224673",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17216,
            "range": "± 1123",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8165,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 737,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 160,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 827,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3026,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 10373,
            "range": "± 1067",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 27630,
            "range": "± 1303",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 45491,
            "range": "± 3352",
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
          "id": "004d2c04a09f897b652c5bf20b20c6023db3dcdb",
          "message": "Use `partiql_ast::ast::AstTypeMap` for `LocationMap` (#394)\n\nAddressing the comment in PR #389, this PR re-uses the `AstTypeMap`\r\nfor `LocationMap` which removes a dependency to `HashMap` as `AstTypeMap`\r\nuses `IndexMap`.",
          "timestamp": "2023-06-14T13:32:49-07:00",
          "tree_id": "bae213675e834573973c112d66493278e10a7ebf",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/004d2c04a09f897b652c5bf20b20c6023db3dcdb"
        },
        "date": 1686775490488,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6627,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 65440,
            "range": "± 867",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 129684,
            "range": "± 1883",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5852,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 41875,
            "range": "± 588",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 85142,
            "range": "± 1376",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 23148,
            "range": "± 404",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 427549,
            "range": "± 5974",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 873706,
            "range": "± 11191",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 28031260,
            "range": "± 535227",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 143519947,
            "range": "± 1527247",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 269445526,
            "range": "± 2782126",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17721,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8129,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 765,
            "range": "± 7",
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
            "value": 883,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3202,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 10143,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 26513,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 41816,
            "range": "± 150",
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
          "id": "dcb3c0be8546b35b44570dd6bcb77930d2ce7fa3",
          "message": "Add aliases for PartiqlCatalog (#395)\n\n- Adds aliases for `PartiqlCatalog`\r\n- Also changes CatalogError to Vec",
          "timestamp": "2023-06-14T15:10:29-07:00",
          "tree_id": "9346c3b8a77c7fc28cf027b17a9ebfa64d2449df",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/dcb3c0be8546b35b44570dd6bcb77930d2ce7fa3"
        },
        "date": 1686781271877,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6254,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 58177,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 114443,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5360,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 37991,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 76812,
            "range": "± 1195",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 20687,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 362161,
            "range": "± 875",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 740406,
            "range": "± 12970",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21723128,
            "range": "± 301798",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122654108,
            "range": "± 481603",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 239668448,
            "range": "± 541794",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14246,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7253,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 651,
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
            "value": 759,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2854,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9070,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22208,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 35239,
            "range": "± 3138",
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
          "id": "a3f64f53504ce4478f81fbf5352351ebd2932201",
          "message": "Rename `StaticType` and add `TypeEnv` to the Catalog (#396)\n\n- Adds TypeEnv to the Catalog\r\n- Renames StaticType to refrain from conveying an inaccurate message that this type is only used for Static typing. with PartiQL b/c seemingly, we need to do a combination of Static And Dynamic Typing.",
          "timestamp": "2023-06-14T16:14:48-07:00",
          "tree_id": "c42090b2370829734c36d042cfde22d59fb0989c",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/a3f64f53504ce4478f81fbf5352351ebd2932201"
        },
        "date": 1686785218983,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6798,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 66347,
            "range": "± 1046",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 129394,
            "range": "± 1955",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5914,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 42800,
            "range": "± 656",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 86598,
            "range": "± 1181",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 23847,
            "range": "± 341",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 437411,
            "range": "± 10127",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 878109,
            "range": "± 9840",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 26414000,
            "range": "± 657182",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 144335716,
            "range": "± 1177810",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 271022392,
            "range": "± 2417086",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 17922,
            "range": "± 417",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8049,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 787,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 171,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 891,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3104,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 10132,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 26420,
            "range": "± 370",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 41890,
            "range": "± 787",
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
          "id": "fd60ab3d463a0cf95fa124b010aeae0e9ef4995b",
          "message": " Include alias in catalog find_by_name (#397)\n\nThis is a missing implementation from #395 which allows\r\nfinding catalog enteries by alias.",
          "timestamp": "2023-06-15T11:05:15-07:00",
          "tree_id": "fdf17d1e271244c9d418fc2d950c0fd1f3baaaf5",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/fd60ab3d463a0cf95fa124b010aeae0e9ef4995b"
        },
        "date": 1686852973035,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5866,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 56327,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 109739,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4963,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35386,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 71439,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19707,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 360530,
            "range": "± 678",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 726676,
            "range": "± 2432",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 24591855,
            "range": "± 665164",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121105626,
            "range": "± 484672",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 228895093,
            "range": "± 397360",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14304,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6756,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 691,
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
            "value": 728,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2613,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8571,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21975,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34815,
            "range": "± 82",
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
          "id": "bd7716dfbec57166d032fefe141a3bdafed04a82",
          "message": "Add macro_rules for `partiql-types` and remove `partiql` from value macro_rules (#398)\n\nAdds first iteration of PartiqlType macro_rules and renames the existing\r\nvalue macro_rules to make shorter.",
          "timestamp": "2023-06-15T15:00:10-07:00",
          "tree_id": "9fb1f357f609864468836b5877a0ff0756b9887b",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/bd7716dfbec57166d032fefe141a3bdafed04a82"
        },
        "date": 1686867078042,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6001,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 56753,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 110650,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4982,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36153,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 73160,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 19469,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 357747,
            "range": "± 896",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 723337,
            "range": "± 1558",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 26210844,
            "range": "± 1825101",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 121760759,
            "range": "± 1415036",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 229787046,
            "range": "± 1845399",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14369,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6816,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 650,
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
            "value": 735,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2660,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8593,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21988,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34789,
            "range": "± 83",
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
          "id": "5629e88a19ca1e3d3d6ac4772b2ed53b3180d405",
          "message": "Modify parser&AST to allow orderby/limit/offset on children of setop (#401)\n\n* Modify parser&AST to allow orderby/limit/offset on children of setop\r\n\r\n* Add docs, update changelog, additional parse tests, minor refactor\r\n\r\n---------\r\n\r\nCo-authored-by: Alan Cai <caialan@amazon.com>",
          "timestamp": "2023-06-28T16:27:13-07:00",
          "tree_id": "58d3c28388fd85d933a5b32d929c72506d826506",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/5629e88a19ca1e3d3d6ac4772b2ed53b3180d405"
        },
        "date": 1687995530224,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5870,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 58046,
            "range": "± 1846",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 109543,
            "range": "± 2610",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5608,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 41215,
            "range": "± 756",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 82711,
            "range": "± 1861",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 22839,
            "range": "± 491",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 428149,
            "range": "± 6372",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 837027,
            "range": "± 21035",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 26140310,
            "range": "± 662213",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 140488315,
            "range": "± 2297844",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 267017773,
            "range": "± 4134560",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16619,
            "range": "± 385",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8001,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 770,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 170,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 868,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2709,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8764,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22901,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 36577,
            "range": "± 596",
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
          "id": "0f49d1b63f5743061eeebb94d9024a66f4efdb13",
          "message": "Fix parsing of multiple consecutive path wildcard, unpivot, path expressions (#405)",
          "timestamp": "2023-07-06T13:00:36-07:00",
          "tree_id": "cee93948a860032dd260d2f1430c960de364f82f",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/0f49d1b63f5743061eeebb94d9024a66f4efdb13"
        },
        "date": 1688674444919,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7583,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 66156,
            "range": "± 3197",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 134079,
            "range": "± 11376",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6523,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 49290,
            "range": "± 3571",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 103258,
            "range": "± 4633",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 128961,
            "range": "± 7451",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 2000121,
            "range": "± 79755",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 3977293,
            "range": "± 207897",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 30811155,
            "range": "± 1574887",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 160098924,
            "range": "± 3808858",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 315382826,
            "range": "± 7469932",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 19250,
            "range": "± 915",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 9058,
            "range": "± 431",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 932,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 178,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 961,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3011,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 10280,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 28695,
            "range": "± 1025",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 39083,
            "range": "± 2649",
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
          "id": "139b276971111534467ed01974b62ed1fe368e24",
          "message": "Slight refactor to PathSteps in grammar (#406)",
          "timestamp": "2023-07-06T14:45:00-07:00",
          "tree_id": "6db32ab1b2b87c0b106fc16693d5cd56037693a1",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/139b276971111534467ed01974b62ed1fe368e24"
        },
        "date": 1688680551285,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5709,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51948,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 101480,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5103,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36058,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 73369,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 65028,
            "range": "± 381",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1038065,
            "range": "± 6633",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2068796,
            "range": "± 3822",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 24252865,
            "range": "± 697456",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 125641873,
            "range": "± 419609",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 236550922,
            "range": "± 921762",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15188,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6943,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 693,
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
            "value": 728,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2205,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7393,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20590,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29387,
            "range": "± 91",
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
          "id": "bb3487b3608b30035422aff1b44f534a287936b9",
          "message": "Add AST to logical plan lowering for IN expressions (#409)",
          "timestamp": "2023-07-11T11:08:18-07:00",
          "tree_id": "ab273f9f37675548ab96bcaf277d77c766882f0f",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/bb3487b3608b30035422aff1b44f534a287936b9"
        },
        "date": 1689099563639,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5982,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 54630,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 107762,
            "range": "± 521",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5005,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35840,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 72397,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 63741,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1025000,
            "range": "± 1518",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2043797,
            "range": "± 4096",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23051598,
            "range": "± 703546",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 123163110,
            "range": "± 470992",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 232900709,
            "range": "± 464222",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14985,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6757,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 722,
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
            "value": 767,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2251,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7446,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20418,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29143,
            "range": "± 70",
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
          "id": "d1bdfb05bf5578dab60de91c5d8b526f8ca70b99",
          "message": "Implements OUTER bag operators (#400)\n\nCo-authored-by: Josh Pschorr <joshps@amazon.com>",
          "timestamp": "2023-07-12T10:14:08-07:00",
          "tree_id": "becfc9b2e23bfa869d12b9c34deeeb11d15a7bc3",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/d1bdfb05bf5578dab60de91c5d8b526f8ca70b99"
        },
        "date": 1689182833068,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6534,
            "range": "± 472",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 59515,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 121699,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6205,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 42933,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 86644,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 75216,
            "range": "± 222",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1212825,
            "range": "± 4490",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2432922,
            "range": "± 11089",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 26858891,
            "range": "± 398062",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 144390233,
            "range": "± 805477",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 272423461,
            "range": "± 579099",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 18172,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8274,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 885,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 173,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 888,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2796,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8964,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 24322,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34943,
            "range": "± 146",
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
          "id": "309dd6a45b8e3f190fa1e2350a6fc3044c034f0e",
          "message": "Box Decimals in `Value` to assure `Value` fits in 16 bytes (#411)",
          "timestamp": "2023-07-18T12:08:24-07:00",
          "tree_id": "9a57c189f3517bc66c0c6347d0e187ebd61330e2",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/309dd6a45b8e3f190fa1e2350a6fc3044c034f0e"
        },
        "date": 1689707938541,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5562,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51507,
            "range": "± 228",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 101079,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5416,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36476,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 73766,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 62788,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 992396,
            "range": "± 4118",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 1994707,
            "range": "± 3420",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22185744,
            "range": "± 230734",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 125747983,
            "range": "± 504860",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 245001105,
            "range": "± 419301",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14698,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 7195,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 697,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 762,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2532,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7870,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19997,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28494,
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
          "id": "2c491e696e7bfd748b65c8ada833cbf74543a666",
          "message": "Add interface for strict mode evaluation. (#412)\n\n* Add interface for strict mode evaluation.\r\n* Add changelog",
          "timestamp": "2023-07-19T14:11:50-07:00",
          "tree_id": "10dc32ed21271a1d901b3d3cb4dcbea0b8ebd2e0",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/2c491e696e7bfd748b65c8ada833cbf74543a666"
        },
        "date": 1689801770200,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5238,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 49572,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 97635,
            "range": "± 455",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5147,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35485,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 72359,
            "range": "± 1012",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67863,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1078826,
            "range": "± 1869",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2156963,
            "range": "± 9160",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23467232,
            "range": "± 869348",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 123264341,
            "range": "± 851631",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 233485927,
            "range": "± 363525",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15442,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6783,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 703,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 725,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2316,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7489,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19789,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29163,
            "range": "± 119",
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
          "id": "a43f057c49d3ab5740c0258ebf6cbe217d972308",
          "message": "Fix `fail_semantics` function (#414)",
          "timestamp": "2023-07-19T14:24:19-07:00",
          "tree_id": "7f4f06bfd818387afee7223c953a9f41162321de",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/a43f057c49d3ab5740c0258ebf6cbe217d972308"
        },
        "date": 1689802485537,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5378,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 49525,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 96736,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5001,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35639,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 72061,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 68295,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1067965,
            "range": "± 1438",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2139144,
            "range": "± 2713",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 21820620,
            "range": "± 160328",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 119950776,
            "range": "± 744158",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 228699104,
            "range": "± 515565",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15278,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6786,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 701,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 733,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2254,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7566,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19962,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29028,
            "range": "± 71",
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
          "id": "625aa081850b8c12513c9a25ac86f85f157e37dc",
          "message": "Change modeling of project lists in logical plan to a `Vec` rather than `HashMap` (#415)",
          "timestamp": "2023-07-28T16:32:21-07:00",
          "tree_id": "3fa7d0cdb6266dd9c450ea8e2dbcca2c1f9ed42f",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/625aa081850b8c12513c9a25ac86f85f157e37dc"
        },
        "date": 1690587773051,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5379,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50408,
            "range": "± 2260",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 100417,
            "range": "± 6670",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4700,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 36433,
            "range": "± 1775",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 74912,
            "range": "± 7558",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 93098,
            "range": "± 3312",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1459815,
            "range": "± 74896",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2989160,
            "range": "± 186553",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 20888650,
            "range": "± 673268",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 114418618,
            "range": "± 3853937",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 221320721,
            "range": "± 10783798",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 13619,
            "range": "± 789",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 6240,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 673,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 43,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 731,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2295,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7637,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20476,
            "range": "± 834",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29737,
            "range": "± 1340",
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
          "id": "191b10ef1b51dc5f7935f9ffbbc176c9989f925c",
          "message": "Separate binding from evaluation of expression & implement strict eval (#413)\n\n* Add interface for strict mode evaluation.\r\n* Separate binding from evaluation of expression & implement strict eval\r\n* Update conformance test data\r\n* Better conformance test reporting\r\n* Refactor to better categorize expressions\r\n* Remove `is_null_or_missing`",
          "timestamp": "2023-08-02T14:24:04-07:00",
          "tree_id": "62c63735969795b1a330e20dcc103a13c823e98b",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/191b10ef1b51dc5f7935f9ffbbc176c9989f925c"
        },
        "date": 1691012155038,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6260,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 58068,
            "range": "± 1031",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 115922,
            "range": "± 1136",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5801,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 41542,
            "range": "± 1008",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 84941,
            "range": "± 1662",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 75941,
            "range": "± 2244",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1238880,
            "range": "± 21167",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2469528,
            "range": "± 36134",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 27253400,
            "range": "± 913364",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 161582161,
            "range": "± 2640630",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 310471135,
            "range": "± 4703593",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 18380,
            "range": "± 475",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8009,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 824,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 63,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 856,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2637,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8520,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23148,
            "range": "± 503",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 33829,
            "range": "± 726",
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
          "id": "59a5fb1a9f82894e1de487242d6345d1754c974d",
          "message": "Add SQL aggregates ANY, SOME, EVERY and their COLL_ versions (#360)",
          "timestamp": "2023-08-09T13:50:25-07:00",
          "tree_id": "2f54d7be47cfdedd96261f12fa14964e6c6481ac",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/59a5fb1a9f82894e1de487242d6345d1754c974d"
        },
        "date": 1691615037620,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 7834,
            "range": "± 558",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 71074,
            "range": "± 3802",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 136241,
            "range": "± 6296",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6513,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 48677,
            "range": "± 2129",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 101177,
            "range": "± 5482",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 119503,
            "range": "± 10197",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1906137,
            "range": "± 79031",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 3853933,
            "range": "± 174246",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 30976556,
            "range": "± 1479521",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 177541867,
            "range": "± 5345974",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 342430723,
            "range": "± 8403296",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 18952,
            "range": "± 802",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 8442,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 889,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 1178,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3782,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 12762,
            "range": "± 477",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 32400,
            "range": "± 1383",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 47327,
            "range": "± 2621",
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
          "id": "2cbc7a436c906ea733126a8cbfb67cd38fb0b270",
          "message": "Variable resolution and dynamic lookup fixes (#416)",
          "timestamp": "2023-08-10T11:13:47-07:00",
          "tree_id": "106e8d75615b16f993d46c2d9d7c41f6cff30e52",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/2cbc7a436c906ea733126a8cbfb67cd38fb0b270"
        },
        "date": 1691691894874,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5157,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 48828,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 96074,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5648,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 43348,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 88223,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 64145,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1024578,
            "range": "± 7511",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2069287,
            "range": "± 21796",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 26153308,
            "range": "± 892960",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 142023118,
            "range": "± 360284",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 271506212,
            "range": "± 351611",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15686,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 3953,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 732,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 730,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2325,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7589,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20550,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29975,
            "range": "± 63",
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
          "id": "d93dfe72cc61874c8e95aad921a340c9b4ca0998",
          "message": "Fixes to ORDER BY name resolution and null ordering bug (#418)\n\nCo-authored-by: Josh Pschorr <josh@pschorr.dev>\r\n\r\n* Apply readability suggestion from Josh",
          "timestamp": "2023-08-10T11:54:48-07:00",
          "tree_id": "2c97c30b7c93f6a98c6693f0488570ba9f3bfdcd",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/d93dfe72cc61874c8e95aad921a340c9b4ca0998"
        },
        "date": 1691694340500,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5274,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 48677,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 96312,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5704,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 43733,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 88337,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 68312,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1078230,
            "range": "± 1377",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2161453,
            "range": "± 4788",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22792114,
            "range": "± 316601",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 139040080,
            "range": "± 514338",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 266873798,
            "range": "± 254469",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15490,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 3938,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 743,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 735,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2221,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7306,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20127,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29077,
            "range": "± 64",
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
          "id": "3dc07732ff5f9615bbb3cd1bdf4560064570c3b0",
          "message": "Improvements to Value iterators (#422)",
          "timestamp": "2023-08-14T13:25:19-07:00",
          "tree_id": "4a46ad6b5b6c755c2cf9663b92bf81c70316face",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/3dc07732ff5f9615bbb3cd1bdf4560064570c3b0"
        },
        "date": 1692045380973,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5328,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 48913,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 96644,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5552,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 43153,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 88649,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67748,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1065888,
            "range": "± 2343",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2135651,
            "range": "± 8123",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23319471,
            "range": "± 648698",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 140944323,
            "range": "± 840104",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 270431546,
            "range": "± 433155",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15574,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 3819,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 786,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 729,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2500,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7840,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20152,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29667,
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
          "id": "3efe446507f36f564da65ec03c917d2b58809c90",
          "message": "Fix list/bag/tuple deep equality (#421)",
          "timestamp": "2023-08-14T13:35:47-07:00",
          "tree_id": "9864c8bb89579541615f13a54579136391eb2170",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/3efe446507f36f564da65ec03c917d2b58809c90"
        },
        "date": 1692045991460,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5716,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50472,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 100288,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5761,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 43603,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 89467,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 63202,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1016684,
            "range": "± 5193",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2009568,
            "range": "± 16907",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23202291,
            "range": "± 1010340",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 148623265,
            "range": "± 608342",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 290478385,
            "range": "± 623157",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 14793,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 4116,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 737,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 747,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2744,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8312,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20505,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29329,
            "range": "± 97",
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
          "id": "154bd5934aa2834ad3ee73089d3562661640a233",
          "message": "Change `BindingsName` to use `Cow<str>` to reduce clones (#423)",
          "timestamp": "2023-08-15T14:04:58-07:00",
          "tree_id": "a2bb2bacdbe705a896c86eb300817383b2514025",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/154bd5934aa2834ad3ee73089d3562661640a233"
        },
        "date": 1692134238232,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6512,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 59284,
            "range": "± 2341",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 117615,
            "range": "± 3688",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6719,
            "range": "± 254",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 53797,
            "range": "± 1177",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 112463,
            "range": "± 4532",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 118062,
            "range": "± 3615",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1832647,
            "range": "± 29492",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 3696231,
            "range": "± 115768",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 26287500,
            "range": "± 885584",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 140993400,
            "range": "± 2768574",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 272181087,
            "range": "± 3938503",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15122,
            "range": "± 619",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 4307,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 720,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 895,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 3192,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 10145,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 25121,
            "range": "± 555",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 37177,
            "range": "± 829",
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
          "id": "de1ac1b8ddd1f3b3fb9d0db5d1548557590a3bea",
          "message": "Add Catalog to NameResolver (#425)\n\nDuring the work for adding a steel thread for PartiQL Typing (Recent PR #389)\r\nand after merging #410 to `feat-type-plan-poc` it is realized that we need to\r\nrefactor the code to remove `DynamicLocalup` `VarExpr` with the assumption that\r\nwe work based off of Typing and Value Environment from the Catalog. We have a\r\nTyping Environment in the Catalog at the moment and we are going to add the\r\nVariable Environment as well. In preparation for such task, we need to make\r\nthe `NameResolver` Catalog aware. In that regard this commit adds the `Catalog`\r\nto `NameResolver`\r\n\r\nExpecting subsequent PR(s) for the name resolving using the Catalog.",
          "timestamp": "2023-08-16T13:34:17-07:00",
          "tree_id": "8d7c2b8694104625663f3b73b044eb36b51b071d",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/de1ac1b8ddd1f3b3fb9d0db5d1548557590a3bea"
        },
        "date": 1692218783810,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 6379,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 58268,
            "range": "± 481",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 115399,
            "range": "± 528",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6697,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 51118,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 104724,
            "range": "± 870",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 79903,
            "range": "± 462",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1272265,
            "range": "± 10271",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2546752,
            "range": "± 14940",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 25896239,
            "range": "± 537544",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 137386600,
            "range": "± 603876",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 260612406,
            "range": "± 1517263",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16234,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 4528,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 704,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 63,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 889,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2899,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9043,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 24231,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 35402,
            "range": "± 93",
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
          "id": "d6605c1382b62406fa47d271954dfd0f0533b6ee",
          "message": "Lower COUNT(*) to logical plan; fix multiple aggregations bug (#429)",
          "timestamp": "2023-08-18T12:51:50-07:00",
          "tree_id": "931bfc1250494e601de28b2c5ce39b35befceb27",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/d6605c1382b62406fa47d271954dfd0f0533b6ee"
        },
        "date": 1692388952376,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse-1",
            "value": 5409,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 48956,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 96770,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5692,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 43059,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 88087,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67387,
            "range": "± 666",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1069101,
            "range": "± 1562",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2140615,
            "range": "± 3910",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22645131,
            "range": "± 429503",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 113245438,
            "range": "± 616613",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 215764789,
            "range": "± 315069",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 13594,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 3765,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 582,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 739,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2440,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7669,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20223,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29422,
            "range": "± 132",
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
          "id": "1a6a1a43dba1f7215cb3e5ed6a65aa1ae2c6ac91",
          "message": "Add benchmarks for aggregation functions, group by, group as (#430)",
          "timestamp": "2023-08-24T11:53:06-07:00",
          "tree_id": "4b0d328584c5c0f78c9af1e5a379eaf2a0e2c78e",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/1a6a1a43dba1f7215cb3e5ed6a65aa1ae2c6ac91"
        },
        "date": 1692904124884,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 1803318,
            "range": "± 127316",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 2388863,
            "range": "± 142607",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 2241127,
            "range": "± 187920",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 2481754,
            "range": "± 220846",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 2157459,
            "range": "± 126362",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 2402939,
            "range": "± 141378",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 2270198,
            "range": "± 145426",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 2447025,
            "range": "± 127175",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 2290416,
            "range": "± 152097",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 2484900,
            "range": "± 185996",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 3062016,
            "range": "± 171164",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 4449609,
            "range": "± 304600",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 5337599,
            "range": "± 398493",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 4443412,
            "range": "± 252696",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 6320931,
            "range": "± 383103",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 7023955,
            "range": "± 430847",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5949,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 54566,
            "range": "± 3378",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 107606,
            "range": "± 6246",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6112,
            "range": "± 573",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 48980,
            "range": "± 2783",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 101774,
            "range": "± 5753",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 106797,
            "range": "± 6023",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1640451,
            "range": "± 80458",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 3447426,
            "range": "± 205586",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 20710877,
            "range": "± 1328457",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 116748951,
            "range": "± 6539488",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 228366702,
            "range": "± 9668219",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 13165,
            "range": "± 1244",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 3968,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 635,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 889,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2588,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8248,
            "range": "± 812",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 22919,
            "range": "± 1794",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 33398,
            "range": "± 2063",
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
          "id": "a5eb149807dc08425496c4dd68149eb2e18abf9a",
          "message": "Add tests for scanning of binary ion and zst-compressed binary ion (#432)",
          "timestamp": "2023-08-28T13:42:42-07:00",
          "tree_id": "f74c611ef906f1689f5aea653ede44bef66416b0",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/a5eb149807dc08425496c4dd68149eb2e18abf9a"
        },
        "date": 1693256250076,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 1618594,
            "range": "± 2166",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 2215492,
            "range": "± 2110",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 1952543,
            "range": "± 11971",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 2202013,
            "range": "± 92376",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 1974547,
            "range": "± 4667",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 2220275,
            "range": "± 3188",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 1975071,
            "range": "± 2326",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 2223568,
            "range": "± 4052",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 1968370,
            "range": "± 2562",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 2215338,
            "range": "± 8573",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 2815560,
            "range": "± 4095",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 3927626,
            "range": "± 27203",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 4663026,
            "range": "± 21839",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 4001914,
            "range": "± 15759",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 5590859,
            "range": "± 51595",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 6343465,
            "range": "± 58104",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5599,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51435,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 99982,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6061,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 44463,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 90825,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 62069,
            "range": "± 1510",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 989829,
            "range": "± 6580",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 1988008,
            "range": "± 16897",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 20290237,
            "range": "± 324516",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 122818710,
            "range": "± 4594789",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 238222689,
            "range": "± 256006",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 12877,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 3832,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 604,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 815,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2554,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7754,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20266,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29475,
            "range": "± 176",
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
          "id": "2db8651c9455bada6dbef08662b2dbc3faeba0b0",
          "message": "Refactor `Debug` output of evaluator for better readability. (#433)",
          "timestamp": "2023-08-28T14:36:17-07:00",
          "tree_id": "8e2d87f1313e40c9a63a448175ede74b10724536",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/2db8651c9455bada6dbef08662b2dbc3faeba0b0"
        },
        "date": 1693259551483,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 1992345,
            "range": "± 7182",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 2642546,
            "range": "± 12751",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 2332317,
            "range": "± 7437",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 2632981,
            "range": "± 8110",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 2365517,
            "range": "± 2877",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 2662147,
            "range": "± 5239",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 2366135,
            "range": "± 3353",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 2663915,
            "range": "± 5457",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 2362641,
            "range": "± 5476",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 2654478,
            "range": "± 7334",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 3458078,
            "range": "± 11480",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 4696561,
            "range": "± 27468",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 5625056,
            "range": "± 25311",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 4885964,
            "range": "± 6868",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 6679151,
            "range": "± 51910",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 7583479,
            "range": "± 48540",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6373,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 58927,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 114711,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6840,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 51211,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 104299,
            "range": "± 443",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 78501,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1257240,
            "range": "± 2633",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2509255,
            "range": "± 18289",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 24336430,
            "range": "± 596622",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 132698103,
            "range": "± 313957",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 250769075,
            "range": "± 413233",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 16260,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 4373,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 716,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 868,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2691,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8597,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23772,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34762,
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
          "id": "4daa1e41803a79cb52be01146de88b69fdffd11b",
          "message": "Refactor `EvalGroupBy` and Aggregations to avoid duplicate effort (#431)\n\n* to_vec on bag & list\r\n\r\n* AddAssign for Value\r\n\r\n* Refactor `EvalGroupBy` and Aggregations to avoid duplicate effort\r\n\r\n* Minor cleanup",
          "timestamp": "2023-08-28T15:25:32-07:00",
          "tree_id": "5f3735a36abe6e9a71e9c16a44889529d06a419e",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/4daa1e41803a79cb52be01146de88b69fdffd11b"
        },
        "date": 1693262514965,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 1213735,
            "range": "± 22093",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 1581541,
            "range": "± 28491",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 1516436,
            "range": "± 50220",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 1557340,
            "range": "± 22757",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 1518742,
            "range": "± 27828",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 1573404,
            "range": "± 15135",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 1529806,
            "range": "± 22172",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 1575506,
            "range": "± 27169",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 1520223,
            "range": "± 20404",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 1562930,
            "range": "± 23956",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 1846347,
            "range": "± 19726",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 2243907,
            "range": "± 18476",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 3273563,
            "range": "± 27406",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 2113337,
            "range": "± 33272",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 2518563,
            "range": "± 33910",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 3563818,
            "range": "± 68288",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6182,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 57398,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 113204,
            "range": "± 1850",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6640,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 50906,
            "range": "± 619",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 104252,
            "range": "± 1619",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 78877,
            "range": "± 765",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1253374,
            "range": "± 14817",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2484789,
            "range": "± 31856",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23474659,
            "range": "± 656591",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 130419005,
            "range": "± 1592997",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 248083342,
            "range": "± 2242372",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15941,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 4249,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 705,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 950,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2818,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 9307,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 25069,
            "range": "± 370",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 36302,
            "range": "± 752",
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
          "id": "14f6e01c2cc22149dbeed4c5329b15f3e55b950f",
          "message": "Add Catalog to resolve_varref in lowering (#434)\n\nCurrently during lowering, when we want to resolve a variable name we do the following:\r\n\r\n1. If there are any `KeySchema`s  in `KeyRegistry` for the current `id` in the stack (lookup from bottom of the stack to the top)\r\n2. Retrieve the `KeySchema`.\r\n3. Lookup the `scopes` in `KeyRegistry` and see which `NodeId` this `Node` is within the scope of.\r\n4. Get the `KeySchema` of the found `Node`.\r\n5. If the variable name is equal to the `KeySchema`'s produce OR lowered case variable name is equal to `produce`'s lowered case and variable name is case-insensitive, then create a new `VarRef` with `Local` lookup and push it to the lookups (for `DynamicLookup`)\r\n6. else create a `Path` with the `produce` name as `VarRef` and variable name as the key component.\r\n\r\n\r\nThis PR adds a step between 5 and 6 to check to see if there is a type for the variable in the global and if so, adds a `VarRef` resolve global to the lookups.\r\n\r\n### Example\r\n**Query**: `SELECT c.id, customers.name FROM customers AS c`;\r\n**Global Typing Environment (in Catalog)**:  <<`customers`: any>>\r\n\r\n**KeyRegistry**:\r\n\r\n``` rust\r\n{\r\n    in_scope: {\r\n        NodeId(\r\n            12, // From\r\n        ): [\r\n            NodeId(\r\n                11, // FromLet\r\n            ),\r\n        ],\r\n        NodeId(\r\n            13, // Select\r\n        ): [\r\n            NodeId(\r\n                11, // FromLet\r\n            ),\r\n        ],\r\n        NodeId(\r\n            14, // QuerySet\r\n        ): [\r\n            NodeId(\r\n                11, // FromLet\r\n            ),\r\n        ],\r\n        NodeId(\r\n            15, // Query\r\n        ): [\r\n            NodeId(\r\n                11, // FromLet\r\n            ),\r\n        ],\r\n        NodeId(\r\n            16, // TopLevel\r\n        ): [\r\n            NodeId(\r\n                11, // FromLet\r\n            ),\r\n        ],\r\n    },\r\n    schema: {\r\n        NodeId(\r\n            11,\r\n        ): KeySchema {\r\n            consume: {\r\n                NameRef {\r\n                    sym: SymbolPrimitive {\r\n                        value: \"customers\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                    lookup: [\r\n                        Global,\r\n                        Local,\r\n                    ],\r\n                },\r\n            },\r\n            produce: {\r\n                Known(\r\n                    SymbolPrimitive {\r\n                        value: \"c\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                ),\r\n            },\r\n        },\r\n        NodeId(\r\n            15,\r\n        ): KeySchema {\r\n            consume: {\r\n                NameRef {\r\n                    sym: SymbolPrimitive {\r\n                        value: \"c\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                    lookup: [\r\n                        Local,\r\n                        Global,\r\n                    ],\r\n                },\r\n                NameRef {\r\n                    sym: SymbolPrimitive {\r\n                        value: \"id\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                    lookup: [\r\n                        Local,\r\n                        Global,\r\n                    ],\r\n                },\r\n                NameRef {\r\n                    sym: SymbolPrimitive {\r\n                        value: \"customers\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                    lookup: [\r\n                        Local,\r\n                        Global,\r\n                    ],\r\n                },\r\n                NameRef {\r\n                    sym: SymbolPrimitive {\r\n                        value: \"name\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                    lookup: [\r\n                        Local,\r\n                        Global,\r\n                    ],\r\n                },\r\n            },\r\n            produce: {\r\n                Known(\r\n                    SymbolPrimitive {\r\n                        value: \"my_id\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                ),\r\n                Known(\r\n                    SymbolPrimitive {\r\n                        value: \"my_name\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                ),\r\n            },\r\n        },\r\n    },\r\n    aliases: {\r\n        NodeId(\r\n            4,\r\n        ): Known(\r\n            SymbolPrimitive {\r\n                value: \"my_id\",\r\n                case: CaseInsensitive,\r\n            },\r\n        ),\r\n        NodeId(\r\n            8,\r\n        ): Known(\r\n            SymbolPrimitive {\r\n                value: \"my_name\",\r\n                case: CaseInsensitive,\r\n            },\r\n        ),\r\n        NodeId(\r\n            11,\r\n        ): Known(\r\n            SymbolPrimitive {\r\n                value: \"c\",\r\n                case: CaseInsensitive,\r\n            },\r\n        ),\r\n    },\r\n}\r\n```\r\n\r\n\r\nFor resolving `customers` in the projection, we find `NodeId` `15` (`Query` in the `AST`) and get the following `KeySchema` from the `KeyRegistry`:\r\n\r\n```rust\r\nKeySchema {\r\n            consume: {\r\n                NameRef {\r\n                    sym: SymbolPrimitive {\r\n                        value: \"c\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                    lookup: [\r\n                        Local,\r\n                        Global,\r\n                    ],\r\n                },\r\n                NameRef {\r\n                    sym: SymbolPrimitive {\r\n                        value: \"id\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                    lookup: [\r\n                        Local,\r\n                        Global,\r\n                    ],\r\n                },\r\n                NameRef {\r\n                    sym: SymbolPrimitive {\r\n                        value: \"customers\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                    lookup: [\r\n                        Local,\r\n                        Global,\r\n                    ],\r\n                },\r\n                NameRef {\r\n                    sym: SymbolPrimitive {\r\n                        value: \"name\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                    lookup: [\r\n                        Local,\r\n                        Global,\r\n                    ],\r\n                },\r\n            },\r\n            produce: {\r\n                Known(\r\n                    SymbolPrimitive {\r\n                        value: \"my_id\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                ),\r\n                Known(\r\n                    SymbolPrimitive {\r\n                        value: \"my_name\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                ),\r\n            },\r\n        },\r\n\r\n```\r\n\r\nWe retrieve the following `scope` which says `15` is in the scope of `11`:\r\n\r\n```rust\r\n        NodeId(\r\n            15, // Query\r\n        ): [\r\n            NodeId(\r\n                11, // FromLet\r\n            ),\r\n        ],\r\n```\r\n\r\nWe get the following `KeySchema` for `11`:\r\n\r\n```rust\r\nKeySchema {\r\n            consume: {\r\n                NameRef {\r\n                    sym: SymbolPrimitive {\r\n                        value: \"customers\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                    lookup: [\r\n                        Global,\r\n                        Local,\r\n                    ],\r\n                },\r\n            },\r\n            produce: {\r\n                Known(\r\n                    SymbolPrimitive {\r\n                        value: \"c\",\r\n                        case: CaseInsensitive,\r\n                    },\r\n                ),\r\n            },\r\n        }\r\n```\r\n\r\nWe check the `produce` and it is unequal to `customers`. We then check the global catalog and we find an entry for `customers`, hence we add a `VarRef` with a `Global` lookup to the look-ups.",
          "timestamp": "2023-08-29T15:28:24-07:00",
          "tree_id": "b3636e60e29c4a2ca388af471facb7d9a09c51a8",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/14f6e01c2cc22149dbeed4c5329b15f3e55b950f"
        },
        "date": 1693349088014,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 1213220,
            "range": "± 12508",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 1572893,
            "range": "± 16767",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 1522868,
            "range": "± 23537",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 1559692,
            "range": "± 22208",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 1521911,
            "range": "± 21203",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 1579583,
            "range": "± 17113",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 1550883,
            "range": "± 8125",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 1578388,
            "range": "± 16666",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 1517316,
            "range": "± 24189",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 1557248,
            "range": "± 31386",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 1851214,
            "range": "± 26914",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 2214797,
            "range": "± 57988",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 3307202,
            "range": "± 158680",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 2092775,
            "range": "± 81306",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 2505991,
            "range": "± 37844",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 3488942,
            "range": "± 42833",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6871,
            "range": "± 532",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 65207,
            "range": "± 33597",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 122542,
            "range": "± 4017",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6536,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 51136,
            "range": "± 1518",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 110807,
            "range": "± 7807",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 77594,
            "range": "± 1283",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1238399,
            "range": "± 60424",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2444791,
            "range": "± 68665",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23777220,
            "range": "± 782446",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 132054690,
            "range": "± 948550",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 247631799,
            "range": "± 4043217",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15542,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 4204,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 682,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 52,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 920,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2630,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8681,
            "range": "± 539",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23262,
            "range": "± 466",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 33925,
            "range": "± 968",
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
          "id": "fbaac0fe030da669828e3f49123374706a7e20eb",
          "message": "Type Simple SFW queries using current `LogicalPlan` as an `IR` (#399)\n\nRelates to: https://github.com/partiql/partiql-spec/issues/49\r\n\r\nAdds the code for typing a query using the current `LogicalPlan` as an `IR`. This PR includes the changes to enable simple SFW queries as the following in both `Strict` and `Permissive` typing modes with `Open` and `Closed` schemas.\r\n\r\n#### Closed schema with `Strict` typing mode.\r\n```SQL\r\n-- customers: <<{'id': INT, 'name': STRING, age: ANY}>>\r\nSELECT customers.id, customers.name FROM customers;\r\n\r\n-- Output schema: <<{'id': INT, 'name': STRING}>>\r\n```\r\n\r\n#### Open schema with `Strict` typing mode and `age` non-existent projection.\r\n```SQL\r\n\r\n-- customers: <<{'id': INT, 'name': STRING}>>\r\nSELECT customers.id, customers.name, customers.age FROM customers;\r\n\r\n-- Output schema: <<{'id': INT, 'name': STRING, 'age': ANY}>>\r\n```\r\n\r\n#### Closed Schema with `Permissive` typing mode and `age` non-existent projection.\r\n```SQL\r\n-- customers: <<{'id': INT, 'name': STRING>>\r\nSELECT customers.id, customers.name, customers.age FROM customers;\r\n\r\n\r\n-- Output schema: <<{'id': INT, 'name': STRING, 'age': NULL}>>\r\n```\r\n\r\nSee: https://github.com/partiql/partiql-spec/discussions/64\r\n\r\n#### Open Schema with `Strict` typing mode and `age` in nested attribute.\r\n```SQL\r\n-- customers: <<{'id': INT, 'name': STRING, 'details': {'age': INT}}>>\r\nSELECT customers.id, customers.name, customers.details.age FROM customers;\r\n\r\n\r\n-- Output schema: <<{'id': INT, 'name': STRING, 'age': INT}>>\r\n```\r\n\r\n#### Open Schema (`customers and details`) with `Strict` typing mode.\r\n```SQL\r\n-- customers: <<{'id': INT, 'name': STRING, 'details': {'age': INT}}>>\r\nSELECT customers.id, customers.name, customers.details.age, customers.details.foo.bar FROM customers;\r\n\r\n\r\n-- Output schema: <<{'id': INT, 'name': STRING, 'age': INT, 'bar': ANY}>>\r\n```\r\n\r\n#### Open Schema (`customers and details`)  with `Strict` typing mode and `age` in nested attribute with Path in `FROM` with alias.\r\n```SQL\r\n-- customers: <<{'id': INT, 'name': STRING, 'details': {'age': INT}}>>\r\nSELECT d.age FROM customers.details AS d;\r\n\r\n\r\n-- Output schema: <<{'age': INT}>>\r\n```\r\n\r\nSee: https://github.com/partiql/partiql-spec/discussions/65\r\n\r\n#### Closed Schema with `Strict` typing mode with `FROM` and `Projection` aliases.\r\n```SQL\r\n-- customers: <<{'id': INT, 'name': STRING, 'age': ANY}>>\r\nSELECT c.id AS my_id, customers.name AS my_name FROM customers AS c;\r\n\r\n\r\n-- Output schema: <<{'my_id': INT, 'my_name': STRING}>>\r\n```\r\n\r\nIn addition:\r\n- fixes some issues with the current `partiql_types` macros, changes the model for some of the types, and adds some helper methods to `PartiqlType`.\r\n- moves constraints to set\r\n- uses ` BTreeSet` for constraints and `AnyOf` to be able to support ordering on the these types; since we're not expecting large number of elements for these types the performance hit seems negligible. \r\n\r\nUpon merging this commit we need to add:\r\n- Support for operators (E.g., unary, binary).\r\n- Support for the rest of the Binding Operators (E.g., `Filter`, `OrderBy`).\r\n- Support for functions.\r\n- Related Conformance Tests.",
          "timestamp": "2023-09-14T16:03:50-07:00",
          "tree_id": "0f6100c9aa92a5b6a54e77b723130399df913418",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/fbaac0fe030da669828e3f49123374706a7e20eb"
        },
        "date": 1694733604948,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 1212926,
            "range": "± 19035",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 1582179,
            "range": "± 27164",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 1511903,
            "range": "± 14718",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 1533225,
            "range": "± 31524",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 1488384,
            "range": "± 19863",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 1553106,
            "range": "± 17910",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 1524643,
            "range": "± 56117",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 1560447,
            "range": "± 21059",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 1523670,
            "range": "± 12597",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 1576659,
            "range": "± 13738",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 1851634,
            "range": "± 8243",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 2244821,
            "range": "± 29424",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 3273668,
            "range": "± 36373",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 2096917,
            "range": "± 21530",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 2501800,
            "range": "± 30606",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 3480626,
            "range": "± 39961",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6232,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 59143,
            "range": "± 912",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 118572,
            "range": "± 1759",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 6634,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 50958,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 103772,
            "range": "± 924",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 79923,
            "range": "± 1133",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1281174,
            "range": "± 23005",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2552074,
            "range": "± 36714",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 23547100,
            "range": "± 629248",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 128954574,
            "range": "± 1564430",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 245984294,
            "range": "± 3999203",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 15317,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 4070,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 685,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 62,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 937,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2832,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8551,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 23275,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 34548,
            "range": "± 779",
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
          "id": "57ce9b80172923173a09d4c3e10565f7ebf796fd",
          "message": "Fix clippy warnings/errors (#436)",
          "timestamp": "2023-10-26T13:16:59-07:00",
          "tree_id": "8246a772d3a6c6e16bad41adc3665bbc68de12f1",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/57ce9b80172923173a09d4c3e10565f7ebf796fd"
        },
        "date": 1698352350577,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 1010917,
            "range": "± 28031",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 1300979,
            "range": "± 2728",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 1251936,
            "range": "± 5916",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 1293721,
            "range": "± 4954",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 1259162,
            "range": "± 5519",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 1298623,
            "range": "± 4017",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 1265816,
            "range": "± 8128",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 1307918,
            "range": "± 2428",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 1258395,
            "range": "± 6754",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 1298318,
            "range": "± 5421",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 1517685,
            "range": "± 3595",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1877847,
            "range": "± 5923",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 2707518,
            "range": "± 5247",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1754770,
            "range": "± 9772",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 2109893,
            "range": "± 12878",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2951261,
            "range": "± 8525",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5530,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50738,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 99995,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5551,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 43082,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 88331,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 64531,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1039325,
            "range": "± 6144",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2079499,
            "range": "± 3489",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 22028250,
            "range": "± 578575",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 114887145,
            "range": "± 776530",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 214497420,
            "range": "± 513872",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 13429,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 3583,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 579,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 736,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2234,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7536,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20279,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 29654,
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
          "id": "6ef3177e3be2fb172c2702da3d5b93c946d2bb0e",
          "message": "chore: Release v0.6.0 (#439)",
          "timestamp": "2023-11-01T13:17:01-07:00",
          "tree_id": "575459de226b6ee2c2bcf4a370e23caf8ff90bab",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/6ef3177e3be2fb172c2702da3d5b93c946d2bb0e"
        },
        "date": 1698870718277,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 941419,
            "range": "± 73644",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 1276486,
            "range": "± 55450",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 1264891,
            "range": "± 86620",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 1266475,
            "range": "± 43908",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 1249892,
            "range": "± 46994",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 1283726,
            "range": "± 104311",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 1262569,
            "range": "± 55099",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 1325749,
            "range": "± 98187",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 1266184,
            "range": "± 60314",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 1270357,
            "range": "± 63684",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 1505602,
            "range": "± 63925",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1807514,
            "range": "± 68759",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 2791700,
            "range": "± 106288",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1763977,
            "range": "± 104833",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 2069455,
            "range": "± 65747",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 3011931,
            "range": "± 92495",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5885,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 48940,
            "range": "± 3352",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 96015,
            "range": "± 5566",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 5179,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 42074,
            "range": "± 8757",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 87148,
            "range": "± 2559",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 96812,
            "range": "± 4347",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1470510,
            "range": "± 62830",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2904917,
            "range": "± 113963",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 19178617,
            "range": "± 1710894",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 101956959,
            "range": "± 4930025",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 197651181,
            "range": "± 6742894",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 11551,
            "range": "± 497",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 3252,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 529,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 41,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 740,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2293,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7491,
            "range": "± 454",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20031,
            "range": "± 1140",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 30696,
            "range": "± 1783",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rajiv.ranganath@gmail.com",
            "name": "Rajiv M Ranganath",
            "username": "rajivr"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "febf4545004f13d5f462aa4b88312cf0382e2208",
          "message": "partiql-visualize-dot: add (#438)",
          "timestamp": "2023-11-08T11:03:24-08:00",
          "tree_id": "0abffca6d83296c66b031ef23c420b62b9506547",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/febf4545004f13d5f462aa4b88312cf0382e2208"
        },
        "date": 1699470899549,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 774930,
            "range": "± 1405",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 863740,
            "range": "± 2341",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 821089,
            "range": "± 31437",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 865003,
            "range": "± 3352",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 824792,
            "range": "± 4183",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 862073,
            "range": "± 3156",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 831647,
            "range": "± 2214",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 873358,
            "range": "± 5289",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 824877,
            "range": "± 12048",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 862124,
            "range": "± 25791",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 975755,
            "range": "± 4692",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1283533,
            "range": "± 17156",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1900087,
            "range": "± 17407",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1312139,
            "range": "± 27719",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1668723,
            "range": "± 12031",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2273825,
            "range": "± 134302",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4954,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 43532,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 85169,
            "range": "± 487",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4378,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32549,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 65733,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 65793,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1021480,
            "range": "± 10077",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2041544,
            "range": "± 43822",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 14225792,
            "range": "± 55566",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 91904584,
            "range": "± 599657",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 175253146,
            "range": "± 1129796",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10587,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2920,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 485,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 646,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2107,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 6721,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 17146,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 23904,
            "range": "± 165",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rajiv.ranganath@atihita.com",
            "name": "Rajiv M Ranganath",
            "username": "rajivr"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3374e624254978d815abfd1ddcef82322ca89aab",
          "message": "partiql-value: single quote tuple attribute (#442)",
          "timestamp": "2024-01-30T17:22:54-08:00",
          "tree_id": "dee131936810e8784904ae3e28dba258e6626212",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/3374e624254978d815abfd1ddcef82322ca89aab"
        },
        "date": 1706664871856,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 763583,
            "range": "± 3390",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 848959,
            "range": "± 3747",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 804209,
            "range": "± 29915",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 841379,
            "range": "± 2663",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 811458,
            "range": "± 12162",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 848553,
            "range": "± 2675",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 822782,
            "range": "± 2645",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 859345,
            "range": "± 2350",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 811349,
            "range": "± 3125",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 849049,
            "range": "± 4051",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 959853,
            "range": "± 3200",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1257292,
            "range": "± 8831",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1809821,
            "range": "± 24568",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1226070,
            "range": "± 12413",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1562560,
            "range": "± 8900",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2142416,
            "range": "± 9792",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4249,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 38868,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 75578,
            "range": "± 10744",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4355,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32542,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 66497,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 65369,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1016409,
            "range": "± 38498",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2052512,
            "range": "± 13028",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13391126,
            "range": "± 150215",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85921655,
            "range": "± 2375249",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 164438426,
            "range": "± 2221754",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9623,
            "range": "± 492",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2451,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 437,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 58,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 624,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1802,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5628,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14538,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21356,
            "range": "± 70",
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
          "id": "05d3fc827c66e483d7d2520c59284d725b81f2d5",
          "message": "Allow ORDER BY to 'see' projection names (#443)",
          "timestamp": "2024-02-07T10:09:18-08:00",
          "tree_id": "8db4af376f5ad2efe0736f93ea3ced7de98afafd",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/05d3fc827c66e483d7d2520c59284d725b81f2d5"
        },
        "date": 1707330097701,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 762950,
            "range": "± 5108",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 848481,
            "range": "± 10112",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 806134,
            "range": "± 14060",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 841558,
            "range": "± 3359",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 814702,
            "range": "± 5147",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 845150,
            "range": "± 23113",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 819597,
            "range": "± 5641",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 857070,
            "range": "± 8162",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 811700,
            "range": "± 4935",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 845651,
            "range": "± 3480",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 964272,
            "range": "± 8929",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1208187,
            "range": "± 11106",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1810359,
            "range": "± 20019",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1245624,
            "range": "± 10761",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1537157,
            "range": "± 21402",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2109521,
            "range": "± 15943",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4302,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39811,
            "range": "± 222",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 79992,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4462,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32043,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 64769,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 65654,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1020884,
            "range": "± 27825",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2042760,
            "range": "± 17083",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13009758,
            "range": "± 289446",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 86227676,
            "range": "± 929942",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 165768763,
            "range": "± 809202",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9723,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2498,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 435,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 618,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1890,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5822,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14946,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21988,
            "range": "± 77",
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
          "id": "00924f27474a25530c8d2f53853956ae3fc2f58d",
          "message": "Minor cleanup and formatting fixes (#445)",
          "timestamp": "2024-02-07T10:28:23-08:00",
          "tree_id": "d64cfa7a01f21a741ff13119b9cc7bf42b668b63",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/00924f27474a25530c8d2f53853956ae3fc2f58d"
        },
        "date": 1707331237334,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 754554,
            "range": "± 4322",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 844531,
            "range": "± 2241",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 797390,
            "range": "± 13786",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 829564,
            "range": "± 6228",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 804063,
            "range": "± 2274",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 839403,
            "range": "± 2762",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 810532,
            "range": "± 3029",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 846513,
            "range": "± 2781",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 808493,
            "range": "± 2023",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 838090,
            "range": "± 2608",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 952146,
            "range": "± 5957",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1193797,
            "range": "± 14549",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1780324,
            "range": "± 17197",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1234325,
            "range": "± 21454",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1509448,
            "range": "± 19471",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2093994,
            "range": "± 19080",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4277,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39387,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 77794,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4404,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32097,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 65060,
            "range": "± 486",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 65323,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1023202,
            "range": "± 23909",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2038989,
            "range": "± 5575",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12530654,
            "range": "± 145755",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 86425714,
            "range": "± 1706273",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 165648119,
            "range": "± 775470",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9722,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2432,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 434,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 577,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1776,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5726,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14529,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21178,
            "range": "± 656",
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
          "id": "4d9ae54d84bd5184ce59f8afee3ce9199092cd51",
          "message": "Add session context to evaluation (#446)",
          "timestamp": "2024-03-07T10:51:18-08:00",
          "tree_id": "534d9a694ac70900593171c502ff4f43a7696fd3",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/4d9ae54d84bd5184ce59f8afee3ce9199092cd51"
        },
        "date": 1709838198350,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 750597,
            "range": "± 7488",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 836315,
            "range": "± 3793",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 793759,
            "range": "± 19048",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 826980,
            "range": "± 4937",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 796934,
            "range": "± 3103",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 832026,
            "range": "± 9211",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 803358,
            "range": "± 2992",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 843917,
            "range": "± 2002",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 799575,
            "range": "± 23513",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 832584,
            "range": "± 13265",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 935970,
            "range": "± 2693",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1166138,
            "range": "± 17416",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1757688,
            "range": "± 17890",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1267440,
            "range": "± 16676",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1532572,
            "range": "± 102361",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2101194,
            "range": "± 18999",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4189,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39717,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 79927,
            "range": "± 1192",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4182,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30759,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 62767,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67023,
            "range": "± 672",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1044691,
            "range": "± 28724",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2085981,
            "range": "± 15790",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12576519,
            "range": "± 152058",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85195131,
            "range": "± 1406453",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 164167304,
            "range": "± 410291",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9793,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2513,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 442,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 567,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1755,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 6093,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 15229,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21704,
            "range": "± 729",
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
          "id": "a177f63dfa7e8fe9c43b6eb1ac11b289adfd52f0",
          "message": "Implment additional unsigned integer `From` for partiql_value::Value (#449)\n\n* Add additional From for Value\r\n\r\nAdds the following:\r\n1. Value::from(u8)\r\n2. Value::from(u16)\r\n3. Value::from(u32)\r\n4. Value::from(u64)\r\n5. Value::from(u128)",
          "timestamp": "2024-03-11T16:38:30-07:00",
          "tree_id": "c5b7c4accdfc169980e547a382cf459705503190",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/a177f63dfa7e8fe9c43b6eb1ac11b289adfd52f0"
        },
        "date": 1710201042659,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 746821,
            "range": "± 19100",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 829264,
            "range": "± 3199",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 787844,
            "range": "± 12562",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 826939,
            "range": "± 2530",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 793890,
            "range": "± 7999",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 841550,
            "range": "± 4400",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 806244,
            "range": "± 5126",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 854146,
            "range": "± 2215",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 802239,
            "range": "± 13457",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 827343,
            "range": "± 3607",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 934828,
            "range": "± 10031",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1173523,
            "range": "± 15018",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1760710,
            "range": "± 18886",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1247320,
            "range": "± 25380",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1504635,
            "range": "± 7476",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2073905,
            "range": "± 10864",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4133,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 38969,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 76003,
            "range": "± 1038",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4404,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 31097,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63106,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 66885,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1043991,
            "range": "± 39924",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2094529,
            "range": "± 8733",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12975916,
            "range": "± 267403",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85761476,
            "range": "± 893565",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 164584010,
            "range": "± 859013",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9758,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2519,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 442,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 548,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1796,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5733,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14943,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21558,
            "range": "± 80",
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
          "id": "6e16561457a01fc31d077eaa9ad9702ab9db89f5",
          "message": "Add errors from BaseTableExpr's to the evaluator (#447)",
          "timestamp": "2024-03-12T11:11:04-07:00",
          "tree_id": "a2e479ddf76cfa646668724616dbdffd53d43746",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/6e16561457a01fc31d077eaa9ad9702ab9db89f5"
        },
        "date": 1710267794031,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 747809,
            "range": "± 34356",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 831777,
            "range": "± 5705",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 790076,
            "range": "± 17042",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 823566,
            "range": "± 23775",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 797208,
            "range": "± 2083",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 828043,
            "range": "± 1950",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 800288,
            "range": "± 8875",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 837043,
            "range": "± 18183",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 796913,
            "range": "± 21548",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 827024,
            "range": "± 4742",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 939603,
            "range": "± 16643",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1205820,
            "range": "± 12994",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1788912,
            "range": "± 13490",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1229401,
            "range": "± 21851",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1518219,
            "range": "± 40358",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2117193,
            "range": "± 92643",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4410,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 40403,
            "range": "± 1440",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 79352,
            "range": "± 554",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4458,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30702,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 62489,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 66573,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1041907,
            "range": "± 10191",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2086756,
            "range": "± 11988",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12654151,
            "range": "± 180220",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85231512,
            "range": "± 860320",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 163672275,
            "range": "± 4185849",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9886,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2496,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 443,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 579,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1984,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5956,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 15655,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 22266,
            "range": "± 329",
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
          "id": "af599b6c11a8c32a734ae28a7d64f6e0665f0b17",
          "message": "chore: Release v0.7.0 (#450)",
          "timestamp": "2024-03-12T12:45:13-07:00",
          "tree_id": "10d4ca6dd41f020b198041934f3e99ebbbf1bc09",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/af599b6c11a8c32a734ae28a7d64f6e0665f0b17"
        },
        "date": 1710273433934,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 744593,
            "range": "± 2615",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 835586,
            "range": "± 16234",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 789549,
            "range": "± 26162",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 828354,
            "range": "± 12721",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 794067,
            "range": "± 4478",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 838780,
            "range": "± 5130",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 799322,
            "range": "± 3331",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 854656,
            "range": "± 2088",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 794350,
            "range": "± 2987",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 831739,
            "range": "± 4740",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 937809,
            "range": "± 3483",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1184842,
            "range": "± 87490",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1734721,
            "range": "± 37235",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1211300,
            "range": "± 19877",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1479859,
            "range": "± 19086",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2052958,
            "range": "± 7398",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4287,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39497,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 79151,
            "range": "± 332",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4300,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 31105,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63780,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 66623,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1042232,
            "range": "± 22706",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2088509,
            "range": "± 10138",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12509924,
            "range": "± 73849",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 86717882,
            "range": "± 338709",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 167332440,
            "range": "± 2758053",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9716,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2466,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 438,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 570,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1774,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5618,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14686,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21070,
            "range": "± 186",
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
          "id": "6d31b395a4b6a4baca5489c7ad0fcbdb8c549867",
          "message": "Add #![deny(rust_2018_idioms)] to all crates (#451)\n\n* Add lint commands to deny lints that fail `rust_2018_idioms`\r\n    find . -iname lib.rs ! -path \"*target/*\" -exec sed -i '' -e '1s;^;#![deny(rust_2018_idioms)]\\n\\n;' {} +\r\n* Fix errors reported by lint `#![deny(rust_2018_idioms)]`",
          "timestamp": "2024-03-13T16:08:49-07:00",
          "tree_id": "e1edd1004dc163622699f87472bfb1650708e623",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/6d31b395a4b6a4baca5489c7ad0fcbdb8c549867"
        },
        "date": 1710372067503,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 749794,
            "range": "± 20334",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 844710,
            "range": "± 16836",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 794064,
            "range": "± 23842",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 827009,
            "range": "± 10296",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 803369,
            "range": "± 9148",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 848859,
            "range": "± 6799",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 803537,
            "range": "± 3943",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 859256,
            "range": "± 1644",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 801012,
            "range": "± 3588",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 834489,
            "range": "± 3412",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 943002,
            "range": "± 6682",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1180518,
            "range": "± 8179",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1794515,
            "range": "± 26653",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1279122,
            "range": "± 6998",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1572001,
            "range": "± 22089",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2158777,
            "range": "± 15168",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4175,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 40432,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 76469,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4263,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30966,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 62785,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 66750,
            "range": "± 1100",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1040907,
            "range": "± 10129",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2084147,
            "range": "± 9640",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12570554,
            "range": "± 179265",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85804492,
            "range": "± 867122",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 165209384,
            "range": "± 404633",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9670,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2464,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 436,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 568,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1799,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5878,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14743,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21472,
            "range": "± 347",
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
          "id": "f9ea25cb249b7c0588740b6ba528a44365ab54be",
          "message": "Add `#![deny(clippy::all)]` to all `lib.rs` (#453)\n\n* Add lint `#![deny(clippy::all)]`\r\n* Automatic fixes from `cargo clippy --fix --all-features; cargo fmt`\r\n* Clean-up based on some clippy recommendations",
          "timestamp": "2024-03-15T13:51:17-07:00",
          "tree_id": "24b675b3b411f14468889560c5f2a5f4ce68e21b",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/f9ea25cb249b7c0588740b6ba528a44365ab54be"
        },
        "date": 1710536606328,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 753295,
            "range": "± 10692",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 840324,
            "range": "± 4163",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 793350,
            "range": "± 10629",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 839384,
            "range": "± 2146",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 798513,
            "range": "± 6679",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 833475,
            "range": "± 2626",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 804812,
            "range": "± 46953",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 840423,
            "range": "± 3375",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 799199,
            "range": "± 31039",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 837469,
            "range": "± 3462",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 940946,
            "range": "± 3800",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1191733,
            "range": "± 24045",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1765004,
            "range": "± 15301",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1266530,
            "range": "± 6843",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1531211,
            "range": "± 18789",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2097034,
            "range": "± 9635",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4246,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39512,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 77971,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4275,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30620,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 62886,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 69165,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1069606,
            "range": "± 19098",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2148107,
            "range": "± 6347",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13395677,
            "range": "± 284023",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 87240379,
            "range": "± 742836",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 167428923,
            "range": "± 586993",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9691,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2511,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 431,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 539,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1762,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5689,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14800,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 20886,
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
          "id": "b6f73b70b2f0045b12e933d4c121ddae21326957",
          "message": " Fix subquery global binding lookup and error propagation (#454)",
          "timestamp": "2024-03-15T19:19:41-07:00",
          "tree_id": "24bb1196c765a53b8a9a512b041d5a5aee6f9333",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/b6f73b70b2f0045b12e933d4c121ddae21326957"
        },
        "date": 1710556311623,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 756669,
            "range": "± 7483",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 837761,
            "range": "± 2793",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 797658,
            "range": "± 15101",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 830209,
            "range": "± 5183",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 795669,
            "range": "± 2988",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 834058,
            "range": "± 2519",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 807020,
            "range": "± 1912",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 846442,
            "range": "± 2326",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 803699,
            "range": "± 3407",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 834747,
            "range": "± 2232",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 945625,
            "range": "± 2785",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1198499,
            "range": "± 21056",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1745326,
            "range": "± 17859",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1200656,
            "range": "± 13514",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1514836,
            "range": "± 27290",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2070089,
            "range": "± 7165",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4315,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39841,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 77570,
            "range": "± 1255",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4386,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 31381,
            "range": "± 968",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63121,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 69295,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1074600,
            "range": "± 12131",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2156016,
            "range": "± 3093",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12537278,
            "range": "± 261870",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85439293,
            "range": "± 841699",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 164229235,
            "range": "± 1538778",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9799,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2524,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 457,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 559,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1753,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5826,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14918,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21549,
            "range": "± 913",
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
          "id": "17886f3636635f4e272f5847afa164438b28bd18",
          "message": "chore: Release (#455)",
          "timestamp": "2024-03-15T22:17:10-07:00",
          "tree_id": "5311bc073f75f978a27e3a7a14899d2de1acedaa",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/17886f3636635f4e272f5847afa164438b28bd18"
        },
        "date": 1710566942681,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 753690,
            "range": "± 4102",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 844147,
            "range": "± 2396",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 795454,
            "range": "± 13612",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 837464,
            "range": "± 6981",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 803174,
            "range": "± 24688",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 837571,
            "range": "± 4577",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 807597,
            "range": "± 15131",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 847482,
            "range": "± 2965",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 796797,
            "range": "± 2873",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 839367,
            "range": "± 1810",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 938359,
            "range": "± 3028",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1189135,
            "range": "± 18550",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1740268,
            "range": "± 11739",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1225613,
            "range": "± 20330",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1484296,
            "range": "± 71180",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2063772,
            "range": "± 49932",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4208,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 38499,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 75779,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4322,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30864,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63361,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 68883,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1067893,
            "range": "± 50804",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2153620,
            "range": "± 7001",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12510282,
            "range": "± 51621",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85224979,
            "range": "± 936441",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 163846964,
            "range": "± 403213",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9807,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2517,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 476,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 553,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1754,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5894,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14875,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21550,
            "range": "± 210",
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
          "id": "00c21e2759cbf2fdf266ec9ff58d014bdcf4abaf",
          "message": "Use a BTreeSet for struct fields to assure stable hashing (#458)",
          "timestamp": "2024-04-04T11:19:58-07:00",
          "tree_id": "06ec6245550be3dc590b0dd9dfe8fdfad961c605",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/00c21e2759cbf2fdf266ec9ff58d014bdcf4abaf"
        },
        "date": 1712255533176,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 747530,
            "range": "± 7627",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 826076,
            "range": "± 3422",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 802113,
            "range": "± 15848",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 822988,
            "range": "± 34472",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 814104,
            "range": "± 15520",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 826390,
            "range": "± 3202",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 814851,
            "range": "± 3125",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 837009,
            "range": "± 6319",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 798883,
            "range": "± 3890",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 826630,
            "range": "± 5629",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 953800,
            "range": "± 4348",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1186162,
            "range": "± 40553",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1750674,
            "range": "± 38977",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1177287,
            "range": "± 8132",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1437098,
            "range": "± 43249",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 1991738,
            "range": "± 10800",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4168,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39006,
            "range": "± 604",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 79630,
            "range": "± 805",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4455,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32603,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 66751,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 66767,
            "range": "± 546",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1043840,
            "range": "± 9008",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2095665,
            "range": "± 11711",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12852768,
            "range": "± 192002",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 86187523,
            "range": "± 765829",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 165518632,
            "range": "± 988872",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9710,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2420,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 430,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 566,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1731,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5627,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14913,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21258,
            "range": "± 168",
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
          "id": "5dc7c6e33666978654d3e4ed519209c2f8fc3038",
          "message": "chore: Release (#460)",
          "timestamp": "2024-04-12T11:52:08-07:00",
          "tree_id": "fd3e3c53e1718808f781d8a3ef04170893284d91",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/5dc7c6e33666978654d3e4ed519209c2f8fc3038"
        },
        "date": 1712948649159,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 753638,
            "range": "± 15469",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 835831,
            "range": "± 2318",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 803420,
            "range": "± 22198",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 832739,
            "range": "± 5956",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 809653,
            "range": "± 16906",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 837758,
            "range": "± 1947",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 818457,
            "range": "± 5953",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 845091,
            "range": "± 2568",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 811960,
            "range": "± 2638",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 837219,
            "range": "± 3773",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 960050,
            "range": "± 7594",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1217763,
            "range": "± 10595",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1771238,
            "range": "± 22203",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1180768,
            "range": "± 7377",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1487101,
            "range": "± 5926",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2044281,
            "range": "± 6449",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4155,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39774,
            "range": "± 779",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 76761,
            "range": "± 617",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4306,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32514,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 67427,
            "range": "± 1612",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 68652,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1077672,
            "range": "± 15834",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2154052,
            "range": "± 4649",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12510063,
            "range": "± 58387",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85840785,
            "range": "± 759790",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 164790916,
            "range": "± 434765",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9750,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2449,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 439,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 592,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1772,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5691,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14582,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21160,
            "range": "± 41",
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
          "id": "2e9e4cd85a7c614f90b01ed0cfb359f74f6ff0bb",
          "message": "Remove `NULL` and `MISSING` `partiql-types` (#463)\n\nAs we are making progress towards fleshing out partiql type semantics with more details, the current informal consensus is that NULL and MISSING can only have meaning as values or constraints (E.g., NOT NULL); considering this, this PR removes PartiQLType::NULL and PartiQLType::MISSING.\r\n\r\nImportant to note is, this PR, types literals NULL and MISSING values as Undefined (or Unknown). The rationale for making this experimental decision is that both NULL and MISSING absent values represent no types; NULL is a property of present types rather than a type itself. E.g., in SQL NULL can be assigned to any type of column. In addition, its presence in operations can lead to NULL. This follows SQL's Three-Valued Logic i.e., \"If a null value affects the result of a logical expression, the result is neither true nor false but unknown.\".\r\n\r\nOne could argue that the Unknown type for PartiQL absent values (NULL and MISSING) is an inhabited Bottom Types.\r\n\r\nThis PR also includes the following:\r\n\r\n- removes unused partiql_ast_passes::partiql_typer\r\n- fixes Clippy multiple_bound_locations errors partiql-eval",
          "timestamp": "2024-06-14T10:29:20-07:00",
          "tree_id": "876f20d12df3b0485feba59fe64e4a66b6982f68",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/2e9e4cd85a7c614f90b01ed0cfb359f74f6ff0bb"
        },
        "date": 1718386884001,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 774533,
            "range": "± 16563",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 861350,
            "range": "± 3171",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 820254,
            "range": "± 13998",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 853809,
            "range": "± 3987",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 825050,
            "range": "± 4414",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 860652,
            "range": "± 2366",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 831514,
            "range": "± 3169",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 867419,
            "range": "± 3058",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 825232,
            "range": "± 5604",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 858983,
            "range": "± 2008",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 970126,
            "range": "± 3363",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1220538,
            "range": "± 11818",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1813863,
            "range": "± 8587",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1252766,
            "range": "± 7657",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1536893,
            "range": "± 33050",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2129085,
            "range": "± 4630",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4294,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39405,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 77075,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4320,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33899,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 68765,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 69407,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1086664,
            "range": "± 89435",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2167995,
            "range": "± 15109",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13411922,
            "range": "± 115353",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85882750,
            "range": "± 298670",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 166363476,
            "range": "± 587071",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9746,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2457,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 426,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 565,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1777,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5797,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14900,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21947,
            "range": "± 95",
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
          "id": "ee13dee0747efeb383db6b8caab223bc6e14e959",
          "message": "Add PartiQL Shape (#464)\n\nRefactors `partiql-types` by adding `PartiqlShape`; with this model, `PartiqlShape`\r\nis one of `Dynamic` (ex. `Any`), `AnyOf`, `Static`, or `Undefined`. `nullability` is\r\n defined as part of `StaticType`.\r\n\r\nThe large diff is as a result of the `PartiqlType` refactoring.",
          "timestamp": "2024-06-19T10:20:53-07:00",
          "tree_id": "d1ccef30805c37edf3f3d7e59b40d5b552108b15",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/ee13dee0747efeb383db6b8caab223bc6e14e959"
        },
        "date": 1718818377870,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 767717,
            "range": "± 11268",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 851108,
            "range": "± 1835",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 813150,
            "range": "± 19464",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 843925,
            "range": "± 32715",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 817304,
            "range": "± 7430",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 846594,
            "range": "± 2351",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 826997,
            "range": "± 4265",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 858037,
            "range": "± 4935",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 814499,
            "range": "± 2444",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 848180,
            "range": "± 2724",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 956398,
            "range": "± 2582",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1235065,
            "range": "± 18090",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1869138,
            "range": "± 19298",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1181105,
            "range": "± 9480",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1527971,
            "range": "± 8624",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2155595,
            "range": "± 9589",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4146,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 37762,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 74801,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4347,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33151,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 67895,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67680,
            "range": "± 766",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1047469,
            "range": "± 8010",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2100142,
            "range": "± 9448",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12798017,
            "range": "± 187554",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 88444199,
            "range": "± 1061763",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 168749431,
            "range": "± 536572",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10111,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2522,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 437,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 571,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1764,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5764,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14759,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21149,
            "range": "± 92",
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
          "id": "d491cb03a98d54d69555d4977a7e21050ff63ca8",
          "message": "Feat struct field optionality (#465)\n\nAdd Struct Field Optionality",
          "timestamp": "2024-06-19T13:34:37-07:00",
          "tree_id": "c0404794c7d8b52a40a919e164a7dfa972c1ef7f",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/d491cb03a98d54d69555d4977a7e21050ff63ca8"
        },
        "date": 1718830032834,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 771884,
            "range": "± 7796",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 860865,
            "range": "± 3796",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 821507,
            "range": "± 17852",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 852909,
            "range": "± 2499",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 823532,
            "range": "± 18618",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 855922,
            "range": "± 2930",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 830631,
            "range": "± 2718",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 864029,
            "range": "± 8166",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 825295,
            "range": "± 4211",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 859280,
            "range": "± 2580",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 969580,
            "range": "± 5511",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1270155,
            "range": "± 34240",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1851931,
            "range": "± 9176",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1236749,
            "range": "± 18604",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1558383,
            "range": "± 13185",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2164143,
            "range": "± 13939",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4257,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 38864,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 77167,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4301,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32700,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 66472,
            "range": "± 846",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 68040,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1063654,
            "range": "± 23963",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2126582,
            "range": "± 5275",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13948729,
            "range": "± 104574",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 89737833,
            "range": "± 1061584",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 172140198,
            "range": "± 542000",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9959,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2519,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 425,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 545,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1707,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5768,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14629,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21622,
            "range": "± 133",
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
          "id": "8a12ed636f86ad2b0029d9f4e58831a520c06ea3",
          "message": "Add codecove token to CI (#466)",
          "timestamp": "2024-06-19T14:21:35-07:00",
          "tree_id": "f113b630d1824d776d5c8061789c2627aacdd2bf",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/8a12ed636f86ad2b0029d9f4e58831a520c06ea3"
        },
        "date": 1718832822069,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 764897,
            "range": "± 11708",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 852499,
            "range": "± 2696",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 811565,
            "range": "± 13862",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 843777,
            "range": "± 8580",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 817531,
            "range": "± 5082",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 849593,
            "range": "± 4399",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 823164,
            "range": "± 3323",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 860020,
            "range": "± 4826",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 819087,
            "range": "± 2677",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 850804,
            "range": "± 1871",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 960812,
            "range": "± 3411",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1226382,
            "range": "± 20768",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1812248,
            "range": "± 42641",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1221422,
            "range": "± 17797",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1555506,
            "range": "± 10833",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2148403,
            "range": "± 8258",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4381,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 40072,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 79358,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4218,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32362,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 67065,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67514,
            "range": "± 1186",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1045666,
            "range": "± 24412",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2100608,
            "range": "± 29789",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13460455,
            "range": "± 157257",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 88050839,
            "range": "± 754363",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 168973117,
            "range": "± 2646837",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9728,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2488,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 434,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 544,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1754,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5789,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14885,
            "range": "± 661",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 22649,
            "range": "± 120",
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
          "id": "3165ebd691796481788b5d4fa827d8edc9f0f0f0",
          "message": "Minor additions to PartiQL types API (#467)\n\nAdd additional APIs to shape",
          "timestamp": "2024-06-20T17:20:08-07:00",
          "tree_id": "7f354f7362f8705ff85bccf42a6ecb7ea575b538",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/3165ebd691796481788b5d4fa827d8edc9f0f0f0"
        },
        "date": 1718929933977,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 765080,
            "range": "± 20313",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 869072,
            "range": "± 19926",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 812004,
            "range": "± 34468",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 848851,
            "range": "± 1785",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 821924,
            "range": "± 1881",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 853578,
            "range": "± 3633",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 826518,
            "range": "± 3755",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 858887,
            "range": "± 2981",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 819889,
            "range": "± 2350",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 852680,
            "range": "± 3038",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 966013,
            "range": "± 2511",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1263789,
            "range": "± 74251",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1877974,
            "range": "± 7831",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1251345,
            "range": "± 29858",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1603792,
            "range": "± 21302",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2211008,
            "range": "± 14009",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4242,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 38562,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 75836,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4307,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33187,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 68944,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67662,
            "range": "± 944",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1054792,
            "range": "± 17808",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2109490,
            "range": "± 6932",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12835001,
            "range": "± 221934",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 88225079,
            "range": "± 866513",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 169301613,
            "range": "± 457580",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9923,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2461,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 441,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 552,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1779,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5754,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14571,
            "range": "± 406",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21477,
            "range": "± 73",
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
          "id": "0ff57fe4d162315456df3d1019713994f9327b7a",
          "message": "Add PartiQL DDL Extension (#468)\n\nAdds `partiql-extensions-ddl`.\r\n\r\n#### Example\r\n\r\n```SQL\r\n\"dependents\" ARRAY<VARCHAR>,\r\n\"details\" STRUCT<\"a\": UNION<TINYINT,DECIMAL(5, 4)>,\"b\": ARRAY<VARCHAR>,\"c\": DOUBLE>,\r\n\"employee_id\" TINYINT,\r\n\"full_name\" VARCHAR,\r\n\"salary\" DECIMAL(8, 2)\"\r\n```",
          "timestamp": "2024-06-24T12:13:44-07:00",
          "tree_id": "29641aed14918a284ac2cb954dccacc002a98508",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/0ff57fe4d162315456df3d1019713994f9327b7a"
        },
        "date": 1719257144141,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 773116,
            "range": "± 2623",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 865623,
            "range": "± 2549",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 823424,
            "range": "± 20776",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 858799,
            "range": "± 2523",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 833652,
            "range": "± 3931",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 860304,
            "range": "± 2109",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 833308,
            "range": "± 5201",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 870059,
            "range": "± 3095",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 829368,
            "range": "± 6942",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 863942,
            "range": "± 2698",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 966623,
            "range": "± 3124",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1236084,
            "range": "± 21254",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1810208,
            "range": "± 6165",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1209188,
            "range": "± 21886",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1547784,
            "range": "± 42181",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2155021,
            "range": "± 9462",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4355,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39951,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 78256,
            "range": "± 373",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4370,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33576,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 68755,
            "range": "± 1872",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67156,
            "range": "± 335",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1046604,
            "range": "± 11680",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2089167,
            "range": "± 8096",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13269424,
            "range": "± 124609",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 87780096,
            "range": "± 416747",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 168266854,
            "range": "± 12768738",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9838,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2477,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 425,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 580,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1776,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5920,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 15097,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 22184,
            "range": "± 274",
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
          "id": "6aacf268dd461ac8c163e6ce03a9ac5ce1e195d0",
          "message": "PartiQL types rename (#469)\n\nRenames StaticTypeVariant to Static to make it shorter and the usage easier—it is behavioral preserving.",
          "timestamp": "2024-06-24T13:12:30-07:00",
          "tree_id": "c56273b5229c5f77a0b73037fb67359b1cfa5ff1",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/6aacf268dd461ac8c163e6ce03a9ac5ce1e195d0"
        },
        "date": 1719260687658,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 763871,
            "range": "± 16021",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 852943,
            "range": "± 3449",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 814660,
            "range": "± 33937",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 849958,
            "range": "± 19612",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 824840,
            "range": "± 9846",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 850504,
            "range": "± 2210",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 823622,
            "range": "± 4384",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 858951,
            "range": "± 43476",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 819811,
            "range": "± 2761",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 851693,
            "range": "± 1550",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 957278,
            "range": "± 2662",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1248673,
            "range": "± 36672",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1806472,
            "range": "± 8763",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1185526,
            "range": "± 19073",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1550045,
            "range": "± 22776",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2135610,
            "range": "± 8933",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4291,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39114,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 76826,
            "range": "± 685",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4348,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32854,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 68294,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67609,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1052784,
            "range": "± 24201",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2112743,
            "range": "± 10868",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13547112,
            "range": "± 413645",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 88002637,
            "range": "± 810315",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 168873333,
            "range": "± 1558481",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9770,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2457,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 435,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 554,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1749,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5816,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 15316,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 22632,
            "range": "± 439",
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
          "id": "93622bd7a0a366f9283e0731b34412e2be590480",
          "message": "chore: Release v0.8.0 (#470)\n\n* chore: Release v0.8.0",
          "timestamp": "2024-06-24T15:04:07-07:00",
          "tree_id": "4b463fe011847a94d0dbef7ce0f9772593e0b199",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/93622bd7a0a366f9283e0731b34412e2be590480"
        },
        "date": 1719267367135,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 767550,
            "range": "± 25529",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 853211,
            "range": "± 3789",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 811172,
            "range": "± 12702",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 848275,
            "range": "± 8242",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 815662,
            "range": "± 4178",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 850462,
            "range": "± 18696",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 821044,
            "range": "± 2913",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 859104,
            "range": "± 3372",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 820101,
            "range": "± 15137",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 854531,
            "range": "± 3329",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 966254,
            "range": "± 5223",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1236532,
            "range": "± 9954",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1814702,
            "range": "± 85856",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1227968,
            "range": "± 4560",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1582041,
            "range": "± 4251",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2170637,
            "range": "± 14894",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4329,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 38661,
            "range": "± 519",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 76090,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4319,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32940,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 68083,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67448,
            "range": "± 2332",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1059015,
            "range": "± 22871",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2114208,
            "range": "± 9437",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13323166,
            "range": "± 626795",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 89188292,
            "range": "± 342895",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 169856980,
            "range": "± 1103692",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9820,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2542,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 427,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 578,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1811,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 6210,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 15803,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 23725,
            "range": "± 142",
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
          "id": "a54868edc47dff19664e5a35df85278107c81733",
          "message": "Initial partial AST pretty-printer (#471)\n\nInitial partial AST pretty-printer",
          "timestamp": "2024-07-09T12:26:39-07:00",
          "tree_id": "66f40f3420898cc920d7dd91bb4ce286b6ae249c",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/a54868edc47dff19664e5a35df85278107c81733"
        },
        "date": 1720553925537,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 768879,
            "range": "± 44474",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 852871,
            "range": "± 4563",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 818561,
            "range": "± 24924",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 846734,
            "range": "± 3407",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 832601,
            "range": "± 5272",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 859514,
            "range": "± 2662",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 827881,
            "range": "± 8412",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 860484,
            "range": "± 2589",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 823950,
            "range": "± 4769",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 850306,
            "range": "± 2494",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 960769,
            "range": "± 2950",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1270408,
            "range": "± 9721",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1880159,
            "range": "± 11463",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1240402,
            "range": "± 20711",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1581066,
            "range": "± 63203",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2187305,
            "range": "± 11171",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4244,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 40550,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 78545,
            "range": "± 2822",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4394,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33209,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 68529,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 68549,
            "range": "± 487",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1078118,
            "range": "± 10086",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2156664,
            "range": "± 8444",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13199986,
            "range": "± 178346",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 88062096,
            "range": "± 1058303",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 168580958,
            "range": "± 556192",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9884,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2488,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 427,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 565,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1750,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5900,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14823,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21985,
            "range": "± 877",
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
          "id": "684fff3092311722c15b5b86506c375a49f78d1a",
          "message": "Add snapshot testing for AST pretty printing (#474)",
          "timestamp": "2024-07-09T13:07:48-07:00",
          "tree_id": "995e7a6a6089a1dabcbf1c55db33e655afa71b05",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/684fff3092311722c15b5b86506c375a49f78d1a"
        },
        "date": 1720556394008,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 771998,
            "range": "± 3660",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 868193,
            "range": "± 32480",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 819127,
            "range": "± 13014",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 850385,
            "range": "± 12627",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 835124,
            "range": "± 7322",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 864818,
            "range": "± 5042",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 826288,
            "range": "± 9355",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 860400,
            "range": "± 3296",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 822396,
            "range": "± 21825",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 851443,
            "range": "± 4511",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 967909,
            "range": "± 18508",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1261195,
            "range": "± 11052",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1872671,
            "range": "± 20081",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1178677,
            "range": "± 10540",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1589490,
            "range": "± 19655",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2165888,
            "range": "± 8946",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4269,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39638,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 77020,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4263,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33192,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 67869,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 69565,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1082605,
            "range": "± 19293",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2170474,
            "range": "± 56024",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12909400,
            "range": "± 104669",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 87331642,
            "range": "± 228299",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 168169555,
            "range": "± 384409",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9942,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2482,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 428,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 556,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1736,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5734,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14833,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21781,
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
          "id": "1b22fb6cdf70b33062ca1f28d99464afc16af809",
          "message": "Add `NodeBuilder` for building ASTs (#476)",
          "timestamp": "2024-07-10T16:09:29-07:00",
          "tree_id": "86612cfa44ae18127eef6d392431d1c17e580c15",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/1b22fb6cdf70b33062ca1f28d99464afc16af809"
        },
        "date": 1720653697912,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 775078,
            "range": "± 8705",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 862652,
            "range": "± 6626",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 830846,
            "range": "± 7811",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 855765,
            "range": "± 2496",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 843309,
            "range": "± 3129",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 871031,
            "range": "± 11411",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 840150,
            "range": "± 3401",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 870457,
            "range": "± 1793",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 833829,
            "range": "± 2930",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 863680,
            "range": "± 2889",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 971146,
            "range": "± 7271",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1228144,
            "range": "± 5068",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1825750,
            "range": "± 17664",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1171155,
            "range": "± 7797",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1475823,
            "range": "± 17342",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2067865,
            "range": "± 12643",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4149,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39532,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 76008,
            "range": "± 562",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4432,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 35043,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 71338,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 69546,
            "range": "± 702",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1078664,
            "range": "± 86226",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2153387,
            "range": "± 13338",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12766894,
            "range": "± 60043",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 86920998,
            "range": "± 1270499",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 167856739,
            "range": "± 489213",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9891,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2474,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 427,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 586,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1762,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5772,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 15496,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 22579,
            "range": "± 111",
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
          "id": "c91c5a4727f65523dea55fbdd9b9530d1df19d63",
          "message": "Replace `BTreeSet` with `IndexSet` in type to order more intuitively (#477)",
          "timestamp": "2024-07-11T14:57:40-07:00",
          "tree_id": "b529e923711d0c625eca4406955d174135bbe4bf",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/c91c5a4727f65523dea55fbdd9b9530d1df19d63"
        },
        "date": 1720735816899,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 774408,
            "range": "± 10165",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 861458,
            "range": "± 2997",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 820770,
            "range": "± 11846",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 864445,
            "range": "± 2515",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 823916,
            "range": "± 39653",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 861591,
            "range": "± 12547",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 832754,
            "range": "± 8948",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 873721,
            "range": "± 13503",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 828017,
            "range": "± 3246",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 864254,
            "range": "± 3454",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 969776,
            "range": "± 3143",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1218335,
            "range": "± 82167",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1832486,
            "range": "± 13544",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1293061,
            "range": "± 12846",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1592716,
            "range": "± 16316",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2189565,
            "range": "± 8129",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4635,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 43481,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 85428,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4322,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32995,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 69154,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 66539,
            "range": "± 327",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1046584,
            "range": "± 28576",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2098596,
            "range": "± 23287",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13184607,
            "range": "± 397271",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 89455403,
            "range": "± 395638",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 171764736,
            "range": "± 1745204",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9850,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2462,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 425,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 573,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1731,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5897,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 15037,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 22022,
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
          "id": "1fc44de0c9577dea496db9538e3f21a04d3ecc93",
          "message": "Refactors to make working with types a bit more ergonomic (#478)",
          "timestamp": "2024-07-23T14:25:09-07:00",
          "tree_id": "3e42c9001eac1f6712f6e9e799fd889b906126b3",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/1fc44de0c9577dea496db9538e3f21a04d3ecc93"
        },
        "date": 1721770632527,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 762740,
            "range": "± 5063",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 864605,
            "range": "± 1933",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 814011,
            "range": "± 18439",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 851936,
            "range": "± 28629",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 817382,
            "range": "± 16574",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 867643,
            "range": "± 44075",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 826160,
            "range": "± 13760",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 878777,
            "range": "± 4361",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 821429,
            "range": "± 3778",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 862825,
            "range": "± 1761",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 972378,
            "range": "± 37693",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1201379,
            "range": "± 10628",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1801572,
            "range": "± 15198",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1289977,
            "range": "± 32602",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1551901,
            "range": "± 17671",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2134103,
            "range": "± 6462",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4280,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 39341,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 77553,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4242,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32969,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 69354,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67342,
            "range": "± 2862",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1049343,
            "range": "± 15624",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2112313,
            "range": "± 5548",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12624353,
            "range": "± 153243",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85081863,
            "range": "± 827855",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 163570222,
            "range": "± 968951",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9919,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2474,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 428,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 586,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1781,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5724,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14771,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 22309,
            "range": "± 139",
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
          "id": "3ca67b770c626c6a4d5493033667661b0792b4ce",
          "message": "chore: Release v0.9.0 (#479)",
          "timestamp": "2024-07-24T06:26:16-07:00",
          "tree_id": "2b0b82d85a97b9ab169b730290f626c30dfbaf20",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/3ca67b770c626c6a4d5493033667661b0792b4ce"
        },
        "date": 1721828303346,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 775927,
            "range": "± 3468",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 856683,
            "range": "± 2372",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 818625,
            "range": "± 15375",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 851347,
            "range": "± 4987",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 825917,
            "range": "± 2576",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 850563,
            "range": "± 7003",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 827990,
            "range": "± 34600",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 864413,
            "range": "± 11462",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 829453,
            "range": "± 16364",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 857194,
            "range": "± 2443",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 968022,
            "range": "± 3875",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1207285,
            "range": "± 18484",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1825661,
            "range": "± 17209",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1240635,
            "range": "± 74432",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1520707,
            "range": "± 16588",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2139065,
            "range": "± 10755",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 4221,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 40063,
            "range": "± 732",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 81388,
            "range": "± 712",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4344,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33761,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 70343,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67749,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1050246,
            "range": "± 23555",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2103307,
            "range": "± 10523",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12792235,
            "range": "± 113115",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 86096723,
            "range": "± 982386",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 165076640,
            "range": "± 2722727",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9756,
            "range": "± 531",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2477,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 425,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 566,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 1729,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 5812,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 14670,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 21437,
            "range": "± 119",
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
          "id": "979c0cb1ce8f7762fca3c7b68c7055e9c6b63f32",
          "message": "Add Parsing of `EXCLUDE` (#480)",
          "timestamp": "2024-07-25T13:11:50-07:00",
          "tree_id": "de17733b4f79459ae7c6b8af80f64df3998d71e6",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/979c0cb1ce8f7762fca3c7b68c7055e9c6b63f32"
        },
        "date": 1721939044313,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 770772,
            "range": "± 2036",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 855654,
            "range": "± 1802",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 821448,
            "range": "± 14841",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 852586,
            "range": "± 17919",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 825473,
            "range": "± 1830",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 854880,
            "range": "± 2705",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 832107,
            "range": "± 3311",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 864136,
            "range": "± 3303",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 824732,
            "range": "± 17967",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 859648,
            "range": "± 2287",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 965929,
            "range": "± 3494",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1234556,
            "range": "± 17076",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1825909,
            "range": "± 15262",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1226847,
            "range": "± 9659",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1517280,
            "range": "± 7336",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2111135,
            "range": "± 10607",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5471,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 48888,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 93314,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4337,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33665,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 70423,
            "range": "± 2461",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67596,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1057775,
            "range": "± 21462",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2119337,
            "range": "± 13062",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13007144,
            "range": "± 118130",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 87319711,
            "range": "± 1064507",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 167286656,
            "range": "± 695881",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9810,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2476,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 439,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 654,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2259,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 6975,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 18265,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 25502,
            "range": "± 250",
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
          "id": "2da9cdba3bbe35219f7ad7015bac053cf12255ad",
          "message": "chore: Release (#481)",
          "timestamp": "2024-07-26T12:25:32-07:00",
          "tree_id": "7c809c8cade15669db4bc54e5401c9fbc87a78a1",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/2da9cdba3bbe35219f7ad7015bac053cf12255ad"
        },
        "date": 1722022665539,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 761913,
            "range": "± 14385",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 853598,
            "range": "± 26650",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 809269,
            "range": "± 13784",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 849768,
            "range": "± 6574",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 820144,
            "range": "± 9836",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 850831,
            "range": "± 2157",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 825705,
            "range": "± 3804",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 863395,
            "range": "± 2866",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 816547,
            "range": "± 1833",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 856852,
            "range": "± 3790",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 959703,
            "range": "± 4902",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1200958,
            "range": "± 8200",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1799692,
            "range": "± 10981",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1267278,
            "range": "± 25309",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1509818,
            "range": "± 14136",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2112653,
            "range": "± 9660",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5677,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 48290,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 93389,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4414,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33339,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 70444,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 66887,
            "range": "± 373",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1042690,
            "range": "± 9301",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2093184,
            "range": "± 7870",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12671873,
            "range": "± 212003",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85566685,
            "range": "± 739891",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 164700563,
            "range": "± 807391",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9804,
            "range": "± 473",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2537,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 434,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 728,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2281,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7258,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 18954,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 27188,
            "range": "± 134",
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
          "id": "6dc088c04587d511ba023065e2b064dcfce0e972",
          "message": "Improvements to pretty-printing for AND/OR/CASE/PIVOT (#482)",
          "timestamp": "2024-07-31T15:06:10-07:00",
          "tree_id": "66f985d760877adac0ad53d0cd4e7a5f8049bbaa",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/6dc088c04587d511ba023065e2b064dcfce0e972"
        },
        "date": 1722464303438,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 769347,
            "range": "± 8086",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 864631,
            "range": "± 3111",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 817735,
            "range": "± 44170",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 859871,
            "range": "± 16448",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 823541,
            "range": "± 16473",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 860789,
            "range": "± 3429",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 828686,
            "range": "± 1915",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 867876,
            "range": "± 32406",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 824516,
            "range": "± 3833",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 868788,
            "range": "± 2851",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 965437,
            "range": "± 13827",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1203160,
            "range": "± 24930",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1805354,
            "range": "± 41273",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1264656,
            "range": "± 12802",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1550278,
            "range": "± 24933",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2145855,
            "range": "± 41224",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5378,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 46773,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 91519,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4321,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33227,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 69079,
            "range": "± 1950",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67288,
            "range": "± 1696",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1050445,
            "range": "± 41700",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2102844,
            "range": "± 19876",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13304642,
            "range": "± 223004",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 86429143,
            "range": "± 2425132",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 166575250,
            "range": "± 2418779",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9850,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2502,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 423,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 775,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2398,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 6958,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 18549,
            "range": "± 418",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 25695,
            "range": "± 131",
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
          "id": "e5b8ce7497f515bc998a31cd8b45fb2f3e9287a2",
          "message": "Move NodeId Generation to a separate crate (#484)\n\nMoves the `NodeId` and its Id generation to a new crate, `partiql-common` so that it can get consumed by multiple crates.\n\n---------\n\nCo-authored-by: Josh Pschorr <joshps@amazon.com>",
          "timestamp": "2024-08-08T18:39:40-07:00",
          "tree_id": "b45d14cd5c5a2e9b9a69f32c1f7b42db2e7b666b",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/e5b8ce7497f515bc998a31cd8b45fb2f3e9287a2"
        },
        "date": 1723168318807,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 775198,
            "range": "± 34461",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 856544,
            "range": "± 5154",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 820861,
            "range": "± 18988",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 852867,
            "range": "± 4970",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 827041,
            "range": "± 2639",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 860210,
            "range": "± 10560",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 835593,
            "range": "± 3961",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 864949,
            "range": "± 4270",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 824926,
            "range": "± 1621",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 857067,
            "range": "± 1983",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 968375,
            "range": "± 8380",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1204293,
            "range": "± 13453",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1801364,
            "range": "± 31255",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1229000,
            "range": "± 13263",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1508448,
            "range": "± 9427",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2098733,
            "range": "± 10245",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5619,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 48240,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 93707,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4326,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33011,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 68472,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 66983,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1047620,
            "range": "± 5242",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2101187,
            "range": "± 26131",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13463456,
            "range": "± 113930",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 91368497,
            "range": "± 276047",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 174268173,
            "range": "± 529438",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9998,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2479,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 428,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 717,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2263,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7145,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 18704,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 25839,
            "range": "± 81",
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
          "id": "7203852bec2bc30b145f6af775076a2b7a2f6efd",
          "message": "Adds `PartiqlShapeBuilder` with `NodeId` generation for the `StaticType` (#485)\n\n* Adds `NodeId` to the `StaticType`\r\n\r\nThis PR\r\n- adds `NodeId` to `StaticType`; this is to be able to use the `id` as a reference to add additional data to the types out of band.\r\n- makes `AutoNodeIdGenerator` thread-safe\r\n- adds `PartiqlShapeBuilder` and moves some `PartiqlShape` APIs to it; this is to be able to generate unique `NodeId`s for a `PartiqlShape` that includes static types that themselves can include other static types.\r\n- adds a static thread safe `shape_builder` function that provides a convenient way for using `PartiqlShapeBuilder` for creating new shapes.\r\n- prepends existing type macros with `type` such as `type_int!` to make macro names more friendly.\r\n- removes `const` PartiQL types under `partiql-types` in favor of `PartiqlShapeBuilder`.\r\n\r\n**_The majority of the diffs are related to the macro renames or replacement with the previous `const` types. The main change is in `partiql-types/src/lib.rs` file._**",
          "timestamp": "2024-08-14T15:17:22-07:00",
          "tree_id": "2403551931147ad0c8020ca89bbad8fcbbe42b51",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/7203852bec2bc30b145f6af775076a2b7a2f6efd"
        },
        "date": 1723674562040,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 761989,
            "range": "± 2584",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 856629,
            "range": "± 2352",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 813071,
            "range": "± 16235",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 849796,
            "range": "± 3499",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 817287,
            "range": "± 12938",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 854925,
            "range": "± 1637",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 828374,
            "range": "± 15165",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 863610,
            "range": "± 3061",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 817340,
            "range": "± 4491",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 857382,
            "range": "± 5483",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 962352,
            "range": "± 2579",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1184994,
            "range": "± 37004",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1802258,
            "range": "± 23102",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1258025,
            "range": "± 50557",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1517674,
            "range": "± 8829",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2111719,
            "range": "± 14340",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5949,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 49885,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 100431,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4345,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33556,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 69779,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67795,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1047210,
            "range": "± 9506",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2098549,
            "range": "± 5644",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12755601,
            "range": "± 45848",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 86481769,
            "range": "± 851403",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 166749677,
            "range": "± 551437",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9733,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2478,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 435,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 901,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2510,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7341,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19828,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 27149,
            "range": "± 181",
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
          "id": "843d3eedc5f25f7743c6e182769d6669d16a7dff",
          "message": "Add metas to PartiQL Common (#488)\n\nAdds an implementation for storing metadata for PartiQL objects. Currently, the implementation does not include any traits. It introduces `PartiqlMetadata` and `PartiqlMetaValue` structures:\r\n\r\n```rust\r\n        let foo_val = PartiqlMetaValue::String(\"foo\".to_string());\r\n        let i64_val = PartiqlMetaValue::Int64(2);\r\n\r\n        let expected_vec_val = vec![foo_val, i64_val];\r\n        let expected_bool_val = true;\r\n        let expected_int_val = 2;\r\n        let expected_float_val = 2.5;\r\n        let expected_str_val = \"foo\";\r\n\r\n        let mut expected_map = PartiqlMetadata::new();\r\n        expected_map.insert(\"bool value\", expected_bool_val.into());\r\n        expected_map.insert(\"integer value\", expected_int_val.into());\r\n\r\n        let mut metas = PartiqlMetadata::new();\r\n        metas.insert(\"vec value\", expected_vec_val.clone().into());\r\n        metas.insert(\"bool value\", expected_bool_val.into());\r\n        metas.insert(\"integer value\", expected_int_val.into());\r\n        metas.insert(\"float value\", expected_float_val.into());\r\n        metas.insert(\"string value\", expected_str_val.into());\r\n        metas.insert(\"map value\", expected_map.clone().into());\r\n\r\n        let vec_val = metas.vec_value(\"vec value\").expect(\"vec meta value\");\r\n        let bool_val = metas.bool_value(\"bool value\").expect(\"bool meta value\");\r\n        let int_val = metas.i32_value(\"integer value\").expect(\"i32 meta value\");\r\n        let float_val = metas.f64_value(\"float value\").expect(\"f64 meta value\");\r\n        let string_val = metas.string_value(\"string value\").expect(\"string meta value\");\r\n        let map_val = metas.map_value(\"map value\").expect(\"map meta value\");\r\n\r\n        assert_eq!(vec_val, expected_vec_val.clone());\r\n        assert_eq!(bool_val, expected_bool_val.clone());\r\n        assert_eq!(int_val, expected_int_val.clone());\r\n        assert_eq!(float_val, expected_float_val.clone());\r\n        assert_eq!(string_val, expected_str_val);\r\n        assert_eq!(map_val, expected_map.clone());\r\n```",
          "timestamp": "2024-08-16T18:19:33-07:00",
          "tree_id": "d0af1aa6d5ffb8e18199902ab098b8c2e7d5b95b",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/843d3eedc5f25f7743c6e182769d6669d16a7dff"
        },
        "date": 1723858297577,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 759774,
            "range": "± 16111",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 851602,
            "range": "± 1675",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 809485,
            "range": "± 31348",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 850096,
            "range": "± 2714",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 813205,
            "range": "± 2203",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 851965,
            "range": "± 18121",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 832033,
            "range": "± 4653",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 861344,
            "range": "± 4073",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 815417,
            "range": "± 2226",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 855702,
            "range": "± 2666",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 957159,
            "range": "± 4904",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1213008,
            "range": "± 15351",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1814083,
            "range": "± 5194",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1263534,
            "range": "± 15338",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1528754,
            "range": "± 12065",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2132246,
            "range": "± 21947",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5994,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 52257,
            "range": "± 560",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 101862,
            "range": "± 335",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4264,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32659,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 69444,
            "range": "± 1090",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67184,
            "range": "± 484",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1052870,
            "range": "± 8363",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2111762,
            "range": "± 8350",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12633251,
            "range": "± 38348",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 85671873,
            "range": "± 809621",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 164703537,
            "range": "± 499568",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9731,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2489,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 432,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 847,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2525,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7762,
            "range": "± 504",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19915,
            "range": "± 679",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 27020,
            "range": "± 208",
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
          "id": "298a7ac5b46763e7d22b4b3510755375b820c565",
          "message": "Add more informational `Display` for `Static` types (#489)",
          "timestamp": "2024-08-20T15:03:05-07:00",
          "tree_id": "dd1e10a738335b957d21bcab5a09ef3ff27aa583",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/298a7ac5b46763e7d22b4b3510755375b820c565"
        },
        "date": 1724192109293,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 765347,
            "range": "± 8513",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 847222,
            "range": "± 7986",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 812823,
            "range": "± 30152",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 842512,
            "range": "± 7959",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 817310,
            "range": "± 5161",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 847133,
            "range": "± 3064",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 819427,
            "range": "± 31464",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 855923,
            "range": "± 5014",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 817783,
            "range": "± 5423",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 848903,
            "range": "± 1816",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 960199,
            "range": "± 3322",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1201753,
            "range": "± 77040",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1810218,
            "range": "± 23349",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1230609,
            "range": "± 44102",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1506612,
            "range": "± 11262",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2107345,
            "range": "± 18120",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5893,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 49753,
            "range": "± 1552",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 97644,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4357,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33633,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 69764,
            "range": "± 674",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 69164,
            "range": "± 565",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1086857,
            "range": "± 29617",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2177362,
            "range": "± 5766",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12823198,
            "range": "± 139262",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 86529718,
            "range": "± 867525",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 165759829,
            "range": "± 439741",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9835,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2518,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 416,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 940,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2513,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7478,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19843,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 27436,
            "range": "± 176",
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
          "id": "deff7a8c6e9be48139fa2bac09f9712309c2f933",
          "message": "Make it more ergonmic to use `PartiqlShapeBuilder` and type macros (#490)",
          "timestamp": "2024-08-22T14:28:16-07:00",
          "tree_id": "75a8c19c09e6431880ce3520b2362a3cbf3bcd8d",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/deff7a8c6e9be48139fa2bac09f9712309c2f933"
        },
        "date": 1724362820420,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 767418,
            "range": "± 16285",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 903823,
            "range": "± 3287",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 819135,
            "range": "± 12531",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 854689,
            "range": "± 1651",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 823294,
            "range": "± 3249",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 857361,
            "range": "± 2573",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 827553,
            "range": "± 2598",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 867044,
            "range": "± 28154",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 822303,
            "range": "± 5920",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 855984,
            "range": "± 2631",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 971203,
            "range": "± 6498",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1196258,
            "range": "± 5932",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1797637,
            "range": "± 8280",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1246192,
            "range": "± 18158",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1525246,
            "range": "± 14601",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2113403,
            "range": "± 18021",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6086,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 52508,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 99616,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4306,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33549,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 69303,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67083,
            "range": "± 2328",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1055829,
            "range": "± 17751",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2106298,
            "range": "± 4583",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12855153,
            "range": "± 122205",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 86807072,
            "range": "± 282541",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 167598895,
            "range": "± 2277227",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10468,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2481,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 418,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 838,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2827,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7932,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20368,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 27608,
            "range": "± 138",
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
          "id": "70b5ea252770dfeecfd80617e7e1880d0dc99ac0",
          "message": "Merge fixes from `dev-ion-doc` feature branch back to `main` (#497)\n\n* Rename `ion-rs` to `ion-rs_old` in preparation of upgrade (#492)\r\n\r\n* Refactor lexer into module & upgrade lexer & parser dependencies (#493)\r\n\r\n* Upgrade project deps to latest; use semver, not wildcard nor tilde (#494)",
          "timestamp": "2024-09-17T15:38:26-07:00",
          "tree_id": "d679b8adc39580a39346765ceeaa0a17e45a1336",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/70b5ea252770dfeecfd80617e7e1880d0dc99ac0"
        },
        "date": 1726613443948,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 759500,
            "range": "± 1408",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 849462,
            "range": "± 4155",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 806498,
            "range": "± 14064",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 843196,
            "range": "± 4176",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 813808,
            "range": "± 2458",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 848859,
            "range": "± 16282",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 814628,
            "range": "± 2353",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 862783,
            "range": "± 3478",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 813703,
            "range": "± 2335",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 846973,
            "range": "± 3297",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 943604,
            "range": "± 2933",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1165672,
            "range": "± 16453",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1762193,
            "range": "± 14659",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1277611,
            "range": "± 15074",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1492997,
            "range": "± 19875",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2123779,
            "range": "± 12249",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6113,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 49828,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 97703,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4307,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32139,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 67087,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67118,
            "range": "± 1029",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1051816,
            "range": "± 9727",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2104501,
            "range": "± 3002",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12482615,
            "range": "± 60899",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 84866990,
            "range": "± 802755",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 163210743,
            "range": "± 892385",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10122,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2456,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 429,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 866,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2616,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7933,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20088,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 27415,
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
          "id": "104a7d10a8c4f8a3a16d7899b2c411a7bfff259f",
          "message": "Add UDFs to catalog; Add extension UDFs for TUPLEUNION/TUPLECONCAT (#496)\n\nExtensions and scalar UDFs\r\n---\r\nAdds ability to create and register scalar User Defined Functions (UDFs)\r\n\r\nTUPLEUNION & TUPLECONCAT\r\n---\r\n \r\nAdds `TUPLEUNION` and `TUPLECONCAT` functions via an extension crate under the `extension` crate namespace as scalar UDFs.\r\n\r\n`TUPLEUNION` is as per 6.3.2 of the spec (https://partiql.org/assets/PartiQL-Specification.pdf#subsubsection.6.3.2)\r\n\r\n`TUPLECONCAT` is inspired by description of concatenating binding environments as per section 3.3 of the spec (https://partiql.org/assets/PartiQL-Specification.pdf#subsection.3.3) and as requested in #495.\r\n\r\n---\r\n\r\n`tupleunion({ 'bob': 1, 'sally': 'error' }, { 'sally': 1 }, { 'sally': 2 }, { 'sally': 3 }, { 'sally': 4 })`\r\n->\r\n`{ 'bob': 1, 'sally': 'error', 'sally': 1, 'sally': 2, 'sally': 3, 'sally': 4 }`\r\n\r\n---\r\n\r\n`tupleconcat({ 'bob': 1, 'sally': 'error' }, { 'sally': 1 }, { 'sally': 2 }, { 'sally': 3 }, { 'sally': 4 })`\r\n->\r\n`{ 'sally': 4, 'bob': 1 }`",
          "timestamp": "2024-10-02T14:44:08-07:00",
          "tree_id": "ea194db6aa8f6e6adc99772924fd468ef8e4cefd",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/104a7d10a8c4f8a3a16d7899b2c411a7bfff259f"
        },
        "date": 1727906199315,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 763771,
            "range": "± 16741",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 851786,
            "range": "± 1842",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 809835,
            "range": "± 14050",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 848062,
            "range": "± 13483",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 812714,
            "range": "± 1506",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 851673,
            "range": "± 7418",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 818977,
            "range": "± 3083",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 860550,
            "range": "± 4522",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 814833,
            "range": "± 2186",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 851388,
            "range": "± 3259",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 967156,
            "range": "± 3874",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1234665,
            "range": "± 29328",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1836036,
            "range": "± 8969",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1291597,
            "range": "± 20131",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1572591,
            "range": "± 7342",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2156366,
            "range": "± 12394",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6093,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50048,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 98554,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4320,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32611,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 68061,
            "range": "± 714",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67828,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1064049,
            "range": "± 14178",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2124398,
            "range": "± 5112",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13498152,
            "range": "± 233756",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 91551008,
            "range": "± 350907",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 176077656,
            "range": "± 654476",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10072,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2635,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 476,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 882,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2576,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7564,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20026,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 27626,
            "range": "± 111",
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
          "id": "9298a157e34af603d0b805fa78cc42a064d05ab3",
          "message": "Update some dependencies and deny checks (#500)",
          "timestamp": "2024-10-03T10:02:38-07:00",
          "tree_id": "eaa12d6b458228e8bd86583a0d8bb617fffbdfc7",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/9298a157e34af603d0b805fa78cc42a064d05ab3"
        },
        "date": 1727975713849,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 767860,
            "range": "± 5048",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 851808,
            "range": "± 11370",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 818214,
            "range": "± 18997",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 842799,
            "range": "± 4117",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 821240,
            "range": "± 5643",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 853724,
            "range": "± 3871",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 827470,
            "range": "± 3550",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 861879,
            "range": "± 2520",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 823023,
            "range": "± 2467",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 853254,
            "range": "± 2893",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 977420,
            "range": "± 9951",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1242492,
            "range": "± 16902",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1848263,
            "range": "± 17574",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1285995,
            "range": "± 11375",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1559719,
            "range": "± 11495",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2169716,
            "range": "± 9905",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6359,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51722,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 98688,
            "range": "± 442",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4400,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32652,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 67029,
            "range": "± 542",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67723,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1054214,
            "range": "± 9079",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2113592,
            "range": "± 5842",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13174407,
            "range": "± 231240",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 90338340,
            "range": "± 875171",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 172849363,
            "range": "± 370968",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9925,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2518,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 447,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 964,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2681,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7864,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20868,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28705,
            "range": "± 322",
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
          "id": "010602f9ef904a87010e4d3a2e74fc86778a302f",
          "message": "chore: Release v0.11.0 (#501)",
          "timestamp": "2024-10-03T11:04:42-07:00",
          "tree_id": "7193bbdf7013460f40a87b8579c42b7df4cc05e5",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/010602f9ef904a87010e4d3a2e74fc86778a302f"
        },
        "date": 1727979452666,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 772643,
            "range": "± 10671",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 862425,
            "range": "± 11947",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 825442,
            "range": "± 17609",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 859663,
            "range": "± 4161",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 830355,
            "range": "± 3355",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 864502,
            "range": "± 4879",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 833467,
            "range": "± 5425",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 869713,
            "range": "± 4344",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 827695,
            "range": "± 3084",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 864752,
            "range": "± 9317",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 976093,
            "range": "± 3970",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1246191,
            "range": "± 16318",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1861769,
            "range": "± 23192",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1236558,
            "range": "± 12999",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1548479,
            "range": "± 12188",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2168209,
            "range": "± 14688",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5962,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50695,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 98934,
            "range": "± 534",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4375,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 33993,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 69493,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70211,
            "range": "± 529",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1091456,
            "range": "± 21025",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2182836,
            "range": "± 12737",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13461740,
            "range": "± 154819",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 90893714,
            "range": "± 444501",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 174500670,
            "range": "± 499618",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9924,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2509,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 450,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 923,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2787,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8032,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21034,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28762,
            "range": "± 164",
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
          "id": "ee329ceae837ebc510e5c9a9b77cf2b37f65860e",
          "message": "Remove some unused dependencies (#502)",
          "timestamp": "2024-10-07T11:42:04-07:00",
          "tree_id": "75341c17809646d7ad3495437d065d2c9b912913",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/ee329ceae837ebc510e5c9a9b77cf2b37f65860e"
        },
        "date": 1728327275675,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 813580,
            "range": "± 2581",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 874135,
            "range": "± 9462",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 834378,
            "range": "± 16561",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 868442,
            "range": "± 17108",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 839033,
            "range": "± 2413",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 877366,
            "range": "± 2075",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 843249,
            "range": "± 3262",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 884298,
            "range": "± 4461",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 840443,
            "range": "± 2392",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 874763,
            "range": "± 6113",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 989994,
            "range": "± 4709",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1269607,
            "range": "± 14409",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1866397,
            "range": "± 32696",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1304722,
            "range": "± 17259",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1579541,
            "range": "± 8088",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2174657,
            "range": "± 9232",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6550,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51925,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 100620,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4303,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32957,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 68246,
            "range": "± 972",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67789,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1064079,
            "range": "± 14587",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2133480,
            "range": "± 10208",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13005971,
            "range": "± 128446",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 89519147,
            "range": "± 401778",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 172390848,
            "range": "± 2300370",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9920,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2618,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 487,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 932,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2730,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8067,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20886,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28417,
            "range": "± 316",
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
          "id": "05793bd2413d57bb737993b139b393393464ecbe",
          "message": "Add a source location for UnexpectedEndOfInput parse error (#504)",
          "timestamp": "2024-10-09T10:29:40-07:00",
          "tree_id": "5ebe9b367520705f7c03e90c5b065c241965d40e",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/05793bd2413d57bb737993b139b393393464ecbe"
        },
        "date": 1728495777619,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 758512,
            "range": "± 10612",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 848621,
            "range": "± 7200",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 810473,
            "range": "± 80417",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 844508,
            "range": "± 2627",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 813068,
            "range": "± 4190",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 848537,
            "range": "± 2362",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 814747,
            "range": "± 6219",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 858958,
            "range": "± 3691",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 809872,
            "range": "± 1987",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 850432,
            "range": "± 2330",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 958506,
            "range": "± 4227",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1220172,
            "range": "± 14437",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1831510,
            "range": "± 130173",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1221445,
            "range": "± 19010",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1510386,
            "range": "± 14587",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2106217,
            "range": "± 9047",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6231,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51206,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 99227,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4231,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 31875,
            "range": "± 236",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 65414,
            "range": "± 1965",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67815,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1051828,
            "range": "± 9443",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2110985,
            "range": "± 9730",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12934632,
            "range": "± 204154",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 89239951,
            "range": "± 261802",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 171190106,
            "range": "± 440487",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9933,
            "range": "± 1046",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2559,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 454,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 936,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2777,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7987,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20795,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28616,
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
          "id": "36b90c5eb25e9f45823d144a7c5e0c3d02d8aae5",
          "message": "Add pretty-printing for `Value` (#503)\n\n* Move generic `pretty` extensions to `partiql-common`\r\n* Add pretty-printing for `Value`",
          "timestamp": "2024-10-09T16:05:24-07:00",
          "tree_id": "e379ce32e6f9837ecb05a2b3ae5236d1163f58a8",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/36b90c5eb25e9f45823d144a7c5e0c3d02d8aae5"
        },
        "date": 1728515870892,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 757550,
            "range": "± 14135",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 846820,
            "range": "± 4815",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 804760,
            "range": "± 17690",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 838944,
            "range": "± 3186",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 810775,
            "range": "± 2512",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 843615,
            "range": "± 11284",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 817235,
            "range": "± 5860",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 854241,
            "range": "± 29133",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 810277,
            "range": "± 3458",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 844957,
            "range": "± 23645",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 961188,
            "range": "± 5157",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1204105,
            "range": "± 13924",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1832649,
            "range": "± 9959",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1262413,
            "range": "± 14368",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1534439,
            "range": "± 28757",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2116623,
            "range": "± 6642",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6042,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 49793,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 93069,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4169,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 31690,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 66238,
            "range": "± 456",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 66874,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1055270,
            "range": "± 20019",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2112655,
            "range": "± 10183",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13247704,
            "range": "± 391970",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 95034828,
            "range": "± 947484",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 182929663,
            "range": "± 493150",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9839,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2590,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 474,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 820,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2667,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8040,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21267,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28958,
            "range": "± 102",
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
          "id": "3f9d17fb38edafb23c1908af27b06c78d66680cd",
          "message": "Add validation for comparison operations (#505)",
          "timestamp": "2024-10-11T10:19:49-07:00",
          "tree_id": "93a3874fb686e3b65db9c9c04149b3b23317fb9f",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/3f9d17fb38edafb23c1908af27b06c78d66680cd"
        },
        "date": 1728667945158,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 765428,
            "range": "± 17728",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 858820,
            "range": "± 20625",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 814540,
            "range": "± 16330",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 852711,
            "range": "± 3888",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 819503,
            "range": "± 5160",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 856574,
            "range": "± 4490",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 827442,
            "range": "± 1701",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 864722,
            "range": "± 1752",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 821912,
            "range": "± 11050",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 857937,
            "range": "± 2288",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 968498,
            "range": "± 45038",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1221287,
            "range": "± 4423",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1838260,
            "range": "± 4472",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1266617,
            "range": "± 22698",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1556324,
            "range": "± 8387",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2188553,
            "range": "± 5721",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6102,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50272,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 99654,
            "range": "± 3473",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4225,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32652,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 67918,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 69724,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1094861,
            "range": "± 9322",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2191116,
            "range": "± 13765",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 13197485,
            "range": "± 249211",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 89259168,
            "range": "± 934982",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 171668148,
            "range": "± 423756",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10085,
            "range": "± 421",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2543,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 475,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 937,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2733,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7922,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20662,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28614,
            "range": "± 227",
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
          "id": "3f3a948430868c659600304840e0a4fe6231c1b9",
          "message": "Lowercase before snake-casing for conformance test string escaping. (#511)\n\nReplace names like: strict_example_7_nu_l_l_and_missin_g_coercion_1\r\n              with: strict_example_7_null_and_missing_coercion_1",
          "timestamp": "2024-10-22T14:36:38-07:00",
          "tree_id": "704ea9db2ee114b7036ce2617d1358451be8188f",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/3f3a948430868c659600304840e0a4fe6231c1b9"
        },
        "date": 1729633738673,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 782025,
            "range": "± 7382",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 873100,
            "range": "± 20686",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 830251,
            "range": "± 18843",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 867471,
            "range": "± 1743",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 836826,
            "range": "± 3549",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 871780,
            "range": "± 3362",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 841277,
            "range": "± 2470",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 879427,
            "range": "± 2659",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 836546,
            "range": "± 3222",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 879618,
            "range": "± 5498",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 979416,
            "range": "± 2352",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1230521,
            "range": "± 4196",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1840114,
            "range": "± 24634",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1259725,
            "range": "± 30318",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1533973,
            "range": "± 58989",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2118265,
            "range": "± 7769",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5897,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50320,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 99007,
            "range": "± 955",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4194,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32373,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 66932,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70626,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1095103,
            "range": "± 20141",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2191097,
            "range": "± 6803",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12926133,
            "range": "± 122547",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 88215128,
            "range": "± 258292",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 169498996,
            "range": "± 2835589",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10213,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2539,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 457,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 914,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2676,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8178,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21128,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28822,
            "range": "± 95",
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
          "id": "d3405e1e704cfa45adeb477228f1ea8ab5144632",
          "message": "Update to latest partiql-tests (#512)",
          "timestamp": "2024-10-22T15:17:54-07:00",
          "tree_id": "71b072b0be04e6861c40d071f5d54cce7e3664e6",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/d3405e1e704cfa45adeb477228f1ea8ab5144632"
        },
        "date": 1729636211458,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 780478,
            "range": "± 2580",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 872665,
            "range": "± 3317",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 826886,
            "range": "± 26373",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 872474,
            "range": "± 5013",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 832383,
            "range": "± 8178",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 874545,
            "range": "± 3316",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 841764,
            "range": "± 3082",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 884128,
            "range": "± 3338",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 833708,
            "range": "± 2956",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 878481,
            "range": "± 5216",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 979415,
            "range": "± 4282",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1238008,
            "range": "± 20467",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1852445,
            "range": "± 7288",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1253725,
            "range": "± 5665",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1542114,
            "range": "± 7312",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2154465,
            "range": "± 20408",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6117,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50407,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 99011,
            "range": "± 492",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4237,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32291,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 66655,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70678,
            "range": "± 781",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1094456,
            "range": "± 20699",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2195345,
            "range": "± 3870",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12949078,
            "range": "± 85697",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 88448994,
            "range": "± 1012969",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 169529483,
            "range": "± 459401",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9851,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2526,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 474,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 945,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2697,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8064,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21116,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28061,
            "range": "± 315",
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
          "id": "a7993de8315cbd2828da00f8e47e59a4aab26e2c",
          "message": "Update partiql-tests to latest (#513)",
          "timestamp": "2024-10-22T15:30:59-07:00",
          "tree_id": "579b8f3110e1ad62c7b8a711b4655c7f86f5f0d6",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/a7993de8315cbd2828da00f8e47e59a4aab26e2c"
        },
        "date": 1729636993390,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 778876,
            "range": "± 6563",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 865101,
            "range": "± 10684",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 832969,
            "range": "± 21135",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 865241,
            "range": "± 3034",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 830045,
            "range": "± 2296",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 866901,
            "range": "± 1679",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 836447,
            "range": "± 4347",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 875477,
            "range": "± 2571",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 837285,
            "range": "± 1921",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 874275,
            "range": "± 5198",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 978145,
            "range": "± 19364",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1224127,
            "range": "± 8025",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1852330,
            "range": "± 7671",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1261519,
            "range": "± 21633",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1532962,
            "range": "± 39963",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2127338,
            "range": "± 13304",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6213,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50489,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 98652,
            "range": "± 498",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4247,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 32835,
            "range": "± 1533",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 67473,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70366,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1091990,
            "range": "± 47002",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2193890,
            "range": "± 10482",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12862783,
            "range": "± 81581",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 88439210,
            "range": "± 934873",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 169215402,
            "range": "± 486929",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10009,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2500,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 457,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 858,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2563,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7927,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20694,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28148,
            "range": "± 109",
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
          "id": "13836d15c2f86c1e0a7d949645eb0c16d8b9c295",
          "message": "Fix build & advisory issues. (#521)\n\n* Update deny license versions, nightly toolchain\r\n* Replace usages of `derivative` with `educe`.\r\n* clippy & fmt",
          "timestamp": "2024-12-05T10:55:04-08:00",
          "tree_id": "999eb186936cd314540863f8dbfd614658baddce",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/13836d15c2f86c1e0a7d949645eb0c16d8b9c295"
        },
        "date": 1733425634546,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 779362,
            "range": "± 4869",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 868514,
            "range": "± 20412",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 827765,
            "range": "± 52033",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 859464,
            "range": "± 3769",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 831144,
            "range": "± 24332",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 865575,
            "range": "± 160042",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 835732,
            "range": "± 1378",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 872876,
            "range": "± 1396",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 831858,
            "range": "± 1660",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 866734,
            "range": "± 5909",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 988615,
            "range": "± 10610",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1244835,
            "range": "± 12282",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1855176,
            "range": "± 7861",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1220105,
            "range": "± 8355",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1507054,
            "range": "± 43127",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2102096,
            "range": "± 17003",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5954,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 49279,
            "range": "± 1112",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 96811,
            "range": "± 951",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4313,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 31572,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 64409,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70716,
            "range": "± 1698",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1102453,
            "range": "± 36687",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2142597,
            "range": "± 46164",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12147533,
            "range": "± 87548",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 76679910,
            "range": "± 3978698",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 146225132,
            "range": "± 1141375",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10119,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2541,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 477,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 951,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2765,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8026,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20927,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28651,
            "range": "± 137",
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
          "id": "4a4b1438c4540d6f552b065340edff2f55f6d3fb",
          "message": "Update partiql-tests (#522)",
          "timestamp": "2024-12-05T10:58:27-08:00",
          "tree_id": "6fe029d2c09300768354dd3784f29f90d38da305",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/4a4b1438c4540d6f552b065340edff2f55f6d3fb"
        },
        "date": 1733425835001,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 790105,
            "range": "± 11027",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 866700,
            "range": "± 2429",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 833098,
            "range": "± 8364",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 868307,
            "range": "± 4014",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 840261,
            "range": "± 5673",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 870972,
            "range": "± 40419",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 844735,
            "range": "± 6067",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 878921,
            "range": "± 22966",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 840506,
            "range": "± 7062",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 868109,
            "range": "± 11415",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 993679,
            "range": "± 4817",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1261944,
            "range": "± 11475",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1869825,
            "range": "± 11386",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1228616,
            "range": "± 6146",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1515867,
            "range": "± 115785",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2129616,
            "range": "± 9534",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6130,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51127,
            "range": "± 2468",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 99090,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4399,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 31298,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 64065,
            "range": "± 3328",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70921,
            "range": "± 1513",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1102491,
            "range": "± 48335",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2193118,
            "range": "± 10706",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12146862,
            "range": "± 99488",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 76690162,
            "range": "± 364929",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 146088058,
            "range": "± 456686",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10025,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2569,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 508,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 971,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2601,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8076,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 21305,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28541,
            "range": "± 1102",
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
          "id": "a07bdfcc17726a8bde658bf95ed96da2a93fc9fc",
          "message": "Refactor Value & Enable Lazy Ion literals. (#523)\n\n* Change Lexing/Parsing of embedded docs to not eagerly validate (#507)\r\n\r\nThis changes the lexer and parser to pass through strings enclosed in backticks un-parsed. (At current, these documents are parsed during lowering).\r\n\r\nSince embedded documents may themselves contain backticks, beginning and ending delimiters consist of an arbitrary odd numbers of backticks (e.g., `` ` ``, `` ``` ``, `` ````` `` etc.) that must be paired (e.g., `` `$ion_data_here::[]` ``, `` ```$ion_data_here::[ $string_with_embedded_backtick:\"`\" ]``` ``, etc.).\r\n\r\nAs opening and closing delimiters are required to be odd in count of backticks, a contiguous string of backticks that is even is interpreted as an empty document.\r\n\r\n* Behavior-preserving refactor of `Value` into a module. (#509)\r\n\r\n* Behavior-preserving refactor of Value into a module. (#510)\r\n\r\n* Change modeling of Literals in the AST remove ambiguity (#517)\r\n\r\nChange parsing and AST-modeling of literals to not share AST structures with non-scalar expressions.\r\n\r\n* Change modeling of boxed ion literals to be lazy until evaluator. (#519)\r\n\r\nChanges the logical plan to have a distinct `Lit` type to hold literals instead of embedded `Value`\r\n\r\n* Refactor lifetimes for new rust warnings",
          "timestamp": "2024-12-05T13:55:25-08:00",
          "tree_id": "7d0f41e7c315654ae4d22c25fd22e5845e2ad7cf",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/a07bdfcc17726a8bde658bf95ed96da2a93fc9fc"
        },
        "date": 1733436452869,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 765417,
            "range": "± 9720",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 857628,
            "range": "± 2290",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 823325,
            "range": "± 14147",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 849976,
            "range": "± 1321",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 825972,
            "range": "± 45164",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 852656,
            "range": "± 2511",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 830428,
            "range": "± 5999",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 860720,
            "range": "± 30809",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 826283,
            "range": "± 12568",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 856047,
            "range": "± 5027",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 995733,
            "range": "± 12537",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1221238,
            "range": "± 30151",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1843394,
            "range": "± 7066",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1202863,
            "range": "± 32389",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1478093,
            "range": "± 36300",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2072542,
            "range": "± 9664",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6165,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51563,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 98311,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4246,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30444,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63588,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70683,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1101877,
            "range": "± 15617",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2204988,
            "range": "± 16988",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 11996073,
            "range": "± 261515",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 76529036,
            "range": "± 455327",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 146551997,
            "range": "± 516782",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9914,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2486,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 460,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 961,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2621,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7912,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20359,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 27825,
            "range": "± 93",
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
          "id": "d07a7e1e02f4773777f5e066433492f30e0ec3ff",
          "message": "Make `bind` consuming (#524)",
          "timestamp": "2024-12-05T14:19:44-08:00",
          "tree_id": "3a0a60499181358489181e6c5fb4e03d8f3512a9",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/d07a7e1e02f4773777f5e066433492f30e0ec3ff"
        },
        "date": 1733437915671,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 770065,
            "range": "± 6886",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 856721,
            "range": "± 3282",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 815223,
            "range": "± 14799",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 858669,
            "range": "± 4273",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 821056,
            "range": "± 2841",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 854399,
            "range": "± 4373",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 825900,
            "range": "± 41288",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 861984,
            "range": "± 2669",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 821892,
            "range": "± 6469",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 860284,
            "range": "± 2356",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 973599,
            "range": "± 2729",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1226677,
            "range": "± 16934",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1852614,
            "range": "± 6514",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1224367,
            "range": "± 13913",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1500889,
            "range": "± 19101",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2096925,
            "range": "± 9815",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6086,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50565,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 98481,
            "range": "± 429",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4280,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30186,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63029,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 71625,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1124229,
            "range": "± 10520",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2253137,
            "range": "± 14872",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 11911863,
            "range": "± 111520",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 77114178,
            "range": "± 293901",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 147487722,
            "range": "± 486288",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9733,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2542,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 481,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 941,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2598,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 8131,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20806,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28512,
            "range": "± 2711",
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
          "id": "27d90d174c78fdbee7841fb68a7d3f52851d3bcd",
          "message": "Add type to `ast::Lit::EmbeddedDocLit` (#525)",
          "timestamp": "2024-12-05T15:59:05-08:00",
          "tree_id": "3e5019301f432584164efa49f431757e942cfa05",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/27d90d174c78fdbee7841fb68a7d3f52851d3bcd"
        },
        "date": 1733443873853,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 762607,
            "range": "± 13004",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 853107,
            "range": "± 13446",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 811102,
            "range": "± 52377",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 841861,
            "range": "± 9876",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 813170,
            "range": "± 11329",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 852586,
            "range": "± 13113",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 822112,
            "range": "± 18721",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 861600,
            "range": "± 12184",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 817410,
            "range": "± 7116",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 846422,
            "range": "± 17409",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 980679,
            "range": "± 11071",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1236545,
            "range": "± 41584",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1851597,
            "range": "± 26232",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1204575,
            "range": "± 13223",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1505227,
            "range": "± 11252",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2123329,
            "range": "± 20173",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6002,
            "range": "± 642",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 50598,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 99235,
            "range": "± 404",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4106,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 31139,
            "range": "± 601",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63510,
            "range": "± 516",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70176,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1093985,
            "range": "± 9281",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2187788,
            "range": "± 7953",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12010339,
            "range": "± 71348",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 76945710,
            "range": "± 1003377",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 147299495,
            "range": "± 1612420",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9927,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2523,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 482,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 953,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2703,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7970,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20582,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28121,
            "range": "± 160",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fchuhan@amazon.com",
            "name": "Chuhan Feng",
            "username": "simonrouse9461"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "52653764de0eab6598c0f2c9aecf1840d76198a0",
          "message": "Bump upload and download artifact versions (#528)",
          "timestamp": "2024-12-18T17:47:35-08:00",
          "tree_id": "9f9bb1753a712d1fc123bfb929038fdca5ec3e15",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/52653764de0eab6598c0f2c9aecf1840d76198a0"
        },
        "date": 1734573600193,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 773932,
            "range": "± 6814",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 855520,
            "range": "± 14896",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 818071,
            "range": "± 17412",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 849442,
            "range": "± 3507",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 822004,
            "range": "± 2424",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 853339,
            "range": "± 10328",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 828995,
            "range": "± 9784",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 861131,
            "range": "± 2886",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 825237,
            "range": "± 2225",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 855031,
            "range": "± 3639",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 976810,
            "range": "± 10710",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1241180,
            "range": "± 6017",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1859476,
            "range": "± 33072",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1193405,
            "range": "± 6998",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1454080,
            "range": "± 7445",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2065311,
            "range": "± 6156",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6171,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 51206,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 98373,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4199,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30584,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63289,
            "range": "± 473",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70864,
            "range": "± 345",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1100699,
            "range": "± 48597",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2205730,
            "range": "± 18177",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12652855,
            "range": "± 173075",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 78722422,
            "range": "± 2049053",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 149051556,
            "range": "± 679275",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9873,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2574,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 450,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 979,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2666,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7797,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20610,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28041,
            "range": "± 137",
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
          "id": "3bcbd2d41ee1cfea8606dd060d1d36c1b6a5bc27",
          "message": "Try fixing GH conformance report target branch (#530)",
          "timestamp": "2025-01-03T10:35:52-08:00",
          "tree_id": "4c363cd3a24deb9fdeb2f6963d1e60b9da3a7d6a",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/3bcbd2d41ee1cfea8606dd060d1d36c1b6a5bc27"
        },
        "date": 1735930098860,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 775357,
            "range": "± 3486",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 851451,
            "range": "± 17128",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 819766,
            "range": "± 14831",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 845407,
            "range": "± 2536",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 825374,
            "range": "± 4595",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 847956,
            "range": "± 5507",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 828738,
            "range": "± 21200",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 860452,
            "range": "± 30143",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 821936,
            "range": "± 19273",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 846130,
            "range": "± 3172",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 965715,
            "range": "± 2888",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1236946,
            "range": "± 7069",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1829539,
            "range": "± 27329",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1196788,
            "range": "± 27611",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1456576,
            "range": "± 12208",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2080475,
            "range": "± 64566",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6009,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 49550,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 97891,
            "range": "± 449",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4406,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30729,
            "range": "± 512",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63513,
            "range": "± 754",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67492,
            "range": "± 866",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1050170,
            "range": "± 14683",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2097509,
            "range": "± 15284",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12336116,
            "range": "± 142122",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 77700703,
            "range": "± 870774",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 148051909,
            "range": "± 624231",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10297,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2589,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 480,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 947,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2618,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7802,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20492,
            "range": "± 804",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 28492,
            "range": "± 316",
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
          "id": "b965c45f11c3f0661eb0a0c8560bcf06112f562d",
          "message": "Fix warnings/errors newly surfaced by rust/clippy 1.84 (#532)",
          "timestamp": "2025-01-10T13:53:14-08:00",
          "tree_id": "e69a7ad546c53f393250dbf8a835025f4e2a1a61",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/b965c45f11c3f0661eb0a0c8560bcf06112f562d"
        },
        "date": 1736546741529,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 766247,
            "range": "± 18752",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 854173,
            "range": "± 21056",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 810955,
            "range": "± 23860",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 849671,
            "range": "± 10308",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 818696,
            "range": "± 2282",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 853340,
            "range": "± 5638",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 822208,
            "range": "± 2273",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 859653,
            "range": "± 4318",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 818895,
            "range": "± 4759",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 856024,
            "range": "± 2834",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 972679,
            "range": "± 12670",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1233558,
            "range": "± 22133",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1845900,
            "range": "± 61781",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1172846,
            "range": "± 38661",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1432505,
            "range": "± 16806",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2049718,
            "range": "± 8971",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6301,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 49139,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 99922,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4242,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30918,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63390,
            "range": "± 497",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 72590,
            "range": "± 511",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1132456,
            "range": "± 6187",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2278769,
            "range": "± 9474",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12654517,
            "range": "± 492500",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 77595784,
            "range": "± 384744",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 147319643,
            "range": "± 790643",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10121,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2532,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 499,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 842,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2730,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7899,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20983,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 27810,
            "range": "± 158",
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
          "id": "1b19d11b67247a412851622af53f5d18194d7e37",
          "message": "Fix conformance test name generation for mixed camel-/snake-case (#531)",
          "timestamp": "2025-01-10T13:53:56-08:00",
          "tree_id": "a4ea6f7227fc1fe877bb4d001dee2e6f287d1168",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/1b19d11b67247a412851622af53f5d18194d7e37"
        },
        "date": 1736546773124,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 766648,
            "range": "± 4238",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 861535,
            "range": "± 1550",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 815139,
            "range": "± 12287",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 854865,
            "range": "± 1904",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 819999,
            "range": "± 2348",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 857922,
            "range": "± 1733",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 825238,
            "range": "± 3604",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 866332,
            "range": "± 2757",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 820739,
            "range": "± 3041",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 864789,
            "range": "± 2361",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 972890,
            "range": "± 3096",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1243160,
            "range": "± 11355",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1849620,
            "range": "± 10064",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1186602,
            "range": "± 4488",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1467134,
            "range": "± 7227",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2067090,
            "range": "± 14009",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 6571,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 54306,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 105393,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4193,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30585,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 62898,
            "range": "± 408",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 72782,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1130914,
            "range": "± 79606",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2264952,
            "range": "± 11751",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 11877539,
            "range": "± 51184",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 75970932,
            "range": "± 3530087",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 145051263,
            "range": "± 959139",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 10004,
            "range": "± 453",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2486,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 459,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 884,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2611,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7809,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 20394,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 27578,
            "range": "± 158",
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
          "id": "76764b799d20d04fb2febfc947435bce2431b1ba",
          "message": "Refactor ShapeBuilder away from a single shared global value (#491)",
          "timestamp": "2025-01-13T14:15:16-08:00",
          "tree_id": "5c903b3a67758798704e4d760d9af3640d0a43ae",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/76764b799d20d04fb2febfc947435bce2431b1ba"
        },
        "date": 1736807261621,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 770153,
            "range": "± 2588",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 859709,
            "range": "± 8950",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 820097,
            "range": "± 16502",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 851393,
            "range": "± 6059",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 825236,
            "range": "± 2850",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 855755,
            "range": "± 6185",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 835713,
            "range": "± 32001",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 867964,
            "range": "± 15459",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 828048,
            "range": "± 2813",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 859180,
            "range": "± 2473",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 977325,
            "range": "± 4155",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1240333,
            "range": "± 4115",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1836969,
            "range": "± 23523",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1178308,
            "range": "± 12217",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1448869,
            "range": "± 19573",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2044763,
            "range": "± 5312",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5434,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 47958,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 92866,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4178,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 31003,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63253,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 67876,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1069256,
            "range": "± 24766",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2193025,
            "range": "± 37115",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12079025,
            "range": "± 163991",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 77791834,
            "range": "± 1120096",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 147818887,
            "range": "± 2140975",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9949,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2592,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 486,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 704,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2316,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7264,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 18563,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 26129,
            "range": "± 126",
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
          "id": "e21bd72abc4380a0a347ba7fa56fbde8fdb69440",
          "message": "Introduce Datum, an interface to introspecting Values (#533)\n\n* Introduce `Datum`, an interface to introspecting `Value`s\n\n* Use `Datum` and `DatumCategory` in evaluator",
          "timestamp": "2025-01-14T13:34:01-08:00",
          "tree_id": "4ce86fd11a470f9bfffdb7674e2bd66d2762791d",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/e21bd72abc4380a0a347ba7fa56fbde8fdb69440"
        },
        "date": 1736891179408,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 775516,
            "range": "± 2472",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 860718,
            "range": "± 3610",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 825762,
            "range": "± 13338",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 856013,
            "range": "± 2178",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 827807,
            "range": "± 3572",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 863961,
            "range": "± 2032",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 833242,
            "range": "± 13387",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 872569,
            "range": "± 2150",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 828631,
            "range": "± 2553",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 859246,
            "range": "± 3067",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 988641,
            "range": "± 11710",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1257536,
            "range": "± 16268",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1872303,
            "range": "± 7934",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1194270,
            "range": "± 12517",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1479657,
            "range": "± 6104",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2077505,
            "range": "± 13519",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5522,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 46801,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 91843,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4343,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 31329,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63609,
            "range": "± 1169",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 71217,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1102206,
            "range": "± 11946",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2194003,
            "range": "± 12838",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 11976854,
            "range": "± 103202",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 78254362,
            "range": "± 1207080",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 148466525,
            "range": "± 476208",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9902,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2540,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 488,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 746,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2294,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7003,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 18215,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 25649,
            "range": "± 253",
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
          "id": "a409335d1ba0340461636d39b4e0db746b635b83",
          "message": "Refactor matching of `BindingsName` (#534)",
          "timestamp": "2025-01-14T13:48:45-08:00",
          "tree_id": "23ba2cd93092042a6813a9931b53ae10a25b593b",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/a409335d1ba0340461636d39b4e0db746b635b83"
        },
        "date": 1736892072445,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 772276,
            "range": "± 3437",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 858566,
            "range": "± 4128",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 827877,
            "range": "± 13071",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 852895,
            "range": "± 6794",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 828341,
            "range": "± 5418",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 857045,
            "range": "± 2362",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 827910,
            "range": "± 3246",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 865956,
            "range": "± 16577",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 824878,
            "range": "± 5881",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 859240,
            "range": "± 4802",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 990177,
            "range": "± 17058",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1291903,
            "range": "± 6879",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1906669,
            "range": "± 8973",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1200477,
            "range": "± 18692",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1495367,
            "range": "± 31044",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2110307,
            "range": "± 9760",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5373,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 46819,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 89844,
            "range": "± 308",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4182,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30764,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63231,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70099,
            "range": "± 1332",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1088274,
            "range": "± 8357",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2255612,
            "range": "± 46211",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12213604,
            "range": "± 145987",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 78102224,
            "range": "± 491542",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 148626494,
            "range": "± 839657",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9902,
            "range": "± 275",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2466,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 467,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 755,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2242,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7039,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 18804,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 26127,
            "range": "± 76",
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
          "id": "3647c563ce7b9b10a7e94af4aa79bb5e03136391",
          "message": "Add `category` for `PartiqlShape` (#535)",
          "timestamp": "2025-01-14T15:20:10-08:00",
          "tree_id": "d7243c1a1e8e5eb6857c5f9b15f6930c243112c4",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/3647c563ce7b9b10a7e94af4aa79bb5e03136391"
        },
        "date": 1736897554353,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 778823,
            "range": "± 4235",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 862469,
            "range": "± 14558",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 824616,
            "range": "± 20684",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 862385,
            "range": "± 25442",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 829546,
            "range": "± 3251",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 867717,
            "range": "± 9625",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 839271,
            "range": "± 3509",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 867308,
            "range": "± 3488",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 829345,
            "range": "± 2584",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 863063,
            "range": "± 17521",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 1007676,
            "range": "± 6642",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1262956,
            "range": "± 28389",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1895714,
            "range": "± 7046",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1204549,
            "range": "± 11239",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1483475,
            "range": "± 12157",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2081734,
            "range": "± 8508",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5605,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 47958,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 94102,
            "range": "± 428",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4242,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30977,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 63911,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70381,
            "range": "± 503",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1096120,
            "range": "± 6850",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2193417,
            "range": "± 17025",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12444594,
            "range": "± 193410",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 78434654,
            "range": "± 1473407",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 149096079,
            "range": "± 763485",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9890,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2498,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 460,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 760,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2271,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7149,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 18629,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 25853,
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
          "id": "59dbbcce3d0310b62fcdde9ed3666d76a52c57ea",
          "message": "Silence some spurious clippy warnings. (#538)",
          "timestamp": "2025-01-15T15:15:17-08:00",
          "tree_id": "22497397e9cd2ead62cf1fa16180d17cefab27e8",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/59dbbcce3d0310b62fcdde9ed3666d76a52c57ea"
        },
        "date": 1736983662519,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 773594,
            "range": "± 4076",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 857665,
            "range": "± 5095",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 822309,
            "range": "± 44600",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 852355,
            "range": "± 10718",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 826952,
            "range": "± 39267",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 866099,
            "range": "± 32879",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 834860,
            "range": "± 30814",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 868044,
            "range": "± 3176",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 832122,
            "range": "± 2947",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 860507,
            "range": "± 8300",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 990002,
            "range": "± 6705",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1278229,
            "range": "± 28446",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1870588,
            "range": "± 10252",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1191863,
            "range": "± 16141",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1461233,
            "range": "± 40502",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2075422,
            "range": "± 12211",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5436,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 47354,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 92881,
            "range": "± 419",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4142,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30301,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 62895,
            "range": "± 743",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 69940,
            "range": "± 1149",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1097476,
            "range": "± 80703",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2192204,
            "range": "± 9567",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12168571,
            "range": "± 111834",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 78372842,
            "range": "± 1204765",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 149908935,
            "range": "± 1993148",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9891,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2558,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 486,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 766,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2495,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 7387,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 19209,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 26160,
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
          "id": "4792d1c4bb23f501d5e9a63f50a9ecade191463d",
          "message": "Refactor equality as per spec. `eqg` and add testing equality ops (#537)",
          "timestamp": "2025-01-15T15:25:43-08:00",
          "tree_id": "e59573824be996f807136cdf9e0a069a6738ce6d",
          "url": "https://github.com/partiql/partiql-lang-rust/commit/4792d1c4bb23f501d5e9a63f50a9ecade191463d"
        },
        "date": 1736984307915,
        "tool": "cargo",
        "benches": [
          {
            "name": "arith_agg-avg",
            "value": 799180,
            "range": "± 10763",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct",
            "value": 882685,
            "range": "± 2463",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count",
            "value": 854073,
            "range": "± 16094",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-count_distinct",
            "value": 878784,
            "range": "± 3765",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min",
            "value": 856840,
            "range": "± 5485",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-min_distinct",
            "value": 882859,
            "range": "± 3213",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max",
            "value": 860209,
            "range": "± 4433",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-max_distinct",
            "value": 888296,
            "range": "± 2830",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum",
            "value": 859308,
            "range": "± 10047",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-sum_distinct",
            "value": 880882,
            "range": "± 9083",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum",
            "value": 1022309,
            "range": "± 7418",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by",
            "value": 1299778,
            "range": "± 9948",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg-count-min-max-sum-group_by-group_as",
            "value": 1935932,
            "range": "± 9387",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct",
            "value": 1235222,
            "range": "± 11077",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by",
            "value": 1529091,
            "range": "± 10680",
            "unit": "ns/iter"
          },
          {
            "name": "arith_agg-avg_distinct-count_distinct-min_distinct-max_distinct-sum_distinct-group_by-group_as",
            "value": 2149568,
            "range": "± 16375",
            "unit": "ns/iter"
          },
          {
            "name": "parse-1",
            "value": 5441,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "parse-15",
            "value": 46367,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "parse-30",
            "value": 90639,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "compile-1",
            "value": 4295,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "compile-15",
            "value": 30685,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "compile-30",
            "value": 64239,
            "range": "± 2548",
            "unit": "ns/iter"
          },
          {
            "name": "plan-1",
            "value": 70894,
            "range": "± 387",
            "unit": "ns/iter"
          },
          {
            "name": "plan-15",
            "value": 1105696,
            "range": "± 10190",
            "unit": "ns/iter"
          },
          {
            "name": "plan-30",
            "value": 2213383,
            "range": "± 25125",
            "unit": "ns/iter"
          },
          {
            "name": "eval-1",
            "value": 12809424,
            "range": "± 223288",
            "unit": "ns/iter"
          },
          {
            "name": "eval-15",
            "value": 79139260,
            "range": "± 1088736",
            "unit": "ns/iter"
          },
          {
            "name": "eval-30",
            "value": 149902007,
            "range": "± 740154",
            "unit": "ns/iter"
          },
          {
            "name": "join",
            "value": 9971,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "simple",
            "value": 2565,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-no",
            "value": 465,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "numbers",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse-simple",
            "value": 736,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse-ion",
            "value": 2304,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse-group",
            "value": 6892,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex",
            "value": 18554,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "parse-complex-fexpr",
            "value": 25476,
            "range": "± 203",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}