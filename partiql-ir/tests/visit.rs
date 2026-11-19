//! The visitor reaches every node kind; the test plan uses every one.
mod common;

use common::*;
use partiql_ir::{tree, TreeNode};
use std::collections::BTreeSet;

fn kinds<'a>(n: &TreeNode<'a>, out: &mut BTreeSet<&'static str>) {
    out.insert(n.kind);
    n.children.iter().for_each(|(_, c)| kinds(c, out));
}

/// partiql-lang-kotlin v1.6.1's public `org.partiql.plan.rel` and `.rex` classes.
const PLK_NODES: [&str; 44] = [
    "RelAggregate",
    "RelCorrelate",
    "RelDistinct",
    "RelExcept",
    "RelExclude",
    "RelFilter",
    "RelIntersect",
    "RelIterate",
    "RelJoin",
    "RelLimit",
    "RelOffset",
    "RelProject",
    "RelScan",
    "RelSort",
    "RelUnion",
    "RelUnpivot",
    "RelWindow",
    "RelWith",
    "RexArray",
    "RexBag",
    "RexCall",
    "RexCase",
    "RexCast",
    "RexCoalesce",
    "RexDispatch",
    "RexError",
    "RexLit",
    "RexMap",
    "RexMapDynamic",
    "RexNullIf",
    "RexPathIndex",
    "RexPathKey",
    "RexPathSymbol",
    "RexPivot",
    "RexSelect",
    "RexSpread",
    "RexStruct",
    "RexSubquery",
    "RexSubqueryComp",
    "RexSubqueryIn",
    "RexSubqueryTest",
    "RexTable",
    "RexTableRef",
    "RexVar",
];

#[test]
fn every_node_plan_covers_the_plk_node_set() {
    let plan = every_node();
    let mut seen = BTreeSet::new();
    kinds(&tree(&plan), &mut seen);
    let missing: Vec<_> = PLK_NODES.iter().filter(|k| !seen.contains(*k)).collect();
    assert!(missing.is_empty(), "missing {missing:?}");
    let extra: Vec<_> = seen
        .iter()
        .filter(|k| (k.starts_with("Rex") || k.starts_with("Rel")) && !k.contains('.'))
        .filter(|k| !PLK_NODES.contains(k))
        .collect();
    assert!(extra.is_empty(), "not a PLK node {extra:?}");
}
