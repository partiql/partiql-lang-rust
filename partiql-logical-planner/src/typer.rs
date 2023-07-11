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
use std::collections::{BTreeSet, HashMap};
use thiserror::Error;

#[macro_export]
macro_rules! ty_ctx {
    (($x:expr, $y:expr)) => {
        TypeEnvContext::from(($x, $y))
    };
}

#[macro_export]
macro_rules! ty_env {
    (($x:expr, $y:expr)) => {
        TypeEnv::from([($x, $y)])
    };
}

/// All errors that occurred during [`partiql_logical::LogicalPlan`] to [`eval::EvalPlan`] creation.
#[derive(Debug)]
pub struct PlanErr {
    pub errors: Vec<TypingError>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum LookupOrder {
    GlobalLocal,
    LocalGlobal,
}

#[derive(Debug, Clone)]
struct TypeEnvContext {
    env: TypeEnv,
    derived_type: Option<PartiqlType>,
}

#[allow(dead_code)]
impl TypeEnvContext {
    fn new() -> Self {
        TypeEnvContext::default()
    }

    fn env(&self) -> &TypeEnv {
        &self.env
    }

    fn derived_type(&self) -> Option<&PartiqlType> {
        self.derived_type.as_ref()
    }

    fn add(env: &TypeEnv, derived_type: &PartiqlType) -> Self {
        TypeEnvContext {
            env: env.clone(),
            derived_type: Some(derived_type.clone()),
        }
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

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum TypingError {
    /// Feature has not yet been implemented.
    #[error("Not yet implemented: {0}")]
    NotYetImplemented(String),
    /// Internal error that was not due to user input or API violation.
    #[error("Illegal State: {0}")]
    IllegalState(String),
}

#[derive(Debug, Clone)]
pub struct PlanTyper<'c> {
    catalog: &'c dyn Catalog,
    logical_plan: LogicalPlan<BindingsOp>,
    errors: Vec<TypingError>,
    type_env_stack: Vec<TypeEnvContext>,
    output: Option<PartiqlType>,
}

#[allow(dead_code)]
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
            Ok(self.output.clone().expect("PartiQL Type"))
        } else {
            Err(PlanErr {
                errors: self.errors.clone(),
            })
        }
    }

    fn type_binop(&mut self, op: &BindingsOp) {
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
                self.type_env_stack.push(ty_ctx![(
                    &new_type_env,
                    &type_ctx.derived_type().unwrap().clone()
                )]);
            }
            BindingsOp::Pivot(_) => {}
            BindingsOp::Unpivot(_) => {}
            BindingsOp::Filter(_) => {}
            BindingsOp::OrderBy(_) => {}
            BindingsOp::LimitOffset(_) => {}
            BindingsOp::Join(_) => {}
            BindingsOp::SetOp => {}
            BindingsOp::Project(partiql_logical::Project { exprs }) => {
                let mut fields = vec![];
                let derived_type_ctx = self.local_type_env().clone();
                for (k, v) in exprs.iter() {
                    self.type_vexpr(v, LookupOrder::LocalGlobal);
                    let type_ctx = self.local_type_env().clone();
                    if let Some(ty) = type_ctx.env().get(&string_to_sym(k.as_str())) {
                        fields.push(StructField::new(k.as_str(), ty.clone()));
                    }
                }
                let ty = PartiqlType::new_struct(partiql_types::StructType::new(BTreeSet::from([
                    StructConstraint::Fields(fields),
                ])));
                let schema = PartiqlType::new_bag(BagType::new(Box::new(ty)));
                self.type_env_stack.push(ty_ctx![(
                    &ty_env![(string_to_sym("schema"), schema)],
                    &derived_type_ctx.derived_type().unwrap().clone()
                )]);
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

    fn type_vexpr(&mut self, v: &ValueExpr, lookup_order: LookupOrder) {
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
                            let type_ctx =
                                ty_ctx![(&ty_env![(sym, self.element_type(ty).clone())], ty)];

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
                self.type_vexpr(&v, LookupOrder::LocalGlobal);
                let type_ctx = self.local_type_env().clone();

                for component in components {
                    match component {
                        PathComponent::Key(key) => {
                            let sym = binding_to_sym(key);
                            if let Some(ty) = type_ctx.env().get(&sym) {
                                let env = ty_env![(sym.clone(), ty.clone())];

                                self.type_env_stack.push(ty_ctx![(
                                    &env,
                                    &type_ctx.derived_type().unwrap().clone()
                                )]);
                            } else if let Some(derived_type) = type_ctx.derived_type() {
                                if let TypeKind::Struct(s) = derived_type.kind() {
                                    // dbg!(s);
                                    if s.is_partial() {
                                        self.type_env_stack.push(ty_ctx![(
                                            &ty_env![(sym.clone(), PartiqlType::new_any())],
                                            &type_ctx.derived_type().unwrap().clone()
                                        )]);
                                    }
                                } else {
                                    todo!()
                                }
                            }
                        }
                        Index(_) => todo!(),
                        PathComponent::KeyExpr(_) => todo!(),
                        PathComponent::IndexExpr(_) => todo!(),
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
        toposort(&graph, None).map_err(|e| PlanErr {
            errors: vec![TypingError::IllegalState(format!(
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
            _ => ty,
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
    use partiql_types::{any, bag, int, r#struct, str, struct_fields, BagType, StructType};

    #[test]
    fn simple_sfw() {
        let fields = struct_fields![("id", int!()), ("name", str!()),];
        let customers_schema = bag![r#struct![BTreeSet::from([fields])]];

        let mut catalog = PartiqlCatalog::default();
        let _oid = catalog.add_type_entry(TypeEnvEntry::new(
            "customers",
            &[],
            customers_schema.clone(),
        ));

        let query = "SELECT customers.id, customers.name, customers.age FROM customers";
        let parsed = parse(query);
        let lg = lower(&parsed).expect("Logical plan");

        let mut typer = PlanTyper::new(&catalog, &lg);
        let actual = typer.type_plan().expect("typer");

        let expected_fields = vec![
            StructField::new("id", int!()),
            StructField::new("name", str!()),
            StructField::new("age", any!()),
        ];

        println!("{:?}", &actual);
        match &actual.kind() {
            TypeKind::Bag(b) => {
                if let TypeKind::Struct(s) = b.element_type().kind() {
                    let fields = s.fields();
                    let f: Vec<_> = fields
                        .iter()
                        .filter(|f| !expected_fields.contains(f))
                        .collect();
                    assert![f.is_empty()];
                }
            }
            _ => panic!("expected bag type"),
        }
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
