window.BENCHMARK_DATA = {
  "lastUpdate": 1675812233570,
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
      }
    ]
  }
}