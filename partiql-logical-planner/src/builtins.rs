use itertools::Itertools;
use once_cell::sync::Lazy;
use partiql_logical as logical;
use partiql_logical::ValueExpr;
use partiql_value::Value;
use std::collections::HashMap;
use std::fmt::Debug;

use partiql_catalog::call_defs::{CallDef, CallSpec, CallSpecArg};
use unicase::UniCase;

fn function_call_def_char_len() -> CallDef {
    CallDef {
        names: vec!["char_length", "character_length"],
        overloads: vec![CallSpec {
            input: vec![CallSpecArg::Positional],
            output: Box::new(|args| {
                logical::ValueExpr::Call(logical::CallExpr {
                    name: logical::CallName::CharLength,
                    arguments: args,
                })
            }),
        }],
    }
}

fn function_call_def_octet_len() -> CallDef {
    CallDef {
        names: vec!["octet_length"],
        overloads: vec![CallSpec {
            input: vec![CallSpecArg::Positional],
            output: Box::new(|args| {
                logical::ValueExpr::Call(logical::CallExpr {
                    name: logical::CallName::OctetLength,
                    arguments: args,
                })
            }),
        }],
    }
}

fn function_call_def_bit_len() -> CallDef {
    CallDef {
        names: vec!["bit_length"],
        overloads: vec![CallSpec {
            input: vec![CallSpecArg::Positional],
            output: Box::new(|args| {
                logical::ValueExpr::Call(logical::CallExpr {
                    name: logical::CallName::BitLength,
                    arguments: args,
                })
            }),
        }],
    }
}

fn function_call_def_lower() -> CallDef {
    CallDef {
        names: vec!["lower"],
        overloads: vec![CallSpec {
            input: vec![CallSpecArg::Positional],
            output: Box::new(|args| {
                logical::ValueExpr::Call(logical::CallExpr {
                    name: logical::CallName::Lower,
                    arguments: args,
                })
            }),
        }],
    }
}

fn function_call_def_upper() -> CallDef {
    CallDef {
        names: vec!["upper"],
        overloads: vec![CallSpec {
            input: vec![CallSpecArg::Positional],
            output: Box::new(|args| {
                logical::ValueExpr::Call(logical::CallExpr {
                    name: logical::CallName::Upper,
                    arguments: args,
                })
            }),
        }],
    }
}

fn function_call_def_substring() -> CallDef {
    CallDef {
        names: vec!["substring"],
        overloads: vec![
            CallSpec {
                input: vec![
                    CallSpecArg::Positional,
                    CallSpecArg::Positional,
                    CallSpecArg::Positional,
                ],
                output: Box::new(|args| {
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::Substring,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![CallSpecArg::Positional, CallSpecArg::Positional],
                output: Box::new(|args| {
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::Substring,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![
                    CallSpecArg::Positional,
                    CallSpecArg::Named("from".into()),
                    CallSpecArg::Named("for".into()),
                ],
                output: Box::new(|args| {
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::Substring,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![CallSpecArg::Positional, CallSpecArg::Named("from".into())],
                output: Box::new(|args| {
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::Substring,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![CallSpecArg::Positional, CallSpecArg::Named("for".into())],
                output: Box::new(|mut args| {
                    args.insert(1, ValueExpr::Lit(Box::new(Value::Integer(0))));
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::Substring,
                        arguments: args,
                    })
                }),
            },
        ],
    }
}

fn function_call_def_overlay() -> CallDef {
    CallDef {
        names: vec!["overlay"],
        overloads: vec![
            CallSpec {
                input: vec![
                    CallSpecArg::Positional,
                    CallSpecArg::Named("placing".into()),
                    CallSpecArg::Named("from".into()),
                    CallSpecArg::Named("for".into()),
                ],
                output: Box::new(|args| {
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::Overlay,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![
                    CallSpecArg::Positional,
                    CallSpecArg::Named("placing".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|args| {
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::Overlay,
                        arguments: args,
                    })
                }),
            },
        ],
    }
}

fn function_call_def_position() -> CallDef {
    CallDef {
        names: vec!["position"],
        overloads: vec![CallSpec {
            input: vec![CallSpecArg::Positional, CallSpecArg::Named("in".into())],
            output: Box::new(|args| {
                logical::ValueExpr::Call(logical::CallExpr {
                    name: logical::CallName::Position,
                    arguments: args,
                })
            }),
        }],
    }
}

fn function_call_def_trim() -> CallDef {
    CallDef {
        names: vec!["trim"],
        overloads: vec![
            CallSpec {
                input: vec![
                    CallSpecArg::Named("leading".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|args| {
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::LTrim,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![
                    CallSpecArg::Named("trailing".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|args| {
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::RTrim,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![
                    CallSpecArg::Named("both".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|args| {
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::BTrim,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![CallSpecArg::Named("from".into())],
                output: Box::new(|mut args| {
                    args.insert(
                        0,
                        ValueExpr::Lit(Box::new(Value::String(" ".to_string().into()))),
                    );

                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::BTrim,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![CallSpecArg::Positional],
                output: Box::new(|mut args| {
                    args.insert(
                        0,
                        ValueExpr::Lit(Box::new(Value::String(" ".to_string().into()))),
                    );
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::BTrim,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![CallSpecArg::Positional, CallSpecArg::Named("from".into())],
                output: Box::new(|args| {
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::BTrim,
                        arguments: args,
                    })
                }),
            },
        ],
    }
}

fn function_call_def_coalesce() -> CallDef {
    CallDef {
        names: vec!["coalesce"],
        overloads: (0..15)
            .map(|n| CallSpec {
                input: std::iter::repeat(CallSpecArg::Positional)
                    .take(n)
                    .collect_vec(),
                output: Box::new(|args| {
                    logical::ValueExpr::CoalesceExpr(logical::CoalesceExpr { elements: args })
                }),
            })
            .collect_vec(),
    }
}

fn function_call_def_nullif() -> CallDef {
    CallDef {
        names: vec!["nullif"],
        overloads: vec![CallSpec {
            input: vec![CallSpecArg::Positional, CallSpecArg::Positional],
            output: Box::new(|mut args| {
                assert_eq!(args.len(), 2);
                let rhs = Box::new(args.pop().unwrap());
                let lhs = Box::new(args.pop().unwrap());
                logical::ValueExpr::NullIfExpr(logical::NullIfExpr { lhs, rhs })
            }),
        }],
    }
}

fn function_call_def_exists() -> CallDef {
    CallDef {
        names: vec!["exists"],
        overloads: vec![CallSpec {
            input: vec![CallSpecArg::Positional],
            output: Box::new(|args| {
                logical::ValueExpr::Call(logical::CallExpr {
                    name: logical::CallName::Exists,
                    arguments: args,
                })
            }),
        }],
    }
}

fn function_call_def_abs() -> CallDef {
    CallDef {
        names: vec!["abs"],
        overloads: vec![CallSpec {
            input: vec![CallSpecArg::Positional],
            output: Box::new(|args| {
                logical::ValueExpr::Call(logical::CallExpr {
                    name: logical::CallName::Abs,
                    arguments: args,
                })
            }),
        }],
    }
}

fn function_call_def_mod() -> CallDef {
    CallDef {
        names: vec!["mod"],
        overloads: vec![CallSpec {
            input: vec![CallSpecArg::Positional, CallSpecArg::Positional],
            output: Box::new(|args| {
                logical::ValueExpr::Call(logical::CallExpr {
                    name: logical::CallName::Mod,
                    arguments: args,
                })
            }),
        }],
    }
}

fn function_call_def_cardinality() -> CallDef {
    CallDef {
        names: vec!["cardinality"],
        overloads: vec![CallSpec {
            input: vec![CallSpecArg::Positional],
            output: Box::new(|args| {
                logical::ValueExpr::Call(logical::CallExpr {
                    name: logical::CallName::Cardinality,
                    arguments: args,
                })
            }),
        }],
    }
}

fn function_call_def_extract() -> CallDef {
    CallDef {
        names: vec!["extract"],
        overloads: vec![
            CallSpec {
                input: vec![
                    CallSpecArg::Named("year".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|mut args| {
                    args.remove(0); // remove first default synthesized argument from parser preprocessor
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::ExtractYear,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![
                    CallSpecArg::Named("month".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|mut args| {
                    args.remove(0); // remove first default synthesized argument from parser preprocessor
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::ExtractMonth,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![
                    CallSpecArg::Named("day".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|mut args| {
                    args.remove(0); // remove first default synthesized argument from parser preprocessor
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::ExtractDay,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![
                    CallSpecArg::Named("hour".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|mut args| {
                    args.remove(0); // remove first default synthesized argument from parser preprocessor
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::ExtractHour,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![
                    CallSpecArg::Named("minute".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|mut args| {
                    args.remove(0); // remove first default synthesized argument from parser preprocessor
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::ExtractMinute,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![
                    CallSpecArg::Named("second".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|mut args| {
                    args.remove(0); // remove first default synthesized argument from parser preprocessor
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::ExtractSecond,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![
                    CallSpecArg::Named("timezone_hour".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|mut args| {
                    args.remove(0); // remove first default synthesized argument from parser preprocessor
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::ExtractTimezoneHour,
                        arguments: args,
                    })
                }),
            },
            CallSpec {
                input: vec![
                    CallSpecArg::Named("timezone_minute".into()),
                    CallSpecArg::Named("from".into()),
                ],
                output: Box::new(|mut args| {
                    args.remove(0); // remove first default synthesized argument from parser preprocessor
                    logical::ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::ExtractTimezoneMinute,
                        arguments: args,
                    })
                }),
            },
        ],
    }
}

pub(crate) static FN_SYM_TAB: Lazy<FnSymTab> = Lazy::new(function_call_def);

/// Function symbol table
#[derive(Debug)]
pub struct FnSymTab {
    calls: HashMap<UniCase<String>, CallDef>,
    synonyms: HashMap<UniCase<String>, UniCase<String>>,
}

impl FnSymTab {
    pub fn lookup(&self, fn_name: &str) -> Option<&CallDef> {
        self.synonyms
            .get(&fn_name.into())
            .and_then(|name| self.calls.get(name))
    }
}

pub fn function_call_def() -> FnSymTab {
    let mut calls: HashMap<UniCase<String>, CallDef> = HashMap::new();
    let mut synonyms: HashMap<UniCase<String>, UniCase<String>> = HashMap::new();

    for def in [
        function_call_def_char_len(),
        function_call_def_octet_len(),
        function_call_def_bit_len(),
        function_call_def_lower(),
        function_call_def_upper(),
        function_call_def_substring(),
        function_call_def_position(),
        function_call_def_overlay(),
        function_call_def_trim(),
        function_call_def_coalesce(),
        function_call_def_nullif(),
        function_call_def_exists(),
        function_call_def_abs(),
        function_call_def_mod(),
        function_call_def_cardinality(),
        function_call_def_extract(),
    ] {
        assert!(!def.names.is_empty());
        let primary = def.names[0];
        synonyms.insert(primary.into(), primary.into());
        for &name in &def.names[1..] {
            synonyms.insert(name.into(), primary.into());
        }

        calls.insert(primary.into(), def);
    }

    FnSymTab { calls, synonyms }
}
