use insta::assert_snapshot;
use partiql_catalog::catalog::PartiqlCatalog;
use partiql_eval::plan::EvaluationMode;
use partiql_extension_ion::decode::{IonDecodeResult, IonDecoderBuilder, IonDecoderConfig};

use crate::common::{compile, evaluate, lower, parse};
use partiql_extension_ion::Encoding;
use partiql_value::{tuple, Value};

mod common;

#[track_caller]
fn decode_ion_text(contents: &str, encoding: Encoding) -> IonDecodeResult {
    let reader = ion_rs_old::ReaderBuilder::new().build(contents)?;
    let mut iter =
        IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(encoding)).build(reader)?;

    let val = iter.next();

    val.unwrap()
}

#[track_caller]
fn pass(contents: &str) {
    let result = decode_ion_text(contents, Encoding::PartiqlEncodedAsIon);
    assert!(result.is_ok());
    dbg!(&result.unwrap());
}

#[track_caller]
fn fail(contents: &str) {
    let result = decode_ion_text(contents, Encoding::PartiqlEncodedAsIon);
    dbg!(&result);
    assert!(result.is_err());
}

#[test]
fn rfc_0025() {
    pass(include_str!("resources/rfc0025-example.ion"))
}

#[test]
fn gpml_paper() {
    pass(include_str!("resources/gpml-paper-example.ion"))
}

#[test]
fn empty_graph() {
    pass("$graph::{ nodes: [], edges: [] }")
}
#[test]
fn graph_wo_annotations() {
    pass("$graph::{ nodes: [], edges: [] }")
}
#[test]
fn empty_graph_no_edges() {
    fail("$graph::{ nodes: [] }")
}

#[test]
fn node_with_id_label_payload() {
    pass(r##"$graph::{ nodes: [ { id: n1, labels: ["a"], payload: 33 } ], edges: [] }"##)
}
#[test]
fn node_with_labels() {
    pass(r##"$graph::{ nodes: [ { id: n1, labels: ["a", "b", "c"] } ], edges: [] }"##)
}
#[test]
fn node_no_labels() {
    pass(r##"$graph::{ nodes: [ { id: n1, labels: [] } ], edges: [] }"##)
}
#[test]
fn node_complex_payload() {
    pass(r##"$graph::{ nodes: [ { id: n1, payload: {height: 12, weight: 152} } ], edges: [] }"##)
}
#[test]
fn node_no_id() {
    fail(r##"$graph::{ nodes: [ { labels: ["a"], payload: "must have the id!" } ], edges: [] }"##)
}

#[test]
fn edge_with_label() {
    pass(
        r##"$graph::{ nodes: [{id: n1}, {id: n2}],
                              edges: [ {id: e1, labels: ["go"], ends: (n1 -> n2)} ] }"##,
    )
}
#[test]
fn edge_with_labels() {
    pass(
        r##"$graph::{ nodes: [{id: n1}, {id: n2}],
                               edges: [ {id: e1, labels: ["go", "went", "gone"], ends: (n1 -> n2)} ] }"##,
    )
}
#[test]
fn undirected_edge_no_labels() {
    pass(
        r##"$graph::{ nodes: [{id: n1}, {id: n2}],
                               edges: [ {id: e1, labels: [], ends: (n1 -- n2)} ] }"##,
    )
}
#[test]
fn undirected_edge_complex_payload() {
    pass(
        r##"$graph::{ nodes: [{id: n1}, {id: n2}],
                               edges: [ {id: e1, ends: (n1 -- n2), payload: {length: 23, thickness: 3} }] }"##,
    )
}
#[test]
fn node_to_itself() {
    pass(
        r##"$graph::{ nodes: [{id: n1}, {id: n2}],
                               edges: [ {id: e1, ends: (n1 -- n1)},
                                        {id: e2, ends: (n2 <- n2)}, ] }"##,
    )
}
#[test]
fn edge_no_id() {
    fail(
        r##"$graph::{ nodes: [{id: n1}, {id: n2}],
                               edges: [ {labels: ["a"], ends: (n2 <- n1)} ] }"##,
    )
}
#[test]
fn edge_no_ends() {
    fail(
        r##"$graph::{ nodes: [{id: n1}, {id: n2}],
                               edges: [ { id: e1, labels: ["a"] } ] }"##,
    )
}

#[test]
fn nodes2_edges3() {
    pass(
        r##"$graph::{
                        nodes: [ {id: n1}, {id: n2} ],
                        edges: [ {id: e1, ends: (n1 -> n2)},
                                 {id: e2, ends: (n1 <- n2)},
                                 {id: e3, ends: (n1 -- n2)} ] }"##,
    )
}

#[test]
fn shared_identifiers() {
    pass(r##"$graph::{nodes: [ {id: x}, {id: y} ], edges: [ {id:x, ends: (x -> y)} ] }"##)
}

// The following examples are valid per ISL, but should be rejected by a processor.
#[test]
#[ignore] // TODO: this currently panics; fix to return error
fn edges_bad_nodes() {
    fail(r##"$graph::{nodes: [ {id: n1}, {id: n2} ], edges: [ {id:e1, ends: (z1 -> z2)} ] }"##)
}
#[test]
#[ignore] // TODO: this currently panics; fix to return error
fn repeated_identifers() {
    fail(
        r##"$graph::{
                        nodes: [ {id: n1}, {id: n1} ],
                        edges: [ {id:e1, ends: (n1 -> n1)},
                                 {id:e1, ends: (n1 -- n1)} ] }"##,
    )
}

#[track_caller]
fn snapshot_test_graph_eval(name: &'static str, contents: &'static str, query: &'static str) {
    let graph = decode_ion_text(contents, Encoding::PartiqlEncodedAsIon).expect("graph decode");
    let bindings = tuple![("g", graph)];

    let parsed = parse(query).expect("parse");
    let catalog = PartiqlCatalog::default();
    let lowered = lower(&catalog, &parsed).expect("lower");
    let plan = compile(EvaluationMode::Permissive, &catalog, lowered).expect("compile");
    let res = evaluate(plan, bindings.into());
    assert!(res.is_ok());
    assert_snapshot!(name, res.unwrap().result);
}

#[test]
fn select_star_rfc_0025() {
    // The RFC 0025 graph has no payloads so these just return an appropriate number of `{}`
    let contents = include_str!("resources/rfc0025-example.ion");
    snapshot_test_graph_eval("RFC0025 Nodes", contents, "(g MATCH (x))");
    snapshot_test_graph_eval("RFC0025 L Triples", contents, "(g MATCH (x) -> (y))");
    snapshot_test_graph_eval("RFC0025 LUR Triples", contents, "(g MATCH (x) - (y))");
}

#[test]
fn select_star_gpml_paper() {
    let contents = include_str!("resources/gpml-paper-example.ion");
    snapshot_test_graph_eval("GPML Nodes", contents, "(g MATCH (x))");
    snapshot_test_graph_eval("GPML L Triples", contents, "(g MATCH (x) -[e]-> (y))");
    snapshot_test_graph_eval("GPML LUR Triples", contents, "(g MATCH (x) -[e]- (y))");
}
