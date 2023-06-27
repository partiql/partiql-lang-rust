use indexmap::IndexMap;
use partiql_ast::ast::{CaseSensitivity, SymbolPrimitive};
use partiql_catalog::Catalog;
use partiql_logical::PathComponent::Index;
use partiql_logical::{BindingsOp, LogicalPlan, OpId, PathComponent, ValueExpr};
use partiql_types::{
    ArrayType, BagType, PartiqlType, StructConstraint, StructField, StructType, TypeKind,
};
use partiql_value::{BindingsName, Value};
use petgraph::algo::toposort;
use petgraph::graph::NodeIndex;
use petgraph::prelude::StableGraph;
use std::collections::HashMap;
use itertools::all;
use petgraph::visit::Walker;
use thiserror::Error;
use partiql_ast::ast::CustomTypeParam::Type;

#[macro_export]
macro_rules! ty_ctx {
    (($x:expr, $y:expr)) => (
        TypeEnvContext::from(($x, $y))
    );
}

#[macro_export]
macro_rules! ty_env {
    (($x:expr, $y:expr)) => (
        TypeEnv::from([($x, $y)])
    );
}

/// All errors that occurred during [`partiql_logical::LogicalPlan`] to [`eval::EvalPlan`] creation.
#[derive(Debug)]
pub struct PlanErr {
    pub errors: Vec<PlanningError>,
}

#[derive(Debug, Clone)]
enum LookupOrder {
    GlobalLocal,
    LocalGlobal,
}

#[derive(Debug, Clone)]
struct TypeEnvContext {
    env: TypeEnv,
    derived_type: Option<PartiqlType>,
}

impl TypeEnvContext {
    fn new() -> Self {
        TypeEnvContext::default()
    }

    fn env(&self) -> &TypeEnv {
        &self.env
    }

    fn derived_type(&self) -> &Option<PartiqlType> {
        &self.derived_type
    }

    fn add(env: &TypeEnv, derived_type: &PartiqlType) -> Self {
        TypeEnvContext { env: env.clone(), derived_type: Some(derived_type.clone())}
    }

    fn update_derived_type(&mut self, derived_type: &PartiqlType) -> Self {
        TypeEnvContext {
            env: self.env.clone(),
            derived_type: Some(derived_type.clone()),
        }
    }

}

impl Default for TypeEnvContext {
    fn default() -> Self {
        TypeEnvContext {
            env: TypeEnv::new(),
            derived_type: None,
        }
    }
}

impl From<(&TypeEnv, &PartiqlType)> for TypeEnvContext {
    fn from(value: (&TypeEnv, &PartiqlType)) -> Self {
        TypeEnvContext {
            env: value.0.clone(),
            derived_type: Some(value.1.clone()),
        }
    }
}

type TypeEnv = IndexMap<SymbolPrimitive, PartiqlType>;

/// An error that can happen during [`partiql_logical::LogicalPlan`] to [`eval::EvalPlan`] creation.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum PlanningError {
    /// Feature has not yet been implemented.
    #[error("Not yet implemented: {0}")]
    NotYetImplemented(String),
    /// Internal error that was not due to user input or API violation.
    #[error("Illegal State: {0}")]
    IllegalState(String),
}

pub struct PlanTyper<'c> {
    catalog: &'c dyn Catalog,
    logical_plan: LogicalPlan<BindingsOp>,
    errors: Vec<PlanningError>,
    type_env_stack: Vec<TypeEnvContext>,
    output: Option<PartiqlType>,
}

impl<'c> PlanTyper<'c> {
    pub fn new(catalog: &'c dyn Catalog, lg: &LogicalPlan<BindingsOp>) -> Self {
        PlanTyper {
            catalog,
            logical_plan: lg.clone(),
            errors: Default::default(),
            type_env_stack: Default::default(),
            output: None,
        }
    }

    pub fn type_plan(&mut self) -> Result<PartiqlType, PlanErr> {
        let ops = self.sort().expect("Ops");

        for idx in ops {
            if let Some(binop) = self.to_stable_graph().expect("Graph").node_weight(idx) {
                self.type_binop(binop)
            }
        }

        if self.errors.is_empty() {
            dbg!(&self.type_env_stack);
            dbg!(&self.output);
            Ok(self.output.clone().expect("PartiQL Type"))
        } else {
            Err(PlanErr {
                errors: self.errors.clone(),
            })
        }
    }

    fn type_binop(&mut self, op: &BindingsOp) -> () {
        match op {
            BindingsOp::Scan(partiql_logical::Scan { expr, as_key, .. }) => {
                self.type_vexpr(expr, LookupOrder::GlobalLocal);
                let type_ctx = self.local_type_env();
                let mut new_type_env = TypeEnv::new();
                for (name, ty) in type_ctx.env().iter() {
                    if as_key.is_empty() {
                        new_type_env.insert(name.clone(), self.element_type(ty).clone());
                    } else {
                        new_type_env.insert(
                            string_to_sym(as_key.as_str()),
                            self.element_type(ty).clone(),
                        );
                    }
                }
                self.type_env_stack.push(ty_ctx![(&new_type_env, &type_ctx.derived_type().clone().unwrap().clone())]);
            }
            BindingsOp::Pivot(_) => {}
            BindingsOp::Unpivot(_) => {}
            BindingsOp::Filter(_) => {}
            BindingsOp::OrderBy(_) => {}
            BindingsOp::LimitOffset(_) => {}
            BindingsOp::Join(_) => {}
            BindingsOp::SetOp => {}
            BindingsOp::Project(partiql_logical::Project { exprs }) => {
                for (k, v) in exprs.iter() {
                    self.type_vexpr(v, LookupOrder::LocalGlobal);
                    let type_ctx = self.local_type_env();
                    if let Some(ty) = type_ctx.env().get(&string_to_sym(k.as_str())) {
                        let ty = PartiqlType::new_struct(partiql_types::StructType::new(vec![
                            StructConstraint::Fields(vec![StructField::new(
                                k.as_str(),
                                ty.clone(),
                            )]),
                        ]));
                        let schema = PartiqlType::new_bag(BagType::new(Box::new(ty.clone())));
                        self.type_env_stack
                        .push(ty_ctx![(&ty_env![(string_to_sym("schema"), schema.clone())], &type_ctx.derived_type().clone().unwrap())]);
                        // .push(TypeEnvContext::from((TypeEnv::from([(string_to_sym("schema"), schema.clone())]), &type_ctx.derived_type().unwrap().clone())));
                    }
                }
            }
            BindingsOp::ProjectAll => {}
            BindingsOp::ProjectValue(_) => {}
            BindingsOp::ExprQuery(_) => {}
            BindingsOp::Distinct => {}
            BindingsOp::GroupBy(_) => {}
            BindingsOp::Having(_) => {}
            BindingsOp::Sink => {
                let type_env = self.local_type_env();
                if let Some(ty) = type_env.env().get(&string_to_sym("schema")) {
                    self.output = Some(ty.clone());
                }
            }
        }
    }

    fn type_vexpr(&mut self, v: &ValueExpr, lookup_order: LookupOrder) -> () {
        fn binding_to_sym(binding: &BindingsName) -> SymbolPrimitive {
            match binding {
                BindingsName::CaseSensitive(s) => SymbolPrimitive {
                    value: s.to_string(),
                    case: CaseSensitivity::CaseSensitive,
                },
                BindingsName::CaseInsensitive(s) => SymbolPrimitive {
                    value: s.to_string(),
                    case: CaseSensitivity::CaseInsensitive,
                },
            }
        }

        match v {
            ValueExpr::VarRef(binding_name) => {
                let sym = binding_to_sym(binding_name);
                let name = sym.clone().value;

                match lookup_order {
                    LookupOrder::GlobalLocal => {
                        if let Some(type_entry) = self.catalog.resolve_type(name.as_str()) {
                            let ty = type_entry.ty();

                            // let type_ctx = TypeEnvContext::from((TypeEnv::from([(sym.clone(), self.element_type(ty))], ty)));

                            let type_ctx = ty_ctx![(&ty_env![(sym.clone(), self.element_type(ty).clone())], ty)];

                            self.type_env_stack.push(type_ctx);
                        } else {
                            todo!("Local lookup after unsuccessful global lookup is not implemented yet")
                        }
                    }
                    LookupOrder::LocalGlobal => {
                        for type_env in self.type_env_stack.clone().into_iter().rev() {
                            if let Some(ty) = type_env.env().get(&sym) {
                                let mut new_type_env = TypeEnv::new();
                                if let TypeKind::Struct(s) = ty.kind() {
                                    for field in s.fields() {
                                        let sym = SymbolPrimitive {
                                            value: field.name().to_string(),
                                            case: CaseSensitivity::CaseInsensitive,
                                        };
                                        new_type_env.insert(sym, field.ty().clone());
                                    }
                                }

                                let type_ctx = ty_ctx![(&new_type_env, ty)];
                                self.type_env_stack.push(type_ctx);
                                break;
                            }
                        }
                    }
                }
            }
            ValueExpr::Path(v, components) => {
                self.type_vexpr(*&v, LookupOrder::LocalGlobal);
                let type_ctx = self.local_type_env().clone();

                for component in components {
                    match component {
                        PathComponent::Key(key) => {
                            let sym = binding_to_sym(key);
                            if let Some(ty) = type_ctx.env().get(&sym) {
                                let env = ty_env![(sym.clone(), ty.clone())];

                                self.type_env_stack.push(ty_ctx![(&env, &type_ctx.derived_type().clone().unwrap())]);
                            } else {
                                if let Some(derived_type) = type_ctx.derived_type() {
                                    match derived_type.kind() {
                                        TypeKind::Struct(s) => {
                                            dbg!(s);
                                            if s.is_partial() {
                                                self.type_env_stack
                                                .push(ty_ctx![(&ty_env![(sym.clone(), PartiqlType::new_any())], &type_ctx.derived_type().clone().unwrap())]);
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        },
                        Index(_) => todo!(),
                        PathComponent::KeyExpr(_) => todo!(),
                        PathComponent::IndexExpr(_) => todo!()
                    }
                }
            }
            ValueExpr::Lit(v) => {
                let kind = match **v {
                    Value::Null => TypeKind::Null,
                    Value::Missing => TypeKind::Missing,
                    Value::Integer(_) => TypeKind::Int,
                    Value::Decimal(_) => TypeKind::Decimal,
                    Value::Boolean(_) => TypeKind::Bool,
                    Value::String(_) => TypeKind::String,
                    Value::Tuple(_) => TypeKind::Struct(StructType::new_any()),
                    Value::List(_) => TypeKind::Array(ArrayType::new_any()),
                    Value::Bag(_) => TypeKind::Bag(BagType::new_any()),
                    _ => todo!(),
                };

                let ty = PartiqlType::new(kind);
                let new_type_env = IndexMap::from([(string_to_sym("_1"), ty.clone())]);
                self.type_env_stack.push(ty_ctx![(&new_type_env, &ty)]);
            }
            _ => todo!(),
        }
    }

    fn sort(&self) -> Result<Vec<NodeIndex>, PlanErr> {
        let graph = self.to_stable_graph().expect("Graph");
        // We are only interested in DAGs that can be used as execution plans, which leads to the
        // following definition.
        // A DAG is a directed, cycle-free graph G = (V, E) with a denoted root node v0 ∈ V such
        // that all v ∈ V \{v0} are reachable from v0. Note that this is the definition of trees
        // without the condition |E| = |V | − 1. Hence, all trees are DAGs.
        // Reference: https://link.springer.com/article/10.1007/s00450-009-0061-0
        toposort(&graph, None).map_err(|e| PlanErr {
            errors: vec![PlanningError::IllegalState(format!(
                "Malformed plan detected: {e:?}"
            ))],
        })
    }

    fn to_stable_graph(&self) -> Result<StableGraph<BindingsOp, u8>, PlanErr> {
        let lg = &self.logical_plan;
        let flows = lg.flows();

        let mut graph: StableGraph<_, _> = Default::default();
        let mut seen = HashMap::new();

        for (s, d, w) in flows {
            let mut add_node = |op_id: &OpId| {
                let logical_op = lg.operator(*op_id).unwrap();
                *seen
                    .entry(*op_id)
                    .or_insert_with(|| graph.add_node(logical_op.clone()))
            };

            let (s, d) = (add_node(s), add_node(d));
            graph.add_edge(s, d, *w);
        }

        Ok(graph)
    }

    #[inline]
    fn element_type<'a>(&'a self, ty: &'a PartiqlType) -> &PartiqlType {
        match ty.kind() {
            TypeKind::Bag(b) => b.element_type(),
            TypeKind::Array(a) => a.element_type(),
            TypeKind::Any => todo!(),
            TypeKind::AnyOf(_) => todo!(),
            _ => &ty,
        }
    }

    fn local_type_env(&self) -> &TypeEnvContext {
        self.type_env_stack.last().expect("TypeEnv")
    }
}

fn string_to_sym(name: &str) -> SymbolPrimitive {
    SymbolPrimitive {
        value: name.to_string(),
        case: CaseSensitivity::CaseInsensitive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{logical, LogicalPlanner};
    use partiql_ast_passes::error::AstTransformationError;
    use partiql_catalog::{PartiqlCatalog, TypeEnvEntry};
    use partiql_parser::{Parsed, Parser};
    use partiql_types::{bag, int, r#struct, str, struct_fields, BagType, StructType};

    #[test]
    fn simple_sfw() {
        let fields = struct_fields![("id", int!()), ("name", str!()),];
        let customers_schema = bag![r#struct![vec![fields]]];

        let mut catalog = PartiqlCatalog::default();
        let _oid = catalog.add_type_entry(TypeEnvEntry::new("customers", &[], customers_schema));

        let query = "SELECT customers.id, customers.name FROM customers";
        // let query = "SELECT customers.id FROM {'id': 1, 'name': 'Bob'} AS customers";
        let parsed = parse(query);
        let lg = lower(&parsed).expect("Logical plan");

        let mut typer = PlanTyper::new(&catalog, &lg);
        let actual = typer.type_plan().expect("typer");

        let expected_fields = struct_fields![("id", int!()),("name", str!())];
        let expected = bag![r#struct![vec![expected_fields]]];
        assert_eq!(actual, expected);
    }

    #[track_caller]
    fn parse(text: &str) -> Parsed {
        Parser::default().parse(text).unwrap()
    }

    #[track_caller]
    fn lower(
        parsed: &Parsed,
    ) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
        let catalog = PartiqlCatalog::default();
        let planner = LogicalPlanner::new(&catalog);
        planner.lower(parsed)
    }
}
