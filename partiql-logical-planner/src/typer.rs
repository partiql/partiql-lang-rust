use crate::typer::LookupOrder::{GlobalLocal, LocalGlobal};
use indexmap::{IndexMap, IndexSet};
use partiql_ast::ast::{CaseSensitivity, SymbolPrimitive};
use partiql_catalog::Catalog;
use partiql_logical::{BindingsOp, LogicalPlan, OpId, PathComponent, ValueExpr, VarRefType};
use partiql_types::{
    dynamic, undefined, ArrayType, BagType, PartiqlShape, ShapeResultError, Static,
    StructConstraint, StructField, StructType,
};
use partiql_value::{BindingsName, Value};
use petgraph::algo::toposort;
use petgraph::graph::NodeIndex;
use petgraph::prelude::StableGraph;
use std::collections::HashMap;
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
        LocalTypeEnv::from([($x, $y)])
    };
}

const OUTPUT_SCHEMA_KEY: &str = "_output_schema";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypeErr {
    pub errors: Vec<TypingError>,
    pub output: Option<PartiqlShape>,
}

impl From<ShapeResultError> for TypeErr {
    fn from(value: ShapeResultError) -> Self {
        TypeErr {
            errors: vec![TypingError::InvalidType(value)],
            output: None,
        }
    }
}

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
    /// Represents an error in type checking
    #[error("TypeCheck: {0}")]
    TypeCheck(String),

    #[error("TypeCheck: {0}")]
    InvalidType(#[from] ShapeResultError),
}

#[derive(Debug, Clone)]
pub enum TypingMode {
    Permissive,
    Strict,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum LookupOrder {
    /// Global then Local
    GlobalLocal,
    /// Local then Global
    LocalGlobal,
    /// Signals for delegating the lookup order, e.g., to [VarRefType]
    Delegate,
}

/// Represents a type environment context
#[derive(Debug, Clone)]
struct TypeEnvContext {
    env: LocalTypeEnv,
    /// Represents the type that is used for creating the `env` in the [TypeEnvContext]
    derived_type: PartiqlShape,
}

#[allow(dead_code)]
impl TypeEnvContext {
    fn new() -> Self {
        TypeEnvContext::default()
    }

    fn env(&self) -> &LocalTypeEnv {
        &self.env
    }

    fn derived_type(&self) -> &PartiqlShape {
        &self.derived_type
    }
}

impl Default for TypeEnvContext {
    fn default() -> Self {
        TypeEnvContext {
            env: LocalTypeEnv::new(),
            derived_type: dynamic!(),
        }
    }
}

impl From<(&LocalTypeEnv, &PartiqlShape)> for TypeEnvContext {
    fn from(value: (&LocalTypeEnv, &PartiqlShape)) -> Self {
        TypeEnvContext {
            env: value.0.clone(),
            derived_type: value.1.clone(),
        }
    }
}

/// Represents a Local Type Environment as opposed to the Global Type Environment in the Catalog.
type LocalTypeEnv = IndexMap<SymbolPrimitive, PartiqlShape>;

#[derive(Debug, Clone)]
pub struct PlanTyper<'c> {
    typing_mode: TypingMode,
    catalog: &'c dyn Catalog,
    logical_plan: LogicalPlan<BindingsOp>,
    errors: Vec<TypingError>,
    type_env_stack: Vec<TypeEnvContext>,
    current_bindings_op: Option<BindingsOp>,
    output: Option<PartiqlShape>,
}

#[allow(dead_code)]
impl<'c> PlanTyper<'c> {
    /// Creates a new [`PlanTyper`] for the given Catalog and Intermediate Representation with `Strict` Typing Mode.
    pub fn new_strict(catalog: &'c dyn Catalog, ir: &LogicalPlan<BindingsOp>) -> Self {
        PlanTyper {
            typing_mode: TypingMode::Strict,
            catalog,
            logical_plan: ir.clone(),
            errors: Default::default(),
            type_env_stack: Default::default(),
            current_bindings_op: Default::default(),
            output: None,
        }
    }

    /// Creates a new [`PlanTyper`] for the given Catalog and Intermediate Representation with `Permissive` Typing Mode.
    pub fn new_permissive(catalog: &'c dyn Catalog, lg: &LogicalPlan<BindingsOp>) -> Self {
        PlanTyper {
            typing_mode: TypingMode::Permissive,
            catalog,
            logical_plan: lg.clone(),
            errors: Default::default(),
            type_env_stack: Default::default(),
            current_bindings_op: Default::default(),
            output: None,
        }
    }

    /// Returns the typing result for the Typer
    pub fn type_plan(&mut self) -> Result<PartiqlShape, TypeErr> {
        let ops = self.sort()?;

        for idx in ops {
            let graph = self.to_stable_graph()?;
            if let Some(binop) = graph.node_weight(idx) {
                self.type_bindings_op(binop);
            }
        }

        if self.errors.is_empty() {
            Ok(self.output.clone().unwrap_or(undefined!()))
        } else {
            let output_schema = self.get_singleton_type_from_env();
            Err(TypeErr {
                errors: self.errors.clone(),
                output: Some(output_schema),
            })
        }
    }

    fn type_bindings_op(&mut self, op: &BindingsOp) {
        self.current_bindings_op = Some(op.clone());
        match op {
            BindingsOp::Scan(partiql_logical::Scan {
                expr,
                as_key,
                at_key,
            }) => {
                if let Some(_at_key) = at_key {
                    self.errors.push(TypingError::NotYetImplemented(
                        "Scan operator with AT key is not implemented yet".to_string(),
                    ));
                }

                self.type_vexpr(expr, LookupOrder::Delegate);
                if !as_key.is_empty() {
                    let type_ctx = &self.local_type_ctx();
                    for (_name, ty) in type_ctx.env() {
                        if let Ok(_s) = ty.expect_struct() {
                            self.type_env_stack.push(ty_ctx![(
                                &ty_env![(string_to_sym(as_key.as_str()), ty.clone())],
                                ty
                            )]);
                        }
                    }
                }
            }
            BindingsOp::Project(partiql_logical::Project { exprs }) => {
                let fields = exprs.iter().map(|(k, v)| {
                    self.type_vexpr(v, LookupOrder::LocalGlobal);
                    StructField::new(k.as_str(), self.get_singleton_type_from_env())
                });

                let ty = PartiqlShape::new_struct(StructType::new(IndexSet::from([
                    StructConstraint::Fields(fields.collect()),
                ])));

                let derived_type_ctx = self.local_type_ctx();
                let derived_type = &self.derived_type(&derived_type_ctx);
                let schema = if derived_type.is_ordered_collection() {
                    PartiqlShape::new_array(ArrayType::new(Box::new(ty)))
                } else if derived_type.is_unordered_collection() {
                    PartiqlShape::new_bag(BagType::new(Box::new(ty)))
                } else {
                    self.errors.push(TypingError::IllegalState(format!(
                        "Expecting Collection for the output Schema but found {:?}",
                        &ty
                    )));
                    ty
                };

                // Marking the output Schema in Typing Environment:
                //
                // The clauses of an PartiQL SFW query are evaluated in the following order:
                // WITH, FROM, LET, WHERE, GROUP BY, HAVING, LETTING (which is special to PartiQL),
                // ORDER BY, LIMIT / OFFSET
                // and SELECT (or SELECT VALUE or PIVOT, which are both special to ion PartiQL).
                // -- PartiQL Spec. 2019 Section 3.3:
                self.type_env_stack.push(ty_ctx![(
                    &ty_env![(string_to_sym(OUTPUT_SCHEMA_KEY), schema)],
                    derived_type
                )]);
            }
            BindingsOp::Sink => {
                if let Some(ty) =
                    self.retrieve_type_from_local_ctx(&string_to_sym(OUTPUT_SCHEMA_KEY))
                {
                    self.output = Some(ty);
                }
            }
            _ => self.errors.push(TypingError::NotYetImplemented(format!(
                "Unsupported BindingOperator: {:?}",
                &op
            ))),
        }
    }

    fn type_vexpr(&mut self, v: &ValueExpr, lookup_order: LookupOrder) {
        fn binding_to_sym(binding: &BindingsName<'_>) -> SymbolPrimitive {
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
            ValueExpr::VarRef(binding_name, _) => {
                let key = binding_to_sym(binding_name);
                match lookup_order {
                    GlobalLocal => {
                        let ty = self.resolve_global_then_local(&key);
                        self.type_varef(&key, &ty);
                    }
                    LocalGlobal => {
                        let ty = self.resolve_local_then_global(&key);
                        self.type_varef(&key, &ty);
                    }
                    LookupOrder::Delegate => self.type_vexpr(v, self.lookup_order(v)),
                };
            }
            ValueExpr::Path(v, components) => {
                self.type_vexpr(v, LookupOrder::Delegate);
                for component in components {
                    match component {
                        PathComponent::Key(key) => {
                            let var = ValueExpr::VarRef(key.clone(), VarRefType::Local);
                            self.type_vexpr(&var, LookupOrder::LocalGlobal);

                            let key_as_sym = binding_to_sym(key);
                            if let Some(ty) = self.retrieve_type_from_local_ctx(&key_as_sym) {
                                let ctx = ty_ctx![(&ty_env![(key_as_sym, ty.clone())], &ty)];
                                self.type_env_stack.push(ctx);
                            } else {
                                let ctx =
                                    ty_ctx![(&ty_env![(key_as_sym, undefined!())], &undefined!())];
                                self.type_env_stack.push(ctx);
                            }
                        }
                        PathComponent::Index(_) => {
                            self.errors.push(TypingError::NotYetImplemented(
                                "Typing [Index] [PathComponent]s".to_string(),
                            ));
                        }
                        PathComponent::KeyExpr(_) => {
                            self.errors.push(TypingError::NotYetImplemented(
                                "Typing [KeyExpr] [PathComponent]s".to_string(),
                            ));
                        }
                        PathComponent::IndexExpr(_) => {
                            self.errors.push(TypingError::NotYetImplemented(
                                "Typing [IndexExpr] [PathComponent]s".to_string(),
                            ));
                        }
                    }
                }
            }
            ValueExpr::Lit(v) => {
                let ty = match **v {
                    Value::Null => PartiqlShape::Undefined,
                    Value::Missing => PartiqlShape::Undefined,
                    Value::Integer(_) => PartiqlShape::new(Static::Int),
                    Value::Decimal(_) => PartiqlShape::new(Static::Decimal),
                    Value::Boolean(_) => PartiqlShape::new(Static::Bool),
                    Value::String(_) => PartiqlShape::new(Static::String),
                    Value::Tuple(_) => PartiqlShape::new(Static::Struct(StructType::new_any())),
                    Value::List(_) => PartiqlShape::new(Static::Array(ArrayType::new_any())),
                    Value::Bag(_) => PartiqlShape::new(Static::Bag(BagType::new_any())),
                    _ => {
                        self.errors.push(TypingError::NotYetImplemented(
                            "Unsupported Literal".to_string(),
                        ));
                        PartiqlShape::Undefined
                    }
                };

                let new_type_env = IndexMap::from([(string_to_sym("_1"), ty.clone())]);
                self.type_env_stack.push(ty_ctx![(&new_type_env, &ty)]);
            }
            ValueExpr::DynamicLookup(v) => {
                if v.is_empty() {
                    self.errors.push(TypingError::IllegalState(format!(
                        "Unexpected Empty DynamicLookup found: {:?}",
                        &v
                    )));
                }

                // TODO for Typing we handle multiple lookups through `[LookupOrder]` hence using
                // the first element. Remove this workaround once we remove DynamicLookup
                let expr = &v[0];
                self.type_vexpr(expr, self.lookup_order(expr));
            }
            _ => self.errors.push(TypingError::NotYetImplemented(format!(
                "Unsupported Value Expression: {:?}",
                &v
            ))),
        }
    }

    fn sort(&self) -> Result<Vec<NodeIndex>, TypeErr> {
        let graph = self.to_stable_graph()?;

        toposort(&graph, None).map_err(|e| TypeErr {
            errors: vec![TypingError::IllegalState(format!(
                "Malformed plan detected: {e:?}"
            ))],
            output: None,
        })
    }

    fn to_stable_graph(&self) -> Result<StableGraph<BindingsOp, u8>, TypeErr> {
        let lg = &self.logical_plan;
        let flows = lg.flows();

        let mut graph: StableGraph<_, _> = Default::default();
        let mut seen = HashMap::new();

        for (s, d, w) in flows {
            let mut add_node = |op_id: &OpId| {
                if let Some(logical_op) = lg.operator(*op_id) {
                    Ok(*seen
                        .entry(*op_id)
                        .or_insert_with(|| graph.add_node(logical_op.clone())))
                } else {
                    Err(TypeErr {
                        errors: vec![TypingError::IllegalState("Malformed IR".to_string())],
                        output: None,
                    })
                }
            };

            let (s, d) = (add_node(s)?, add_node(d)?);
            graph.add_edge(s, d, *w);
        }

        Ok(graph)
    }

    fn element_type<'a>(&'a mut self, ty: &'a PartiqlShape) -> PartiqlShape {
        match ty {
            PartiqlShape::Dynamic => dynamic!(),
            PartiqlShape::Static(s) => match s.ty() {
                Static::Bag(b) => b.element_type().clone(),
                Static::Array(a) => a.element_type().clone(),
                _ => ty.clone(),
            },
            undefined!() => {
                todo!("Undefined type in catalog")
            }
            PartiqlShape::AnyOf(_any_of) => ty.clone(),
        }
    }

    fn retrieve_type_from_local_ctx(&mut self, key: &SymbolPrimitive) -> Option<PartiqlShape> {
        let type_ctx = self.local_type_ctx();
        let env = type_ctx.env().clone();
        let derived_type = self.derived_type(&type_ctx);

        if let Some(ty) = env.get(key) {
            Some(ty.clone())
        } else if let Ok(s) = derived_type.expect_struct() {
            if s.is_partial() {
                Some(dynamic!())
            } else {
                match &self.typing_mode {
                    TypingMode::Permissive => Some(undefined!()),
                    TypingMode::Strict => {
                        self.errors.push(TypingError::TypeCheck(format!(
                            "No Typing Information for {:?} in closed Schema {:?}",
                            &key, &derived_type
                        )));
                        None
                    }
                }
            }
        } else if derived_type.is_dynamic() {
            Some(dynamic!())
        } else {
            self.errors.push(TypingError::IllegalState(format!(
                "Illegal Derive Type {:?}",
                &derived_type
            )));
            None
        }
    }

    fn derived_type(&mut self, ty_ctx: &TypeEnvContext) -> PartiqlShape {
        let ty = ty_ctx.derived_type();
        ty.clone()
    }

    fn local_type_ctx(&mut self) -> TypeEnvContext {
        let out = self
            .type_env_stack
            .last()
            .ok_or_else(|| TypingError::IllegalState("Malformed TypeEnv stack".to_string()));

        match out {
            Ok(out) => out.clone(),
            Err(err) => {
                self.errors.push(err);
                TypeEnvContext::new()
            }
        }
    }

    fn lookup_order(&self, v: &ValueExpr) -> LookupOrder {
        match v {
            ValueExpr::VarRef(_, varef_type) => match varef_type {
                VarRefType::Global => GlobalLocal,
                VarRefType::Local => LocalGlobal,
            },
            _ => match self.current_bindings_op {
                Some(BindingsOp::Scan(_)) => GlobalLocal,
                _ => LocalGlobal,
            },
        }
    }

    fn resolve_global_then_local(&mut self, key: &SymbolPrimitive) -> PartiqlShape {
        let ty = self.resolve_global(key);
        match ty.is_undefined() {
            true => self.resolve_local(key),
            false => ty,
        }
    }

    fn resolve_local_then_global(&mut self, key: &SymbolPrimitive) -> PartiqlShape {
        let ty = self.resolve_local(key);
        match ty.is_undefined() {
            true => self.resolve_global(key),
            false => ty,
        }
    }

    fn resolve_global(&mut self, key: &SymbolPrimitive) -> PartiqlShape {
        if let Some(type_entry) = self.catalog.resolve_type(key.value.as_str()) {
            let ty = self.element_type(type_entry.ty());
            ty
        } else {
            undefined!()
        }
    }

    fn resolve_local(&mut self, key: &SymbolPrimitive) -> PartiqlShape {
        for type_ctx in self.type_env_stack.iter().rev() {
            if let Some(ty) = type_ctx.env().get(key) {
                return ty.clone();
            }
        }

        undefined!()
    }

    fn type_with_undefined(&mut self, key: &SymbolPrimitive) {
        if let TypingMode::Permissive = &self.typing_mode {
            // TODO Revise this once the following discussion is conclusive and spec. is
            // in place: https://github.com/partiql/partiql-spec/discussions/64
            let type_ctx = ty_ctx![(&ty_env![(key.clone(), undefined!())], &undefined!())];

            self.type_env_stack.push(type_ctx);
        }
    }

    // A helper function to extract one type out of the environment when we expect it.
    // E.g., in projections, when we expect to infer one type from the project list items.
    fn get_singleton_type_from_env(&mut self) -> PartiqlShape {
        let ctx = self.local_type_ctx();
        let env = ctx.env();
        if env.len() != 1 {
            self.errors.push(TypingError::IllegalState(format!(
                "Unexpected Typing Environment; expected typing environment with only one type but found {:?} types",
                &env.len()
            )));
            undefined!()
        } else {
            env[0].clone()
        }
    }

    fn type_varef(&mut self, key: &SymbolPrimitive, ty: &PartiqlShape) {
        if ty.is_undefined() {
            self.type_with_undefined(key);
        } else {
            let mut new_type_env = LocalTypeEnv::new();
            if let Ok(s) = ty.expect_struct() {
                for b in to_bindings(&s) {
                    new_type_env.insert(b.0, b.1);
                }

                let type_ctx = ty_ctx![(&new_type_env, ty)];
                self.type_env_stack.push(type_ctx);
            } else {
                new_type_env.insert(key.clone(), ty.clone());
                let type_ctx = ty_ctx![(&new_type_env, ty)];
                self.type_env_stack.push(type_ctx);
            }
        }
    }
}

fn string_to_sym(name: &str) -> SymbolPrimitive {
    SymbolPrimitive {
        value: name.to_string(),
        case: CaseSensitivity::CaseInsensitive,
    }
}

fn to_bindings(s: &StructType) -> Vec<(SymbolPrimitive, PartiqlShape)> {
    s.fields()
        .map(|field| {
            let sym = SymbolPrimitive {
                value: field.name().to_string(),
                case: CaseSensitivity::CaseInsensitive,
            };

            (sym, field.ty().clone())
        })
        .collect()
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
        // Closed schema with `Strict` typing mode.
        assert_query_typing(
            TypingMode::Strict,
            "SELECT customers.id, customers.name FROM customers",
            create_customer_schema(
                false,
                [
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("age", dynamic!()),
                ]
                .into(),
            ),
            vec![
                StructField::new("id", int!()),
                StructField::new("name", str!()),
            ],
        )
        .expect("Type");

        // Closed Schema with and without prefix in projections
        assert_query_typing(
            TypingMode::Strict,
            "SELECT id, customers.name FROM customers",
            create_customer_schema(
                false,
                [
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("age", dynamic!()),
                ]
                .into(),
            ),
            vec![
                StructField::new("id", int!()),
                StructField::new("name", str!()),
            ],
        )
        .expect("Type");

        // Open schema with `Strict` typing mode and `age` non-existent projection.
        assert_query_typing(
            TypingMode::Strict,
            "SELECT customers.id, customers.name, customers.age FROM customers",
            create_customer_schema(
                true,
                [
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("age", dynamic!()),
                ]
                .into(),
            ),
            vec![
                StructField::new("id", int!()),
                StructField::new("name", str!()),
                StructField::new("age", dynamic!()),
            ],
        )
        .expect("Type");

        // Closed Schema with `Permissive` typing mode and `age` non-existent projection.
        assert_query_typing(
            TypingMode::Permissive,
            "SELECT customers.id, customers.name, customers.age FROM customers",
            create_customer_schema(
                false,
                [
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                ]
                .into(),
            ),
            vec![
                StructField::new("id", int!()),
                StructField::new("name", str!()),
                StructField::new("age", undefined!()),
            ],
        )
        .expect("Type");

        // Open Schema with `Strict` typing mode and `age` in nested attribute.
        let details_fields = struct_fields![("age", int!())];
        let details = r#struct![IndexSet::from([details_fields])];

        assert_query_typing(
            TypingMode::Strict,
            "SELECT customers.id, customers.name, customers.details.age FROM customers",
            create_customer_schema(
                true,
                [
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("details", details.clone()),
                ]
                .into(),
            ),
            vec![
                StructField::new("id", int!()),
                StructField::new("name", str!()),
                StructField::new("age", int!()),
            ],
        )
        .expect("Type");

        // Open Schema with `Strict` typing mode and `bar` in nested attribute.
        assert_query_typing(
            TypingMode::Strict,
            "SELECT customers.id, customers.name, customers.details.age, customers.details.foo.bar FROM customers",
            create_customer_schema(true, [
                StructField::new("id", int!()),
                StructField::new("name", str!()),
                StructField::new("details", details.clone()),
            ].into()),
            vec![
                StructField::new("id", int!()),
                StructField::new("name", str!()),
                StructField::new("age", int!()),
                StructField::new("bar", dynamic!()),
            ],
        )
            .expect("Type");
    }

    #[test]
    fn simple_sfw_with_alias() {
        // Open Schema with `Strict` typing mode and `age` in nested attribute.
        let details_fields = struct_fields![("age", int!())];
        let details = r#struct![IndexSet::from([details_fields])];

        // TODO Revise this behavior once the following discussion is conclusive and spec. is
        // in place: https://github.com/partiql/partiql-spec/discussions/65
        assert_query_typing(
            TypingMode::Strict,
            "SELECT d.age FROM customers.details AS d",
            create_customer_schema(
                true,
                [
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("details", details.clone()),
                ]
                .into(),
            ),
            vec![StructField::new("age", int!())],
        )
        .expect("Type");

        // Closed Schema with Strict typing mode with FROM and Projection aliases.
        assert_query_typing(
            TypingMode::Strict,
            "SELECT c.id AS my_id, customers.name AS my_name FROM customers AS c",
            create_customer_schema(
                false,
                [
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("age", dynamic!()),
                ]
                .into(),
            ),
            vec![
                StructField::new("my_id", int!()),
                StructField::new("my_name", str!()),
            ],
        )
        .expect("Type");
    }

    #[test]
    fn simple_sfw_err() {
        // Closed Schema with `Strict` typing mode and `age` non-existent projection.
        let err1 = r#"No Typing Information for SymbolPrimitive { value: "age", case: CaseInsensitive } in closed Schema Static(StaticType { ty: Struct(StructType { constraints: {Fields({StructField { optional: false, name: "id", ty: Static(StaticType { ty: Int, nullable: true }) }, StructField { optional: false, name: "name", ty: Static(StaticType { ty: String, nullable: true }) }}), Open(false)} }), nullable: true })"#;

        assert_err(
            assert_query_typing(
                TypingMode::Strict,
                "SELECT customers.id, customers.name, customers.age FROM customers",
                create_customer_schema(
                    false,
                    [
                        StructField::new("id", int!()),
                        StructField::new("name", str!()),
                    ]
                    .into(),
                ),
                vec![],
            ),
            vec![TypingError::TypeCheck(err1.to_string())],
            Some(bag![r#struct![IndexSet::from([StructConstraint::Fields(
                [
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("age", undefined!()),
                ]
                .into()
            ),])]]),
        );

        // Closed Schema with `Strict` typing mode and `bar` non-existent projection from closed nested `details`.
        let details_fields = struct_fields![("age", int!())];
        let details = r#struct![IndexSet::from([
            details_fields,
            StructConstraint::Open(false)
        ])];

        let err1 = r#"No Typing Information for SymbolPrimitive { value: "details", case: CaseInsensitive } in closed Schema Static(StaticType { ty: Struct(StructType { constraints: {Fields({StructField { optional: false, name: "age", ty: Static(StaticType { ty: Int, nullable: true }) }}), Open(false)} }), nullable: true })"#;
        let err2 = r"Illegal Derive Type Undefined";

        assert_err(
            assert_query_typing(
                TypingMode::Strict,
                "SELECT customers.id, customers.name, customers.details.bar FROM customers",
                create_customer_schema(
                    false,
                    [
                        StructField::new("id", int!()),
                        StructField::new("name", str!()),
                        StructField::new("details", details),
                    ]
                    .into(),
                ),
                vec![],
            ),
            vec![
                TypingError::TypeCheck(err1.to_string()),
                TypingError::IllegalState(err2.to_string()),
            ],
            Some(bag![r#struct![IndexSet::from([StructConstraint::Fields(
                [
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("bar", undefined!()),
                ]
                .into()
            ),])]]),
        );
    }

    fn assert_err(
        result: Result<(), TypeErr>,
        expected_errors: Vec<TypingError>,
        output: Option<PartiqlShape>,
    ) {
        match result {
            Ok(()) => {
                panic!("Expected Error");
            }
            Err(e) => {
                assert_eq!(
                    e,
                    TypeErr {
                        errors: expected_errors,
                        output,
                    }
                );
            }
        };
    }

    fn create_customer_schema(is_open: bool, fields: IndexSet<StructField>) -> PartiqlShape {
        bag![r#struct![IndexSet::from([
            StructConstraint::Fields(fields),
            StructConstraint::Open(is_open)
        ])]]
    }

    fn assert_query_typing(
        mode: TypingMode,
        query: &str,
        schema: PartiqlShape,
        expected_fields: Vec<StructField>,
    ) -> Result<(), TypeErr> {
        let expected_fields: IndexSet<_> = expected_fields.into_iter().collect();
        let actual = type_query(mode, query, TypeEnvEntry::new("customers", &[], schema))?
            .expect_static()?;

        match &actual.ty() {
            Static::Bag(b) => {
                if let Ok(s) = b.element_type().expect_struct() {
                    let fields: IndexSet<_> = s.fields().collect();

                    let f: Vec<_> = expected_fields
                        .iter()
                        .filter(|f| !fields.contains(*f))
                        .collect();
                    assert!(f.is_empty());
                    assert_eq!(expected_fields.len(), fields.len());
                    println!("query: {query:?}");
                    println!("actual: {actual:?}");
                    Ok(())
                } else {
                    Err(TypeErr {
                        errors: vec![TypingError::TypeCheck("[Struct] type expected".to_string())],
                        output: None,
                    })
                }
            }
            _ => Err(TypeErr {
                errors: vec![TypingError::TypeCheck("[Bag] type expected".to_string())],
                output: None,
            }),
        }
    }

    fn type_query(
        mode: TypingMode,
        query: &str,
        type_env_entry: TypeEnvEntry<'_>,
    ) -> Result<PartiqlShape, TypeErr> {
        let mut catalog = PartiqlCatalog::default();
        let _oid = catalog.add_type_entry(type_env_entry);

        let parsed = parse(query);
        let lg = lower(&parsed, &catalog).expect("Logical plan");

        let mut typer = match mode {
            TypingMode::Permissive => PlanTyper::new_permissive(&catalog, &lg),
            TypingMode::Strict => PlanTyper::new_strict(&catalog, &lg),
        };

        typer.type_plan()
    }

    #[track_caller]
    fn parse(text: &str) -> Parsed<'_> {
        Parser::default().parse(text).unwrap()
    }

    #[track_caller]
    fn lower(
        parsed: &Parsed<'_>,
        catalog: &dyn Catalog,
    ) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
        let planner = LogicalPlanner::new(catalog);
        planner.lower(parsed)
    }
}
