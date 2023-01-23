window.BENCHMARK_DATA = {
  "lastUpdate": 1674509179747,
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
      }
    ]
  }
}