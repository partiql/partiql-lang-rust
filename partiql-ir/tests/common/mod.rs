//! Hand-built plans shared by the tests.
#![allow(dead_code)]

use partiql_ir::*;
use partiql_types::{
    PartiqlNoIdShapeBuilder, PartiqlShape, Static, StructField, StructType, TypeParams,
};
use partiql_value::Value;

pub fn b() -> PartiqlNoIdShapeBuilder {
    PartiqlNoIdShapeBuilder::default()
}

pub fn s(ty: Static) -> PartiqlShape {
    b().new_static(ty)
}

pub fn dynamic() -> PartiqlShape {
    PartiqlShape::Dynamic
}

pub fn string() -> PartiqlShape {
    s(Static::String)
}

pub fn int() -> PartiqlShape {
    s(Static::Int32)
}

pub fn bool_() -> PartiqlShape {
    s(Static::Bool)
}

pub fn row(fields: &[(&str, PartiqlShape)]) -> PartiqlShape {
    b().new_row(fields.iter().map(|(n, t)| StructField::new(n, t.clone())))
}

pub fn bag(elem: PartiqlShape) -> PartiqlShape {
    b().new_bag_of(elem)
}

pub fn rel_type(fields: &[(&str, PartiqlShape)]) -> RelType {
    RelType::new(
        fields
            .iter()
            .map(|(n, t)| RelField::new(*n, t.clone()))
            .collect(),
        false,
    )
}

pub fn str_lit(v: &str) -> Rex {
    Rex::lit(string(), Value::from(v))
}

pub fn int_lit(v: i64) -> Rex {
    Rex::lit(int(), Value::from(v))
}

/// The shape of scribe's `default.T` subset used by the design's probe: a bag of
/// `ROW{a: BOOL, b: INTEGER, ts: TIMESTAMP(6)}` with an unspecified timestamp precision.
pub fn t_row() -> PartiqlShape {
    let ts = b().new_static_with_params(
        Static::Timestamp(6),
        TypeParams::default().with_unspecified_precision(true),
    );
    row(&[("a", bool_()), ("b", int()), ("ts", ts)])
}

/// `SELECT t.a, t.b FROM default.T AS t`, typed as partiql-lang-kotlin v1.6.1 plans it:
/// `RexSelect(RelProject(RelScan(RexTable)), RexVar)`, the projection a `RexStruct` of
/// `RexPathKey`s over `RexVar(0, 0)`.
pub fn select_from_t() -> Plan {
    let t = t_row();
    let out = row(&[("a", bool_()), ("b", int())]);
    let var_t = || Rex::var(t.clone(), 0, 0);
    let key = |k: &str, ty: PartiqlShape| Rex::path_key(ty, var_t(), str_lit(k));
    let scan = Rel::scan(
        rel_type(&[("t", t.clone())]),
        Rex::table(bag(t.clone()), QualifiedName::new("default", ["s"], "T")),
    );
    let projection = Rex::struct_(
        out.clone(),
        vec![
            StructEntry::new(str_lit("a"), key("a", bool_())),
            StructEntry::new(str_lit("b"), key("b", int())),
        ],
    );
    let project = Rel::project(rel_type(&[("_1", out.clone())]), scan, vec![projection]);
    Plan::query(Rex::select(bag(out.clone()), project, Rex::var(out, 0, 0)))
}

pub fn routine(name: &str, params: Vec<PartiqlShape>, returns: PartiqlShape) -> RoutineRef {
    RoutineRef::new(
        QualifiedName::new("$system", Vec::<String>::new(), name),
        format!("{name}__{}", params.len()),
        params,
        returns,
    )
}

/// A plan that uses every `Rex` and `Rel` node (the values are not meaningful).
pub fn every_node() -> Plan {
    let t = t_row();
    let t_rel = || rel_type(&[("t", t.clone())]);
    let var = || Rex::var(t.clone(), 0, 0);
    let scan = || {
        Rel::scan(
            t_rel(),
            Rex::table(bag(t.clone()), QualifiedName::new("default", ["s"], "T")),
        )
    };
    let sub = |rex: Rex| Rex::select(bag(dynamic()), scan(), rex);

    // FROM t, t.arr AS x  (correlate) JOIN UNPIVOT .. ON true
    let correlate = Rel::correlate(
        rel_type(&[("t", t.clone()), ("x", dynamic())]),
        scan(),
        Rel::iterate(
            rel_type(&[("x", dynamic())]),
            Rex::path_symbol(dynamic(), var(), "arr"),
        ),
        JoinType::Inner,
    );
    let join = Rel::join(
        rel_type(&[
            ("t", t.clone()),
            ("x", dynamic()),
            ("v", dynamic()),
            ("k", string()),
        ]),
        correlate,
        Rel::unpivot(
            rel_type(&[("v", dynamic()), ("k", string())]),
            Rex::table_ref(dynamic(), 0, 1),
        ),
        Rex::lit(bool_(), Value::from(true)),
        JoinType::Left,
    );
    let filter = Rel::filter(
        join.ty().clone(),
        join,
        Rex::subquery_test(bool_(), scan(), SubqueryTest::Exists),
    );
    let count = routine("count", vec![dynamic()], s(Static::Int64));
    let agg = Rel::aggregate(
        rel_type(&[("c", s(Static::Int64)), ("g", bool_())]),
        filter,
        vec![Measure::new(count, vec![var()], true)],
        vec![Rex::path_key(bool_(), var(), str_lit("a"))],
    );
    let distinct = Rel::distinct(agg.ty().clone(), agg);
    let exclude = Rel::exclude(
        distinct.ty().clone(),
        distinct,
        vec![Exclusion::new(
            var(),
            vec![
                ExclusionItem::struct_symbol(
                    "a",
                    vec![
                        ExclusionItem::coll_index(0, vec![]),
                        ExclusionItem::coll_wildcard(vec![]),
                    ],
                ),
                ExclusionItem::struct_key("B", vec![ExclusionItem::struct_wildcard(vec![])]),
            ],
        )],
    );
    let sort_key = || Collation::new(Rex::var(bool_(), 0, 1), Order::Desc, Nulls::Last);
    let sort = Rel::sort(exclude.ty().clone(), exclude, vec![sort_key()]);
    let limit = Rel::limit(sort.ty().clone(), sort, int_lit(10));
    let offset = Rel::offset(limit.ty().clone(), limit, int_lit(2));
    let rank = WindowFunction::new(
        WindowSignature::new("rank", vec![], s(Static::Int64), false),
        vec![],
    );
    let window = Rel::window(
        rel_type(&[
            ("c", s(Static::Int64)),
            ("g", bool_()),
            ("r", s(Static::Int64)),
        ]),
        Some("w".to_string()),
        offset,
        vec!["r".to_string()],
        vec![rank],
        vec![sort_key()],
        vec![Rex::var(bool_(), 0, 1)],
    );
    let set_ops = Rel::union(
        t_rel(),
        Rel::intersect(t_rel(), scan(), scan(), false),
        Rel::except(t_rel(), scan(), scan(), true),
        false,
    );
    let with = Rel::with(
        window.ty().clone(),
        window,
        vec![WithElement::new("w1", sub(var()))],
    );

    let plus = routine("plus", vec![int(), int()], int());
    let abs_int = routine("abs", vec![int()], int());
    let abs_str = routine("abs", vec![string()], string());
    let map_ty = b().new_map(string(), int()).expect("map");
    let projections = vec![
        Rex::array(b().new_array_of(int()), vec![int_lit(1), int_lit(2)]),
        Rex::bag(bag(int()), vec![int_lit(3)]),
        Rex::call(int(), plus, vec![int_lit(1), int_lit(2)]),
        Rex::case(
            int(),
            Some(int_lit(1)),
            vec![CaseBranch::new(int_lit(1), int_lit(2))],
            Some(int_lit(3)),
        ),
        Rex::case(
            dynamic(),
            None,
            vec![CaseBranch::new(
                Rex::lit(bool_(), Value::from(false)),
                int_lit(2),
            )],
            None,
        ),
        Rex::cast(s(Static::Int64), int_lit(1), s(Static::Int64)),
        Rex::coalesce(int(), vec![int_lit(1), int_lit(2)]),
        Rex::dispatch(
            dynamic(),
            "abs",
            vec![abs_int, abs_str],
            vec![Rex::var(dynamic(), 1, 0)],
        ),
        Rex::error(dynamic()),
        Rex::map(
            map_ty.clone(),
            string(),
            int(),
            vec![MapEntry::new(str_lit("k"), int_lit(1))],
        ),
        Rex::map_dynamic(dynamic(), vec![MapEntry::new(str_lit("k"), int_lit(1))]),
        Rex::null_if(int(), int_lit(1), int_lit(2)),
        Rex::path_index(dynamic(), Rex::var(dynamic(), 0, 1), int_lit(0)),
        Rex::pivot(
            s(Static::Struct(StructType::new_any())),
            set_ops,
            str_lit("k"),
            var(),
        ),
        Rex::spread(
            s(Static::Struct(StructType::new_any())),
            vec![var(), Rex::var(dynamic(), 0, 1)],
        ),
        Rex::struct_(
            row(&[("a", int())]),
            vec![StructEntry::new(str_lit("a"), int_lit(1))],
        ),
        Rex::subquery(
            int(),
            scan(),
            Rex::path_key(int(), var(), str_lit("b")),
            true,
        ),
        Rex::subquery_comp(
            bool_(),
            scan(),
            vec![int_lit(1)],
            Comparison::Ge,
            Quantifier::Any,
        ),
        Rex::subquery_in(bool_(), scan(), vec![int_lit(1)]),
    ];
    let out = rel_type(&[("_1", dynamic())]);
    let project = Rel::project(out, with, projections);
    Plan::query(Rex::select(
        bag(dynamic()),
        project,
        Rex::var(dynamic(), 0, 0),
    ))
}
