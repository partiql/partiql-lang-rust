use crate::peg::grammar::Rule;
use crate::prelude::ParserError;
use crate::result::ParserResult;
use itertools::Itertools;
use partiql_ast::experimental::ast;
use partiql_ast::experimental::ast::{
    FromClause, FromClauseKind, JoinKind, JoinSpec,
};
use pest::iterators::{Pair, Pairs};
use std::str::FromStr;

pub(crate) fn build_query(mut pairs: Pairs<Rule>) -> ParserResult<Box<ast::Expr>> {
    let pair = pairs.next().unwrap();

    let rule = pair.as_rule();

    match rule {
        Rule::query_full => build_query(pair.into_inner()),
        Rule::query => build_query(pair.into_inner()),
        Rule::sfw_query => build_sfw_query(pair.into_inner()),
        Rule::expr_query => build_expr_query(pair.into_inner()),
        _ => todo!("Unhandled rule [{:?}]", rule),
    }
}

pub(crate) fn build_sfw_query(pairs: Pairs<Rule>) -> ParserResult<Box<ast::Expr>> {
    let setq = None;
    let mut project = None;
    let mut from = None;
    let from_let = None;
    let mut where_clause = None;
    let mut group_by = None;
    let mut having = None;
    let mut order_by = None;
    let mut limit = None;
    let mut offset = None;

    for pair in pairs {
        let rule = pair.as_rule();

        match rule {
            Rule::select_clause => project = Some(build_select_clause(pair.into_inner())?),
            Rule::from_clause => from = Some(build_from_clause(pair.into_inner())?),
            Rule::limit_clause => limit = Some(build_limit_clause(pair.into_inner())?),
            Rule::offset_clause => offset = Some(build_offset_clause(pair.into_inner())?),
            Rule::order_by_clause => order_by = Some(build_order_by_clause(pair.into_inner())?),
            Rule::where_clause => where_clause = Some(build_where_clause(pair.into_inner())?),
            Rule::having_clause => having = Some(build_having_clause(pair.into_inner())?),
            Rule::group_by_clause => group_by = Some(build_group_by_clause(pair.into_inner())?),
            _ => todo!("Unhandled rule [{:?}]", rule),
        }
    }

    let project = project.expect("Select required");
    Ok(Box::new(ast::Expr {
        kind: ast::ExprKind::Select(ast::Select {
            setq,
            project,
            from,
            from_let,
            where_clause,
            group_by,
            having,
            order_by,
            limit,
            offset,
        }),
    }))
}

pub(crate) fn build_group_by_clause(pairs: Pairs<Rule>) -> ParserResult<Box<ast::GroupByExpr>> {
    let mut parts: Vec<_> = pairs.rev().collect();

    let mut next = parts.pop().unwrap();
    let strategy = if let Rule::group_all = next.as_rule() {
        next = parts.pop().unwrap();
        ast::GroupingStrategy {
            kind: ast::GroupingStrategyKind::GroupFull,
        }
    } else {
        ast::GroupingStrategy {
            kind: ast::GroupingStrategyKind::GroupPartial,
        }
    };

    let keys = next
        .into_inner()
        .map(|out_binding| {
            let mut parts = out_binding.into_inner();
            let expr = build_expr_query(parts.next().unwrap().into_inner());
            let as_alias = parts.next().map(|pair| ast::SymbolPrimitive {
                value: pair.as_str().trim_matches('"').to_string(),
            });
            expr.map(|expr| ast::GroupKey {
                expr: *expr,
                as_alias,
            })
        })
        .collect::<ParserResult<Vec<_>>>()?;
    let key_list = ast::GroupKeyList { keys };

    let group_as_alias = parts.pop().map(|pair| ast::SymbolPrimitive {
            value: pair.as_str().trim_matches('"').to_string(),
        });

    Ok(Box::new(ast::GroupByExpr {
        strategy,
        key_list,
        group_as_alias,
    }))
}

pub(crate) fn build_order_by_clause(mut pairs: Pairs<Rule>) -> ParserResult<Box<ast::OrderByExpr>> {
    let pair = pairs.next().unwrap();

    let rule = pair.as_rule();

    let order_expr = match rule {
        Rule::order_sort_preserve => ast::OrderByExpr { sort_specs: vec![] },
        Rule::order_sort_spec_list => ast::OrderByExpr {
            sort_specs: build_order_sort_specs(pair.into_inner())?,
        },
        _ => todo!("Unhandled rule [{:?}]", rule),
    };
    Ok(Box::new(order_expr))
}

pub(crate) fn build_order_sort_specs(pairs: Pairs<Rule>) -> ParserResult<Vec<ast::SortSpec>> {
    pairs
        .map(|pair| build_order_sort_spec(pair.into_inner()))
        .collect::<ParserResult<Vec<_>>>()
}

pub(crate) fn build_order_sort_spec(mut pairs: Pairs<Rule>) -> ParserResult<ast::SortSpec> {
    let expr = build_expr_query(pairs.next().unwrap().into_inner())?;
    let mut ordering_spec = None;
    let mut null_ordering_spec = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::order_by_spec => {
                let kind = if pair.as_str().to_lowercase() == "asc" {
                    ast::OrderingSpecKind::Asc
                } else {
                    ast::OrderingSpecKind::Desc
                };
                ordering_spec = Some(ast::OrderingSpec { kind })
            }
            Rule::order_by_null_spec => {
                let kind = if pair.as_str().to_lowercase().contains("first") {
                    ast::NullOrderingSpecKind::First
                } else {
                    ast::NullOrderingSpecKind::Last
                };
                null_ordering_spec = Some(ast::NullOrderingSpec { kind })
            }
            _ => todo!("Unhandled rule [{:?}]", pair.as_rule()),
        }
    }

    Ok(ast::SortSpec {
        expr,
        ordering_spec,
        null_ordering_spec,
    })
}

pub(crate) fn build_limit_clause(mut pairs: Pairs<Rule>) -> ParserResult<Box<ast::Expr>> {
    build_expr_query(pairs.next().unwrap().into_inner())
}

pub(crate) fn build_offset_clause(mut pairs: Pairs<Rule>) -> ParserResult<Box<ast::Expr>> {
    build_expr_query(pairs.next().unwrap().into_inner())
}

pub(crate) fn build_where_clause(pairs: Pairs<Rule>) -> ParserResult<Box<ast::Expr>> {
    build_expr_query(pairs)
}

pub(crate) fn build_having_clause(pairs: Pairs<Rule>) -> ParserResult<Box<ast::Expr>> {
    build_expr_query(pairs)
}

pub(crate) fn build_select_clause(mut pairs: Pairs<Rule>) -> ParserResult<ast::Projection> {
    let pair = pairs.next().unwrap();

    let rule = pair.as_rule();

    match rule {
        Rule::select_sql_clause => {
            let project_items = pair
                .into_inner()
                .map(|out_binding| match out_binding.as_rule() {
                    Rule::out_binding => {
                        let mut parts = out_binding.into_inner();
                        let expr = build_expr_query(parts.next().unwrap().into_inner());
                        let as_alias = parts.next().map(|pair| ast::SymbolPrimitive {
                            value: pair.as_str().trim_matches('"').to_string(),
                        });
                        expr.map(|expr| ast::ProjectItem {
                            kind: ast::ProjectItemKind::ProjectExpr(ast::ProjectExpr {
                                expr: *expr,
                                as_alias,
                            }),
                        })
                    }
                    _ => todo!("Unhandled rule [{:?}]", rule),
                })
                .collect::<ParserResult<Vec<_>>>()?;
            Ok(ast::Projection {
                kind: ast::ProjectionKind::ProjectList(ast::ProjectList { project_items }),
            })
        }
        _ => todo!("Unhandled rule [{:?}]", rule),
    }
}

pub(crate) fn build_from_clause(pairs: Pairs<Rule>) -> ParserResult<ast::FromClause> {
    let froms: Vec<ast::FromClause> = pairs
        .map(build_table_reference)
        .collect::<ParserResult<Vec<_>>>()?;

    let from = froms
        .into_iter()
        .reduce(|lfrom, rfrom| {
            let join = ast::Join {
                kind: ast::JoinKind::Cross,
                left: Box::new(lfrom),
                right: Box::new(rfrom),
                predicate: None,
            };
            ast::FromClause {
                kind: ast::FromClauseKind::Join(join),
            }
        })
        .unwrap(); // safe, because we know there's at least 1 input
    Ok(from)
}

fn build_table_reference(pair: Pair<Rule>) -> Result<FromClause, ParserError> {
    let rule = pair.as_rule();

    match rule {
        Rule::table_joined => build_table_joined(pair.into_inner()),
        Rule::table_unpivot => build_table_unpivot(pair.into_inner()),
        Rule::table_base_reference => build_table_base_reference(pair.into_inner()),
        _ => todo!("Unhandled rule [{:?}]", rule),
    }
}

fn build_table_unpivot(mut pairs: Pairs<Rule>) -> Result<FromClause, ParserError> {
    let expr = build_expr_query(pairs.next().unwrap().into_inner());
    let as_alias = pairs.next().map(|pair| ast::SymbolPrimitive {
        value: pair.as_str().trim_matches('"').to_string(),
    });
    let at_alias = pairs.next().map(|pair| ast::SymbolPrimitive {
        value: pair.as_str().trim_matches('"').to_string(),
    });
    expr.map(|expr| ast::FromClause {
        kind: FromClauseKind::FromLet(ast::FromLet {
            kind: ast::FromLetKind::Unpivot,
            expr,
            as_alias,
            at_alias,
            by_alias: None,
        }),
    })
}

fn build_table_base_reference(mut pairs: Pairs<Rule>) -> Result<FromClause, ParserError> {
    let expr = build_expr_query(pairs.next().unwrap().into_inner());
    let as_alias = pairs.next().map(|pair| ast::SymbolPrimitive {
        value: pair.as_str().trim_matches('"').to_string(),
    });
    expr.map(|expr| ast::FromClause {
        kind: FromClauseKind::FromLet(ast::FromLet {
            kind: ast::FromLetKind::Scan,
            expr,
            as_alias,
            at_alias: None,
            by_alias: None,
        }),
    })
}

fn build_table_joined(mut pairs: Pairs<Rule>) -> Result<FromClause, ParserError> {
    let pair = pairs.next().unwrap();
    let rule = pair.as_rule();

    match rule {
        Rule::table_joined => build_table_joined(pair.into_inner()),
        Rule::table_cross_join => build_table_cross_join(pair.into_inner()),
        Rule::table_natural_join => build_table_natural_join(pair.into_inner()),
        Rule::table_join_spec => build_table_join_spec(pair.into_inner()),
        _ => todo!("Unhandled rule [{:?}]", rule),
    }
}

fn build_join_spec(mut pairs: Pairs<Rule>) -> Result<JoinSpec, ParserError> {
    let pair = pairs.next().unwrap();
    let rule = pair.as_rule();

    match rule {
        Rule::search_condition => {
            todo!()
        }
        Rule::path_expr_list => {
            todo!()
        }
        _ => todo!("Unhandled rule [{:?}]", rule),
    }
}

fn build_table_join_spec(pairs: Pairs<Rule>) -> Result<FromClause, ParserError> {
    let mut parts: Vec<_> = pairs.collect();
    let spec = build_join_spec(parts.pop().unwrap().into_inner())?;
    let right = build_table_reference(parts.pop().unwrap())?;
    let kind = if parts.len() > 1 {
        let pair = parts.pop().unwrap();
        let rule = pair.as_rule();

        match rule {
            Rule::join_type => {
                let mut inner = pair.into_inner();
                if let Some(outer_type) = inner.next() {
                    let outer_str = outer_type.as_str().to_lowercase();
                    if outer_str == "left" {
                        JoinKind::Left
                    } else if outer_str == "right" {
                        JoinKind::Right
                    } else if outer_str == "full" {
                        JoinKind::Full
                    } else {
                        panic!("Unknown Join type [{:?}]", outer_type)
                    }
                } else {
                    JoinKind::Inner
                }
            }
            _ => todo!("Unhandled rule [{:?}]", rule),
        }
    } else {
        JoinKind::Inner
    };
    let left = build_table_base_reference(parts.pop().unwrap().into_inner())?;

    Ok(ast::FromClause {
        kind: FromClauseKind::Join(ast::Join {
            kind,
            left: Box::new(left),
            right: Box::new(right),
            predicate: Some(spec),
        }),
    })
}

fn build_table_natural_join(pairs: Pairs<Rule>) -> Result<FromClause, ParserError> {
    let mut parts: Vec<_> = pairs.collect();
    let right = build_table_reference(parts.pop().unwrap())?;
    let kind = if parts.len() > 1 {
        let pair = parts.pop().unwrap();
        let rule = pair.as_rule();

        match rule {
            Rule::join_type => {
                let mut inner = pair.into_inner();
                if let Some(outer_type) = inner.next() {
                    let outer_str = outer_type.as_str().to_lowercase();
                    if outer_str == "left" {
                        JoinKind::Left
                    } else if outer_str == "right" {
                        JoinKind::Right
                    } else if outer_str == "full" {
                        JoinKind::Full
                    } else {
                        panic!("Unknown Join type [{:?}]", outer_type)
                    }
                } else {
                    JoinKind::Inner
                }
            }
            _ => todo!("Unhandled rule [{:?}]", rule),
        }
    } else {
        JoinKind::Inner
    };
    let left = build_table_base_reference(parts.pop().unwrap().into_inner())?;

    Ok(ast::FromClause {
        kind: FromClauseKind::Join(ast::Join {
            kind,
            left: Box::new(left),
            right: Box::new(right),
            predicate: None,
        }),
    })
}

fn build_table_cross_join(mut pairs: Pairs<Rule>) -> Result<FromClause, ParserError> {
    let left = build_table_base_reference(pairs.next().unwrap().into_inner())?;
    let right = build_table_reference(pairs.next().unwrap())?;

    Ok(ast::FromClause {
        kind: FromClauseKind::Join(ast::Join {
            kind: ast::JoinKind::Cross,
            left: Box::new(left),
            right: Box::new(right),
            predicate: None,
        }),
    })
}

pub(crate) fn build_expr_query(pairs: Pairs<Rule>) -> ParserResult<Box<ast::Expr>> {
    let mut parts: Vec<_> = pairs
        .map(|pair| {
            let rule = pair.as_rule();

            match rule {
                Rule::expr_query => build_expr_query(pair.into_inner()),
                Rule::expr_term => build_expr_term(pair.into_inner()),
                Rule::op_bool => {
                    if pair.as_str().to_lowercase() == "or" {
                        Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Or(ast::Or { operands: vec![] }),
                        }))
                    } else if pair.as_str().to_lowercase() == "and" {
                        Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::And(ast::And { operands: vec![] }),
                        }))
                    } else {
                        panic!("Unexpected bool operator [{:?}]", pair.as_str());
                    }
                }
                Rule::op_infix => {
                    // TODO this doesn't handle precedence correctly
                    // TODO use Pest's precedence climber
                    let op_pair = pair.into_inner().next().unwrap();
                    match op_pair.as_rule() {
                        Rule::op_caret => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Exponentiate(ast::Exponentiate {
                                operands: vec![],
                            }),
                        })),
                        Rule::op_multiply => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Times(ast::Times { operands: vec![] }),
                        })),
                        Rule::op_divide => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Divide(ast::Divide { operands: vec![] }),
                        })),
                        Rule::op_modulus => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Modulo(ast::Modulo { operands: vec![] }),
                        })),
                        Rule::op_add => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Plus(ast::Plus { operands: vec![] }),
                        })),
                        Rule::op_subtract => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Minus(ast::Minus { operands: vec![] }),
                        })),

                        Rule::op_eq => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Eq(ast::Eq { operands: vec![] }),
                        })),
                        Rule::op_neq => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Ne(ast::Ne { operands: vec![] }),
                        })),
                        Rule::op_lt => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Lt(ast::Lt { operands: vec![] }),
                        })),
                        Rule::op_gt => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Gt(ast::Gt { operands: vec![] }),
                        })),
                        Rule::op_lteq => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Lte(ast::Lte { operands: vec![] }),
                        })),
                        Rule::op_gteq => Ok(Box::new(ast::Expr {
                            kind: ast::ExprKind::Gte(ast::Gte { operands: vec![] }),
                        })),

                        _ => panic!("Unexpected infix operator [{:?}]", op_pair.as_str()),
                    }
                }
                _ => todo!("Unhandled rule [{:?}]", rule),
            }
        })
        .collect::<ParserResult<Vec<_>>>()?;

    let len = parts.len();
    let res = if len == 1 {
        parts.pop().unwrap()
    } else if len == 2 {
        let (_unary, _expr) = (parts.pop().unwrap(), parts.pop().unwrap());
        todo!()
    } else if len == 3 {
        let (lhs, operator, rhs) = (
            parts.pop().unwrap(),
            parts.pop().unwrap(),
            parts.pop().unwrap(),
        );
        let new_operands = vec![lhs, rhs];
        let op = *operator;
        let new_op = if let ast::Expr { kind } = op {
            match kind {
                ast::ExprKind::And(ast::And { .. }) => ast::Expr {
                    kind: ast::ExprKind::And(ast::And {
                        operands: new_operands,
                    }),
                },
                ast::ExprKind::Or(ast::Or { .. }) => ast::Expr {
                    kind: ast::ExprKind::Or(ast::Or {
                        operands: new_operands,
                    }),
                },
                ast::ExprKind::Plus(ast::Plus { .. }) => ast::Expr {
                    kind: ast::ExprKind::Plus(ast::Plus {
                        operands: new_operands,
                    }),
                },
                // TODO all other infix operators
                ast::ExprKind::Eq(ast::Eq { .. }) => ast::Expr {
                    kind: ast::ExprKind::Eq(ast::Eq {
                        operands: new_operands,
                    }),
                },
                _ => todo!("Unhandled operator kind [{:?}]", kind),
            }
        } else {
            todo!("Unhandled operator [{:?}]", op)
        };
        Box::new(new_op)
    } else {
        todo!("Unexpected part of expr_query")
    };

    Ok(res)
}

pub(crate) fn build_expr_term(mut pairs: Pairs<Rule>) -> ParserResult<Box<ast::Expr>> {
    let pair = pairs.next().unwrap();

    let rule = pair.as_rule();

    match rule {
        Rule::query => build_query(pair.into_inner()),
        Rule::expr_function_call => build_expr_function_call(pair.into_inner()),
        Rule::literal => build_literal(pair.into_inner()),
        Rule::path_expr => build_path_expr(pair.into_inner()),
        Rule::expr_tuple => {
            let fields: Vec<ast::ExprPair> = pair
                .into_inner()
                .map(|pair| build_expr_query(pair.into_inner()))
                .tuples::<(_, _)>()
                .map(|(k, v)| {
                    if let Ok(first) = k {
                        if let Ok(second) = v {
                            Ok(ast::ExprPair { first, second })
                        } else {
                            Err(v.unwrap_err())
                        }
                    } else {
                        Err(k.unwrap_err())
                    }
                })
                .collect::<ParserResult<Vec<_>>>()?;

            Ok(Box::new(ast::Expr {
                kind: ast::ExprKind::Struct(ast::Struct { fields }),
            }))
        }
        Rule::expr_array => {
            let values = pair
                .into_inner()
                .map(|pair| build_expr_query(pair.into_inner()))
                .collect::<ParserResult<Vec<_>>>()?;
            Ok(Box::new(ast::Expr {
                kind: ast::ExprKind::List(ast::List { values }),
            }))
        }
        Rule::expr_bag => {
            let values = pair
                .into_inner()
                .map(|pair| build_expr_query(pair.into_inner()))
                .collect::<ParserResult<Vec<_>>>()?;
            Ok(Box::new(ast::Expr {
                kind: ast::ExprKind::Bag(ast::Bag { values }),
            }))
        }
        _ => todo!("Unhandled rule [{:?}]", rule),
    }
}

pub(crate) fn build_expr_function_call(mut pairs: Pairs<Rule>) -> ParserResult<Box<ast::Expr>> {
    let pair = pairs.next().unwrap();

    let func_name = ast::SymbolPrimitive {
        value: pair.as_str().trim_matches('"').to_string(),
    };

    let args = pairs
        .map(|pair| build_expr_query(pair.into_inner()))
        .collect::<ParserResult<Vec<_>>>()?;

    Ok(Box::new(ast::Expr {
        kind: ast::ExprKind::Call(ast::Call { func_name, args }),
    }))
}

pub(crate) fn build_path_expr(mut pairs: Pairs<Rule>) -> ParserResult<Box<ast::Expr>> {
    if let Some(pair) = pairs.next() {
        let rule = pair.as_rule();

        match rule {
            Rule::identifier => {
                let ident = ast::Expr {
                    kind: ast::ExprKind::VarRef(ast::VarRef {
                        name: ast::SymbolPrimitive {
                            value: pair.as_str().trim_matches('"').to_string(),
                        },
                        case: ast::CaseSensitivity {
                            kind: ast::CaseSensitivityKind::CaseInsensitive,
                        },
                        qualifier: ast::ScopeQualifier {
                            kind: ast::ScopeQualifierKind::Unqualified,
                        },
                    }),
                };
                Ok(Box::new(ast::Expr {
                    kind: ast::ExprKind::Path(ast::Path {
                        root: Box::new(ident),
                        steps: vec![],
                    }),
                }))
            }
            _ => todo!("Unhandled rule [{:?}]", rule),
        }
    } else {
        let ident = ast::Expr {
            kind: ast::ExprKind::VarRef(ast::VarRef {
                name: ast::SymbolPrimitive {
                    value: "*".to_string(),
                },
                case: ast::CaseSensitivity {
                    kind: ast::CaseSensitivityKind::CaseInsensitive,
                },
                qualifier: ast::ScopeQualifier {
                    kind: ast::ScopeQualifierKind::Unqualified,
                },
            }),
        };
        Ok(Box::new(ast::Expr {
            kind: ast::ExprKind::Path(ast::Path {
                root: Box::new(ident),
                steps: vec![],
            }),
        }))
    }
}

pub(crate) fn build_literal(mut pairs: Pairs<Rule>) -> ParserResult<Box<ast::Expr>> {
    fn build_string(lit: Pair<Rule>) -> ast::Expr {
        let litstr = lit.as_str();
        ast::Expr {
            kind: ast::ExprKind::Lit(ast::Lit {
                kind: ast::LitKind::CharStringLit(ast::CharStringLit {
                    value: litstr[1..litstr.len() - 1].to_string(),
                }),
            }),
        }
    }

    let lit = pairs.next().unwrap();
    let res = match lit.as_rule() {
        Rule::literal_absent => {
            if lit.as_str().to_lowercase() == "null" {
                ast::Expr {
                    kind: ast::ExprKind::Lit(ast::Lit {
                        kind: ast::LitKind::Null,
                    }),
                }
            } else {
                ast::Expr {
                    kind: ast::ExprKind::Lit(ast::Lit {
                        kind: ast::LitKind::Missing,
                    }),
                }
            }
        }
        Rule::literal_ion => todo!(),
        Rule::literal_string => build_string(lit),
        Rule::literal_bool => {
            let value = lit.as_str().to_lowercase() == "true";
            ast::Expr {
                kind: ast::ExprKind::Lit(ast::Lit {
                    kind: ast::LitKind::BoolLit(ast::BoolLit { value }),
                }),
            }
        }
        Rule::literal_real => {
            let value = if let Ok(dec) = rust_decimal::Decimal::from_str(lit.as_str()) {
                dec
            } else if let Ok(dec) = rust_decimal::Decimal::from_scientific(lit.as_str()) {
                dec
            } else {
                todo!()
            };
            ast::Expr {
                kind: ast::ExprKind::Lit(ast::Lit {
                    kind: ast::LitKind::NumericLit(ast::NumericLit {
                        kind: ast::NumericLitKind::DecimalLit(ast::DecimalLit { value }),
                    }),
                }),
            }
        }
        Rule::literal_int => {
            let value: i64 = lit.as_str().parse().unwrap();
            ast::Expr {
                kind: ast::ExprKind::Lit(ast::Lit {
                    kind: ast::LitKind::NumericLit(ast::NumericLit {
                        kind: ast::NumericLitKind::Int64Lit(ast::Int64Lit { value }),
                    }),
                }),
            }
        }
        Rule::literal_tuple => {
            let fields: Vec<ast::ExprPair> = lit
                .into_inner()
                .map(|e| match e.as_rule() {
                    Rule::literal_string => Ok(Box::new(build_string(e))),
                    _ => build_literal(e.into_inner()),
                })
                .tuples::<(_, _)>()
                .map(|(k, v)| {
                    if let Ok(first) = k {
                        if let Ok(second) = v {
                            Ok(ast::ExprPair { first, second })
                        } else {
                            Err(v.unwrap_err())
                        }
                    } else {
                        Err(k.unwrap_err())
                    }
                })
                .collect::<ParserResult<Vec<_>>>()?;

            ast::Expr {
                kind: ast::ExprKind::Struct(ast::Struct { fields }),
            }
        }
        Rule::literal_array => {
            let values = lit
                .into_inner()
                .map(|pair| build_literal(pair.into_inner()))
                .collect::<ParserResult<Vec<_>>>()?;
            ast::Expr {
                kind: ast::ExprKind::List(ast::List { values }),
            }
        }
        Rule::literal_bag => {
            let values = lit
                .into_inner()
                .map(|pair| build_literal(pair.into_inner()))
                .collect::<ParserResult<Vec<_>>>()?;
            ast::Expr {
                kind: ast::ExprKind::Bag(ast::Bag { values }),
            }
        }
        _ => panic!("Unexpected rule [{:?}]", lit.as_rule()),
    };

    Ok(Box::new(res))
}
