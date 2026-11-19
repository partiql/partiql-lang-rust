//! Tests of `partiql_catalog::typed`, ported from partiql-lang-kotlin v1.6.1 where it has them
//! (`CatalogsTest`, `SessionPathTest`, `RoutineBindingTest`) and otherwise derived from the
//! behavior of its `Identifier`, `Env.resolveTable` and `StandardCatalog`.

use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, SharedCatalog, TypeEnvEntry};
use partiql_catalog::typed::{
    equals_ignore_case, resolve_table, AggOverload, CatalogNameAmbiguous, Catalogs,
    FixedAggOverload, FixedFnOverload, FnOverload, IdentPart, Identifier, LegacyCatalogAdapter,
    Name, Namespace, Path, ResolveError, RoutineBinding, RoutineBindingError, RoutineSignature,
    Session, SessionError, SystemCatalog, Table, TypedCatalog, SYSTEM_CATALOG_NAME,
};
use partiql_types::{type_bag, type_int, type_string, PartiqlNoIdShapeBuilder, PartiqlShape};
use std::collections::BTreeMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------------------------
// Identifier matching
// ---------------------------------------------------------------------------------------------

#[test]
fn regular_ascii_matches_ignoring_case() {
    let part = IdentPart::regular("orders");
    assert!(part.matches("orders"));
    assert!(part.matches("ORDERS"));
    assert!(part.matches("Orders"));
    assert!(!part.matches("order"));
    assert!(!part.matches("orders_"));
    assert!(IdentPart::regular("ORDERS").matches("orders"));
}

#[test]
fn delimited_ascii_matches_exactly() {
    let part = IdentPart::delimited("orders");
    assert!(part.matches("orders"));
    assert!(!part.matches("ORDERS"));
    assert!(!part.matches("Orders"));
    assert!(IdentPart::delimited("Orders").matches("Orders"));
}

#[test]
fn regular_non_ascii_matches_ignoring_case() {
    assert!(IdentPart::regular("caf\u{e9}").matches("CAF\u{c9}")); // café / CAFÉ
    assert!(IdentPart::regular("\u{c4}pfel").matches("\u{e4}PFEL")); // Äpfel / äPFEL
    assert!(
        IdentPart::regular("\u{3c3}\u{3bf}\u{3c6}\u{3af}\u{3b1}").matches(
        "\u{3a3}\u{39f}\u{3a6}\u{38a}\u{391}" // σοφία / ΣΟΦΊΑ
    )
    );
    assert!(IdentPart::regular("\u{43c}\u{438}\u{440}").matches("\u{41c}\u{418}\u{420}")); // мир / МИР
    assert!(!IdentPart::regular("caf\u{e9}").matches("CAFE"));
}

#[test]
fn delimited_non_ascii_matches_exactly() {
    let part = IdentPart::delimited("caf\u{e9}");
    assert!(part.matches("caf\u{e9}"));
    assert!(!part.matches("CAF\u{c9}"));
    assert!(!part.matches("Caf\u{e9}"));
    // No normalization: a precomposed é is not e + COMBINING ACUTE ACCENT.
    assert!(!part.matches("cafe\u{301}"));
}

/// The regular comparison is Java's `String.equalsIgnoreCase` (partiql-lang-kotlin
/// `Identifier.Simple.matches`), which maps one character to one character and does no full
/// case folding.
#[test]
fn regular_matching_follows_java_equals_ignore_case() {
    // ß uppercases to "SS" only under full case mapping.
    assert!(!equals_ignore_case("stra\u{df}e", "STRASSE"));
    assert!(!equals_ignore_case("\u{df}", "ss"));
    // ß and capital sharp s ẞ share a lowercase.
    assert!(equals_ignore_case("\u{df}", "\u{1e9e}"));
    // Dotless ı uppercases to I, so it matches both I and i.
    assert!(equals_ignore_case("\u{131}", "I"));
    assert!(equals_ignore_case("\u{131}", "i"));
    // İ (I with dot above) lowercases to i by its single-character mapping.
    assert!(equals_ignore_case("\u{130}", "i"));
    // Final sigma ς and σ both uppercase to Σ.
    assert!(equals_ignore_case("\u{3c2}", "\u{3c3}"));
    // The Kelvin sign K lowercases to k.
    assert!(equals_ignore_case("\u{212a}", "k"));
    // Lengths must agree.
    assert!(!equals_ignore_case("ab", "a"));
    assert!(equals_ignore_case("", ""));
}

#[test]
fn identifier_matches_only_unqualified() {
    let id = Identifier::regular("orders");
    assert!(id.matches("ORDERS", true));
    assert!(!id.matches("ORDERS", false));
    assert!(id.matches("orders", false));
    assert!(!Identifier::delimited("orders").matches("ORDERS", true));
    let qualified = Identifier::regular_parts(["shop", "orders"]).unwrap();
    assert!(!qualified.matches("orders", true));
    assert!(!qualified.matches("orders", false));
}

#[test]
fn identifier_parts_and_display() {
    let id = Identifier::of([
        IdentPart::regular("shop"),
        IdentPart::delimited("Orders"),
        IdentPart::regular("total"),
    ])
    .unwrap();
    assert_eq!(id.len(), 3);
    assert!(id.has_qualifier());
    assert_eq!(id.identifier().text(), "total");
    assert_eq!(id.qualifier().len(), 2);
    assert_eq!(id.to_string(), "shop.\"Orders\".total");
    assert_eq!(
        id.append(&Identifier::regular("x")).to_string(),
        "shop.\"Orders\".total.x"
    );
    assert!(Identifier::of(Vec::<IdentPart>::new()).is_none());
    assert!(Identifier::regular_parts(Vec::<String>::new()).is_none());
}

/// partiql-lang-kotlin's `Identifier.Simple.equals` compares the text only.
#[test]
fn identifier_equality_ignores_regular_flag() {
    assert_eq!(IdentPart::regular("a"), IdentPart::delimited("a"));
    assert_ne!(IdentPart::regular("a"), IdentPart::regular("A"));
    assert_eq!(Identifier::regular("a"), Identifier::delimited("a"));
    let mut set = std::collections::HashSet::new();
    set.insert(Identifier::regular("a"));
    assert!(set.contains(&Identifier::delimited("a")));
}

#[test]
fn name_namespace_and_path_display() {
    let name = Name::of(["shop", "Orders"]).unwrap();
    assert_eq!(name.name(), "Orders");
    assert_eq!(name.namespace(), &Namespace::of(["shop"]));
    assert!(name.has_namespace());
    assert_eq!(name.len(), 2);
    assert_eq!(name.to_string(), "\"shop\".\"Orders\"");
    assert_eq!(Name::of(["t"]).unwrap().to_string(), "\"t\"");
    assert!(Name::of(Vec::<String>::new()).is_none());
    assert_ne!(Name::of(["t"]), Name::of(["T"]));

    assert_eq!(Namespace::empty().to_string(), "");
    assert!(Namespace::empty().as_identifier().is_none());
    assert_eq!(Namespace::of(["a", "b"]).to_string(), "\"a\".\"b\"");

    let path = Path::of([Namespace::of(["a"]), Namespace::of(["b", "c"])]);
    assert_eq!(path.to_string(), "PATH: (\"a\", \"b\".\"c\")");
    assert_eq!(
        path.promote(&Namespace::of(["b", "c"])).unwrap(),
        Path::of([Namespace::of(["b", "c"]), Namespace::of(["a"])])
    );
    assert!(path.promote(&Namespace::of(["z"])).is_err());
}

// ---------------------------------------------------------------------------------------------
// A namespaced test catalog (partiql-lang-kotlin planner `TestCatalog`)
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Default)]
struct Tree {
    name: String,
    table: Option<Table>,
    children: Vec<Tree>,
}

impl Tree {
    /// A regular part matches a child ignoring case (the first in order); a delimited one exactly.
    fn get(&self, part: &IdentPart) -> Option<&Tree> {
        if part.is_regular() {
            if let Some(child) = self.children.iter().find(|c| part.matches(&c.name)) {
                return Some(child);
            }
        }
        self.children.iter().find(|c| c.name == part.text())
    }

    fn get_or_put(&mut self, name: &str) -> &mut Tree {
        let i = match self.children.iter().position(|c| c.name == name) {
            Some(i) => i,
            None => {
                self.children.push(Tree {
                    name: name.to_string(),
                    ..Tree::default()
                });
                self.children.len() - 1
            }
        };
        &mut self.children[i]
    }
}

#[derive(Debug)]
struct TestCatalog {
    name: String,
    root: Tree,
}

impl TestCatalog {
    fn shared(name: &str, tables: &[&[&str]]) -> Arc<dyn TypedCatalog> {
        let mut root = Tree::default();
        for parts in tables {
            let mut curr = &mut root;
            for part in *parts {
                curr = curr.get_or_put(part);
            }
            let name = Name::of(parts.iter().copied()).unwrap();
            curr.table = Some(Table::new(name, schema()));
        }
        Arc::new(TestCatalog {
            name: name.to_string(),
            root,
        })
    }
}

impl TypedCatalog for TestCatalog {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_table(&self, _session: &Session, name: &Name) -> Option<Table> {
        let mut curr = &self.root;
        for part in name.parts() {
            match curr.get(&IdentPart::delimited(part)) {
                Some(next) => curr = next,
                None => break,
            }
        }
        curr.table.clone()
    }

    fn resolve_table(&self, _session: &Session, identifier: &Identifier) -> Option<Name> {
        let mut curr = &self.root;
        for part in identifier.parts() {
            match curr.get(part) {
                Some(next) => curr = next,
                None => break,
            }
        }
        curr.table.as_ref().map(|t| t.name().clone())
    }
}

fn schema() -> PartiqlShape {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let row = bld.new_struct_of_dyn();
    type_bag!(bld, row)
}

fn session(current: &str, namespace: &[&str], catalogs: Vec<Arc<dyn TypedCatalog>>) -> Session {
    Session::builder()
        .catalog(current)
        .namespace(Namespace::of(namespace.iter().copied()))
        .catalogs(catalogs)
        .build()
        .unwrap()
}

fn resolved(s: &Session, id: &Identifier) -> Option<(String, String, Vec<String>)> {
    resolve_table(s, id).unwrap().map(|r| {
        (
            r.catalog().to_string(),
            r.table().name().to_string(),
            r.path().iter().map(ToString::to_string).collect(),
        )
    })
}

fn reg(parts: &[&str]) -> Identifier {
    Identifier::regular_parts(parts.iter().copied()).unwrap()
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(ToString::to_string).collect()
}

// ---------------------------------------------------------------------------------------------
// Table resolution order (partiql-lang-kotlin `Env.resolveTable`)
// ---------------------------------------------------------------------------------------------

#[test]
fn step1_current_catalog_unqualified() {
    let s = session(
        "main",
        &[],
        vec![TestCatalog::shared("main", &[&["orders"]])],
    );
    assert_eq!(
        resolved(&s, &reg(&["ORDERS"])),
        Some(("main".into(), "\"orders\"".into(), vec![]))
    );
    assert_eq!(resolved(&s, &Identifier::delimited("ORDERS")), None);
    assert_eq!(resolved(&s, &reg(&["missing"])), None);
}

#[test]
fn step1_prepends_session_namespace() {
    let s = session(
        "main",
        &["shop"],
        vec![TestCatalog::shared("main", &[&["shop", "orders"]])],
    );
    assert_eq!(
        resolved(&s, &reg(&["orders"])),
        Some(("main".into(), "\"shop\".\"orders\"".into(), vec![]))
    );
}

/// The session namespace is prepended as delimited parts, so it matches exactly.
#[test]
fn step1_session_namespace_is_delimited() {
    let s = session(
        "main",
        &["SHOP"],
        vec![TestCatalog::shared("main", &[&["shop", "orders"]])],
    );
    assert_eq!(resolved(&s, &reg(&["orders"])), None);
}

#[test]
fn step1_leftover_parts_are_a_path() {
    let s = session(
        "main",
        &[],
        vec![TestCatalog::shared("main", &[&["orders"]])],
    );
    let id = Identifier::of([
        IdentPart::regular("orders"),
        IdentPart::delimited("Total"),
        IdentPart::regular("amount"),
    ])
    .unwrap();
    assert_eq!(
        resolved(&s, &id),
        Some((
            "main".into(),
            "\"orders\"".into(),
            strs(&["\"Total\"", "amount"])
        ))
    );
}

#[test]
fn step1_leftover_counts_namespace_parts() {
    let s = session(
        "main",
        &["shop"],
        vec![TestCatalog::shared("main", &[&["shop", "orders"]])],
    );
    assert_eq!(
        resolved(&s, &reg(&["orders", "total"])),
        Some((
            "main".into(),
            "\"shop\".\"orders\"".into(),
            strs(&["total"])
        ))
    );
}

#[test]
fn step2_schema_qualified_when_namespace_prefix_fails() {
    let s = session(
        "main",
        &["other"],
        vec![TestCatalog::shared("main", &[&["shop", "orders"]])],
    );
    assert_eq!(
        resolved(&s, &reg(&["shop", "orders"])),
        Some(("main".into(), "\"shop\".\"orders\"".into(), vec![]))
    );
    // Unqualified, there is no step 2.
    assert_eq!(resolved(&s, &reg(&["orders"])), None);
}

#[test]
fn step1_wins_over_step2() {
    let s = session(
        "main",
        &["shop"],
        vec![TestCatalog::shared(
            "main",
            &[&["shop", "shop", "orders"], &["shop", "orders"]],
        )],
    );
    // 1 finds shop.shop.orders; 2 would find shop.orders.
    assert_eq!(
        resolved(&s, &reg(&["shop", "orders"])),
        Some(("main".into(), "\"shop\".\"shop\".\"orders\"".into(), vec![]))
    );
}

#[test]
fn step3_catalog_qualified() {
    let s = session(
        "main",
        &[],
        vec![
            TestCatalog::shared("main", &[]),
            TestCatalog::shared("Archive", &[&["orders"]]),
        ],
    );
    // A regular head matches the catalog name ignoring case.
    assert_eq!(
        resolved(&s, &reg(&["archive", "orders"])),
        Some(("Archive".into(), "\"orders\"".into(), vec![]))
    );
    // A delimited head matches exactly.
    let exact = Identifier::of([
        IdentPart::delimited("Archive"),
        IdentPart::regular("orders"),
    ]);
    assert_eq!(
        resolved(&s, &exact.unwrap()),
        Some(("Archive".into(), "\"orders\"".into(), vec![]))
    );
    let wrong = Identifier::of([
        IdentPart::delimited("archive"),
        IdentPart::regular("orders"),
    ]);
    assert_eq!(resolved(&s, &wrong.unwrap()), None);
    // The session namespace is not prepended in step 3.
    let s = session(
        "main",
        &["shop"],
        vec![
            TestCatalog::shared("main", &[]),
            TestCatalog::shared("archive", &[&["orders"]]),
        ],
    );
    assert_eq!(
        resolved(&s, &reg(&["archive", "orders", "total"])),
        Some(("archive".into(), "\"orders\"".into(), strs(&["total"])))
    );
}

#[test]
fn step3_non_ascii_catalog_name() {
    let s = session(
        "main",
        &[],
        vec![
            TestCatalog::shared("main", &[]),
            TestCatalog::shared("caf\u{e9}", &[&["orders"]]),
        ],
    );
    assert_eq!(
        resolved(&s, &reg(&["CAF\u{c9}", "orders"])),
        Some(("caf\u{e9}".into(), "\"orders\"".into(), vec![]))
    );
}

#[test]
fn step2_wins_over_step3() {
    let s = session(
        "main",
        &[],
        vec![
            TestCatalog::shared("main", &[&["archive", "orders"]]),
            TestCatalog::shared("archive", &[&["orders"]]),
        ],
    );
    assert_eq!(
        resolved(&s, &reg(&["archive", "orders"])),
        Some(("main".into(), "\"archive\".\"orders\"".into(), vec![]))
    );
}

#[test]
fn step1_wins_over_step3() {
    // `archive.orders` is the table `archive` with the path `orders` in the current catalog.
    let s = session(
        "main",
        &[],
        vec![
            TestCatalog::shared("main", &[&["archive"]]),
            TestCatalog::shared("archive", &[&["orders"]]),
        ],
    );
    assert_eq!(
        resolved(&s, &reg(&["archive", "orders"])),
        Some(("main".into(), "\"archive\"".into(), strs(&["orders"])))
    );
}

#[test]
fn step3_missing_catalog_is_not_found() {
    let s = session("main", &[], vec![TestCatalog::shared("main", &[])]);
    assert_eq!(resolved(&s, &reg(&["nowhere", "orders"])), None);
}

#[test]
fn step3_ambiguous_catalog_name_is_an_error() {
    let s = session(
        "main",
        &[],
        vec![
            TestCatalog::shared("main", &[]),
            TestCatalog::shared("archive", &[&["orders"]]),
            TestCatalog::shared("ARCHIVE", &[&["orders"]]),
        ],
    );
    let err = resolve_table(&s, &reg(&["Archive", "orders"])).unwrap_err();
    assert_eq!(
        err,
        ResolveError::CatalogNameAmbiguous(CatalogNameAmbiguous::new(strs(&[
            "ARCHIVE", "archive"
        ])))
    );
    // A delimited head is exact, so not ambiguous.
    let id = Identifier::of([
        IdentPart::delimited("archive"),
        IdentPart::regular("orders"),
    ]);
    assert_eq!(
        resolved(&s, &id.unwrap()),
        Some(("archive".into(), "\"orders\"".into(), vec![]))
    );
}

#[test]
fn resolved_name_without_table_falls_through() {
    // A catalog that resolves a name it has no table for (step 1), so step 2 is tried.
    #[derive(Debug)]
    struct Dangling;
    impl TypedCatalog for Dangling {
        fn name(&self) -> &str {
            "main"
        }
        fn get_table(&self, _: &Session, name: &Name) -> Option<Table> {
            (name == &Name::of(["shop", "orders"]).unwrap()).then(|| Table::dynamic(name.clone()))
        }
        fn resolve_table(&self, _: &Session, id: &Identifier) -> Option<Name> {
            Name::of(id.parts().map(|p| p.text().to_string()))
        }
    }
    let s = session("main", &["other"], vec![Arc::new(Dangling)]);
    let r = resolve_table(&s, &reg(&["shop", "orders"]))
        .unwrap()
        .unwrap();
    assert_eq!(r.table().name(), &Name::of(["shop", "orders"]).unwrap());
    assert!(r.table().schema().is_dynamic());
}

#[test]
fn missing_default_catalog_is_an_error() {
    let s = session(
        "nope",
        &[],
        vec![TestCatalog::shared("main", &[&["orders"]])],
    );
    assert_eq!(
        resolve_table(&s, &reg(&["orders"])).unwrap_err(),
        ResolveError::MissingDefaultCatalog("nope".into())
    );
}

#[test]
fn system_catalog_has_no_tables() {
    let s = Session::empty();
    assert_eq!(s.catalog(), SYSTEM_CATALOG_NAME);
    assert_eq!(resolved(&s, &reg(&["orders"])), None);
    assert!(SystemCatalog::default().as_routine_catalog().is_some());
}

// ---------------------------------------------------------------------------------------------
// Catalogs (partiql-lang-kotlin `CatalogsTest`)
// ---------------------------------------------------------------------------------------------

fn names(catalogs: &Catalogs) -> Vec<String> {
    catalogs.iter().map(|c| c.name().to_string()).collect()
}

#[test]
fn case_insensitive_lookup_returns_zero_or_one_match() {
    let catalogs = Catalogs::of([TestCatalog::shared("Demo", &[])]);
    let found = catalogs.get_catalog("DEMO", true).unwrap().unwrap();
    assert_eq!(found.name(), "Demo");
    assert!(catalogs.get_catalog("missing", true).unwrap().is_none());
}

#[test]
fn case_insensitive_lookup_reports_all_matches_in_exact_name_order() {
    let catalogs = Catalogs::of([
        TestCatalog::shared("demo", &[]),
        TestCatalog::shared("Demo", &[]),
        TestCatalog::shared("DEMO", &[]),
    ]);
    let err = catalogs.get_catalog("dEmO", true).unwrap_err();
    assert_eq!(err.catalog_names(), strs(&["DEMO", "Demo", "demo"]));
    assert_eq!(
        err.to_string(),
        "Catalog name is ambiguous; matched DEMO, Demo, demo."
    );
}

#[test]
fn exact_lookup_keeps_case_distinct_catalogs_separate() {
    let catalogs = Catalogs::of([
        TestCatalog::shared("demo", &[]),
        TestCatalog::shared("DEMO", &[]),
    ]);
    assert_eq!(
        catalogs.get_catalog("demo", false).unwrap().unwrap().name(),
        "demo"
    );
    assert_eq!(
        catalogs.get_catalog("DEMO", false).unwrap().unwrap().name(),
        "DEMO"
    );
    assert!(catalogs.get_catalog("Demo", false).unwrap().is_none());
}

#[test]
fn catalogs_of_replaces_same_name_in_place() {
    let catalogs = Catalogs::of([
        TestCatalog::shared("a", &[]),
        TestCatalog::shared("b", &[]),
        TestCatalog::shared("a", &[&["t"]]),
    ]);
    assert_eq!(names(&catalogs), strs(&["a", "b"]));
    let a = catalogs.get_catalog("a", false).unwrap().unwrap();
    let s = Session::empty();
    assert!(a.resolve_table(&s, &reg(&["t"])).is_some());
}

// ---------------------------------------------------------------------------------------------
// Session (partiql-lang-kotlin `Session.Builder`, `SessionPathTest`)
// ---------------------------------------------------------------------------------------------

fn ns(levels: &[&str]) -> Namespace {
    Namespace::of(levels.iter().copied())
}

fn example() -> Vec<Arc<dyn TypedCatalog>> {
    vec![TestCatalog::shared("example", &[])]
}

#[test]
fn session_defaults() {
    let s = Session::builder()
        .catalog("example")
        .catalogs(example())
        .build()
        .unwrap();
    assert_eq!(s.identity(), "unknown");
    assert_eq!(s.namespace(), &Namespace::empty());
    assert_eq!(s.properties(), &BTreeMap::new());
    assert_eq!(names(s.catalogs()), strs(&["example", SYSTEM_CATALOG_NAME]));
    let s = Session::builder()
        .catalog("example")
        .identity("tester")
        .property("k", "v")
        .build()
        .unwrap();
    assert_eq!(s.identity(), "tester");
    assert_eq!(s.properties().get("k").map(String::as_str), Some("v"));
}

#[test]
fn session_requires_a_catalog() {
    assert_eq!(
        Session::builder().catalogs(example()).build().unwrap_err(),
        SessionError::MissingCatalog
    );
}

#[test]
fn session_catalog_names_must_be_unique() {
    let err = Session::builder()
        .catalog("example")
        .catalogs(example())
        .catalogs(example())
        .build()
        .unwrap_err();
    assert_eq!(err, SessionError::DuplicateCatalogName("example".into()));
    assert_eq!(err.to_string(), "Catalog names must be unique: example");
}

#[test]
fn omitted_path_is_current_catalog_namespace_then_system() {
    let s = Session::builder()
        .catalogs(example())
        .catalog("example")
        .namespace(ns(&["tables"]))
        .build()
        .unwrap();
    assert_eq!(
        s.path().namespaces(),
        &[ns(&["example", "tables"]), ns(&[SYSTEM_CATALOG_NAME])]
    );
}

#[test]
fn explicit_path_is_independent_of_namespace_and_appends_missing_system() {
    let s = Session::builder()
        .catalogs(example())
        .catalog("example")
        .namespace(ns(&["tables"]))
        .path(Path::of([
            ns(&["example", "collection"]),
            ns(&["example", "math"]),
        ]))
        .build()
        .unwrap();
    assert_eq!(s.namespace(), &ns(&["tables"]));
    assert_eq!(
        s.path().namespaces(),
        &[
            ns(&["example", "collection"]),
            ns(&["example", "math"]),
            ns(&[SYSTEM_CATALOG_NAME])
        ]
    );
}

#[test]
fn explicit_system_keeps_any_supplied_position() {
    let sys = ns(&[SYSTEM_CATALOG_NAME]);
    let c = ns(&["example", "collection"]);
    let m = ns(&["example", "math"]);
    for entries in [
        vec![sys.clone(), c.clone(), m.clone()],
        vec![c.clone(), sys.clone(), m.clone()],
        vec![c.clone(), m.clone(), sys.clone()],
    ] {
        let s = Session::builder()
            .catalogs(example())
            .catalog("example")
            .path(Path::of(entries.clone()))
            .build()
            .unwrap();
        assert_eq!(s.path().namespaces(), entries.as_slice());
    }
}

#[test]
fn supplied_path_duplicates_are_preserved() {
    let entries = vec![
        ns(&["example", "collection"]),
        ns(&[SYSTEM_CATALOG_NAME]),
        ns(&["example", "collection"]),
    ];
    let s = Session::builder()
        .catalogs(example())
        .catalog("example")
        .path(Path::of(entries.clone()))
        .build()
        .unwrap();
    assert_eq!(s.path().namespaces(), entries.as_slice());
}

#[test]
fn explicit_empty_path_contains_only_system() {
    let s = Session::builder()
        .catalogs(example())
        .catalog("example")
        .path(Path::of([]))
        .build()
        .unwrap();
    assert_eq!(s.path().namespaces(), &[ns(&[SYSTEM_CATALOG_NAME])]);
}

#[test]
fn custom_system_is_independent_of_builder_order_and_appended_when_missing() {
    let path = Path::of([ns(&["builtins"]), ns(&["example", "math"])]);
    let system = || TestCatalog::shared("builtins", &[]);
    let a = Session::builder()
        .catalogs(example())
        .catalog("example")
        .path(path.clone())
        .system(system())
        .build()
        .unwrap();
    let b = Session::builder()
        .catalogs(example())
        .catalog("example")
        .system(system())
        .path(path.clone())
        .build()
        .unwrap();
    assert_eq!(a.path(), path);
    assert_eq!(b.path(), path);
    assert_eq!(names(a.catalogs()), strs(&["example", "builtins"]));

    let s = Session::builder()
        .catalogs(example())
        .catalog("example")
        .path(Path::of([ns(&["example", "math"])]))
        .system(system())
        .build()
        .unwrap();
    assert_eq!(
        s.path().namespaces(),
        &[ns(&["example", "math"]), ns(&["builtins"])]
    );
}

// ---------------------------------------------------------------------------------------------
// Routine signatures (partiql-lang-kotlin `RoutineBindingTest`, `FnOverload.Builder`)
// ---------------------------------------------------------------------------------------------

fn pow() -> Arc<dyn FnOverload> {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let int = type_int!(bld);
    Arc::new(FixedFnOverload::new(RoutineSignature::with_types(
        "pow",
        [int.clone(), int.clone()],
        int,
    )))
}

#[test]
fn routine_binding_keeps_name_and_overloads() {
    let binding = RoutineBinding::new(Name::of(["math", "pow"]).unwrap(), vec![pow()]).unwrap();
    assert_eq!(
        binding.canonical_name(),
        &Name::of(["math", "pow"]).unwrap()
    );
    assert_eq!(binding.overloads().len(), 1);
}

#[test]
fn routine_binding_rejects_empty_overloads_and_name_parts() {
    let err = RoutineBinding::<dyn FnOverload>::new(Name::of(["pow"]).unwrap(), vec![]);
    assert_eq!(err.unwrap_err(), RoutineBindingError::NoOverloads);
    assert_eq!(
        RoutineBindingError::NoOverloads.to_string(),
        "Routine binding overloads cannot be empty"
    );
    let err = RoutineBinding::new(Name::of(["", "pow"]).unwrap(), vec![pow()]);
    assert_eq!(err.unwrap_err(), RoutineBindingError::EmptyNamePart);
}

#[test]
fn fixed_overload_has_one_instance() {
    let f = pow();
    assert_eq!(f.signature().name(), "pow");
    assert_eq!(f.signature().arity(), 2);
    let instance = f.instance(&[PartiqlShape::Dynamic]).unwrap();
    let params: Vec<&str> = instance.parameters().iter().map(|p| p.name()).collect();
    assert_eq!(params, ["arg0", "arg1"]);
    assert!(instance.is_null_call());
    assert!(instance.is_missing_call());
    let mut bld = PartiqlNoIdShapeBuilder::default();
    assert_eq!(instance.returns(), &type_int!(bld));
    let sig = instance.with_null_call(false).with_missing_call(false);
    assert!(!sig.is_null_call() && !sig.is_missing_call());
}

// ---------------------------------------------------------------------------------------------
// LegacyCatalogAdapter over PartiqlCatalog
// ---------------------------------------------------------------------------------------------

fn legacy() -> Arc<dyn SharedCatalog> {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let mut catalog = PartiqlCatalog::default();
    let ints = type_bag!(bld, type_int!(bld));
    let strings = type_bag!(bld, type_string!(bld));
    catalog
        .add_type_entry(TypeEnvEntry::new("orders", &["purchases"], ints))
        .unwrap();
    catalog
        .add_type_entry(TypeEnvEntry::new("caf\u{e9}", &[], strings))
        .unwrap();
    Arc::new(catalog.to_shared_catalog())
}

#[test]
fn adapter_name_and_tables() {
    let adapter = LegacyCatalogAdapter::new(legacy());
    let s = Session::empty();
    assert_eq!(adapter.name(), "default");

    let name = adapter.resolve_table(&s, &reg(&["ORDERS"])).unwrap();
    assert_eq!(name, Name::of(["ORDERS"]).unwrap());
    let table = adapter.get_table(&s, &name).unwrap();
    assert!(table.schema().is_bag());
    assert_eq!(table.name(), &name);

    // Aliases resolve too.
    assert!(adapter.resolve_table(&s, &reg(&["purchases"])).is_some());
    // Non-ASCII: PartiqlCatalog ignores case by full Unicode case folding.
    assert!(adapter.resolve_table(&s, &reg(&["CAF\u{c9}"])).is_some());
    assert!(adapter.resolve_table(&s, &reg(&["missing"])).is_none());
    // Only the first part is matched; namespaced names have no table.
    assert_eq!(
        adapter.resolve_table(&s, &reg(&["orders", "total"])),
        Some(Name::of(["orders"]).unwrap())
    );
    assert!(adapter
        .get_table(&s, &Name::of(["x", "orders"]).unwrap())
        .is_none());
    assert!(adapter.get_functions(&s, "upper").is_empty());
    assert!(adapter.as_routine_catalog().is_none());
}

/// PartiqlCatalog's names ignore case even when quoted; the adapter keeps the inner catalog's
/// rules rather than rejecting a delimited match.
#[test]
fn adapter_delimited_follows_inner_catalog_rules() {
    let adapter = LegacyCatalogAdapter::new(legacy());
    let s = Session::empty();
    assert!(adapter
        .resolve_table(&s, &Identifier::delimited("orders"))
        .is_some());
    assert!(adapter
        .resolve_table(&s, &Identifier::delimited("ORDERS"))
        .is_some());
}

/// A shared catalog that honors quoting, as `partiql-tools`' catalog does: a quoted name as
/// written, a bare name folded to lowercase. It has the case-distinct tables `orders` and
/// `Orders`.
#[derive(Debug)]
struct Folding;

impl partiql_catalog::catalog::ReadOnlyCatalog for Folding {
    fn name(&self) -> &str {
        "folding"
    }
    fn get_function(&self, _: &str) -> Option<partiql_catalog::catalog::FunctionEntry<'_>> {
        None
    }
    fn get_function_by_id(
        &self,
        _: partiql_common::catalog::ObjectId,
    ) -> Option<partiql_catalog::catalog::FunctionEntry<'_>> {
        None
    }
    fn resolve_type(&self, name: &str) -> Option<partiql_catalog::catalog::TypeEntry> {
        self.resolve_binding_type(&partiql_value::BindingsName::CaseInsensitive(name.into()))
    }
    fn resolve_binding_type(
        &self,
        name: &partiql_value::BindingsName<'_>,
    ) -> Option<partiql_catalog::catalog::TypeEntry> {
        use partiql_value::BindingsName;
        let canonical = match name {
            BindingsName::CaseSensitive(s) => s.to_string(),
            BindingsName::CaseInsensitive(s) => s.to_lowercase(),
            _ => return None,
        };
        let (entry, ty) = match canonical.as_str() {
            "orders" => (1u64, PartiqlShape::Dynamic),
            "Orders" => (2, schema()),
            _ => return None,
        };
        Some(partiql_catalog::catalog::TypeEntry::new(
            partiql_common::catalog::ObjectId::new(1u64.into(), entry.into()),
            ty,
        ))
    }
}

impl SharedCatalog for Folding {}

#[test]
fn adapter_delimited_exact_when_inner_catalog_honors_quoting() {
    let shared: Arc<dyn SharedCatalog> = Arc::new(Folding);
    let adapter = LegacyCatalogAdapter::new(shared);
    let s = Session::empty();
    let quoted = adapter
        .resolve_table(&s, &Identifier::delimited("Orders"))
        .unwrap();
    assert!(adapter.get_table(&s, &quoted).unwrap().schema().is_bag());
    assert!(adapter
        .resolve_table(&s, &Identifier::delimited("ORDERS"))
        .is_none());
    // A regular name resolves by the inner catalog's folding, and the resolved name (the text as
    // written) gets back the same entry.
    let name = adapter.resolve_table(&s, &reg(&["ORDERS"])).unwrap();
    assert_eq!(name, Name::of(["ORDERS"]).unwrap());
    assert!(adapter.get_table(&s, &name).unwrap().schema().is_dynamic());
    // A regular `Orders` folds to `orders`, but its exact text names the other table, so the
    // adapter cannot name it and resolves nothing rather than the wrong table.
    assert!(adapter.resolve_table(&s, &reg(&["Orders"])).is_none());
    assert!(adapter.resolve_table(&s, &reg(&["orders"])).is_some());
}

#[test]
fn adapter_in_a_session_resolves_through_env_order() {
    let adapter: Arc<dyn TypedCatalog> = Arc::new(LegacyCatalogAdapter::new(legacy()));
    let s = session("default", &[], vec![adapter]);
    assert_eq!(
        resolved(&s, &reg(&["Orders", "total"])),
        Some(("default".into(), "\"Orders\"".into(), strs(&["total"])))
    );
    // Catalog-qualified through the adapter's name.
    assert_eq!(
        resolved(&s, &reg(&["DEFAULT", "orders"])),
        Some(("default".into(), "\"orders\"".into(), vec![]))
    );
}

/// The adapter matches by the inner catalog's case rules: PartiqlCatalog folds case fully, so
/// `STRASSE` finds `stra\u{df}e`, which `equals_ignore_case` (Java's rules) would not match.
#[test]
fn adapter_follows_partiql_catalog_full_folding() {
    let mut catalog = PartiqlCatalog::default();
    catalog
        .add_type_entry(TypeEnvEntry::new("stra\u{df}e", &[], PartiqlShape::Dynamic))
        .unwrap();
    let shared: Arc<dyn SharedCatalog> = Arc::new(catalog.to_shared_catalog());
    let adapter = LegacyCatalogAdapter::new(shared);
    let s = Session::empty();
    assert!(!equals_ignore_case("stra\u{df}e", "STRASSE"));
    assert!(adapter.resolve_table(&s, &reg(&["STRASSE"])).is_some());
    assert!(adapter
        .resolve_table(&s, &Identifier::delimited("STRASSE"))
        .is_some());
}

/// With a session namespace, step 1 prepends it, and the adapter matches only the first part.
#[test]
fn adapter_with_session_namespace() {
    let adapter: Arc<dyn TypedCatalog> = Arc::new(LegacyCatalogAdapter::new(legacy()));
    let s = session("default", &["shop"], vec![adapter]);
    assert_eq!(resolved(&s, &reg(&["orders"])), None);
    // Qualified, step 2 looks the identifier up as written.
    assert_eq!(
        resolved(&s, &reg(&["orders", "total"])),
        Some(("default".into(), "\"orders\"".into(), strs(&["total"])))
    );
}

/// An ambiguous catalog name matters only when step 3 is reached.
#[test]
fn step3_ambiguity_not_raised_when_earlier_step_resolves() {
    let catalogs = |main: Arc<dyn TypedCatalog>| {
        vec![
            main,
            TestCatalog::shared("archive", &[&["orders"]]),
            TestCatalog::shared("ARCHIVE", &[&["orders"]]),
        ]
    };
    let s = session(
        "main",
        &[],
        catalogs(TestCatalog::shared("main", &[&["archive", "orders"]])),
    );
    assert_eq!(
        resolved(&s, &reg(&["Archive", "orders"])),
        Some(("main".into(), "\"archive\".\"orders\"".into(), vec![]))
    );
    let s = session(
        "main",
        &["other"],
        catalogs(TestCatalog::shared("main", &[&["archive", "orders"]])),
    );
    assert_eq!(
        resolved(&s, &reg(&["Archive", "orders"])),
        Some(("main".into(), "\"archive\".\"orders\"".into(), vec![]))
    );
}

/// As in partiql-lang-kotlin, the system catalog replaces a catalog with the same name.
#[test]
fn system_catalog_replaces_same_named_catalog() {
    let s = Session::builder()
        .catalog(SYSTEM_CATALOG_NAME)
        .catalogs([TestCatalog::shared(SYSTEM_CATALOG_NAME, &[&["orders"]])])
        .build()
        .unwrap();
    assert_eq!(names(s.catalogs()), strs(&[SYSTEM_CATALOG_NAME]));
    assert_eq!(resolved(&s, &reg(&["orders"])), None);
}

/// With a namespace in an adapter-backed current catalog, an unqualified name does not resolve to
/// a table named like the namespace (a `SharedCatalog` has no namespaces).
#[test]
fn adapter_namespace_does_not_match_table_named_like_it() {
    let mut catalog = PartiqlCatalog::default();
    for name in ["orders", "shop"] {
        catalog
            .add_type_entry(TypeEnvEntry::new(name, &[], PartiqlShape::Dynamic))
            .unwrap();
    }
    let shared: Arc<dyn SharedCatalog> = Arc::new(catalog.to_shared_catalog());
    let adapter: Arc<dyn TypedCatalog> = Arc::new(LegacyCatalogAdapter::new(shared));
    let s = session("default", &["shop"], vec![adapter.clone()]);
    assert_eq!(resolved(&s, &reg(&["orders"])), None);
    // Written by the user, `shop.orders` is still the table `shop` with the path `orders`.
    assert_eq!(
        resolved(&s, &reg(&["shop", "orders"])),
        Some(("default".into(), "\"shop\"".into(), strs(&["orders"])))
    );
    // Without a namespace, nothing changes.
    let s = session("default", &[], vec![adapter]);
    assert_eq!(
        resolved(&s, &reg(&["orders"])),
        Some(("default".into(), "\"orders\"".into(), vec![]))
    );
}

#[test]
fn debug_output_omits_identity_and_property_values() {
    let builder = Session::builder()
        .catalog("example")
        .identity("sample-user")
        .property("sample-key", "sample-value");
    let shown = format!("{builder:?}");
    assert!(shown.contains("sample-key"));
    assert!(!shown.contains("sample-value") && !shown.contains("sample-user"));
    let shown = format!("{:?}", builder.build().unwrap());
    assert!(shown.contains("sample-key") && shown.contains("example"));
    assert!(!shown.contains("sample-value") && !shown.contains("sample-user"));
    let adapter = LegacyCatalogAdapter::new(legacy());
    assert_eq!(
        format!("{adapter:?}"),
        "LegacyCatalogAdapter { name: \"default\", .. }"
    );
}

#[test]
fn fixed_agg_overload_has_one_instance() {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let int = type_int!(bld);
    let agg: Arc<dyn AggOverload> = Arc::new(FixedAggOverload::new(RoutineSignature::with_types(
        "sum",
        [int.clone()],
        int,
    )));
    assert_eq!(agg.signature().name(), "sum");
    assert_eq!(agg.signature().arity(), 1);
    let instance = agg.instance(&[PartiqlShape::Undefined]).unwrap();
    assert_eq!(instance.parameters()[0].name(), "arg0");
    let binding = RoutineBinding::new(Name::of(["sum"]).unwrap(), vec![agg]).unwrap();
    assert_eq!(binding.overloads().len(), 1);
}
