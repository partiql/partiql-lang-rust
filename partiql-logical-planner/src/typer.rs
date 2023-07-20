use indexmap::IndexMap;
use partiql_ast::ast::{CaseSensitivity, SymbolPrimitive};
use partiql_catalog::Catalog;
use partiql_logical::{BindingsOp, LogicalPlan, OpId, PathComponent, ValueExpr};
use partiql_types::{
    any, missing, unknown, ArrayType, BagType, PartiqlType, StructConstraint, StructField,
    StructType, TypeKind,
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
        LocalTypeEnv::from([($x, $y)])
    };
}

const OUTPUT_SCHEMA_KEY: &str = "_output_schema";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypeErr {
    pub errors: Vec<TypingError>,
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
}

#[derive(Debug, Clone)]
pub enum TypingMode {
    Permissive,
    Strict,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum LookupOrder {
    GlobalLocal,
    LocalGlobal,
}

/// Represents a type environment context
#[derive(Debug, Clone)]
struct TypeEnvContext {
    env: LocalTypeEnv,
    /// Represents the type that is used for creating the `env` in the [TypeEnvContext]
    derived_type: PartiqlType,
}

#[allow(dead_code)]
impl TypeEnvContext {
    fn new() -> Self {
        TypeEnvContext::default()
    }

    fn env(&self) -> &LocalTypeEnv {
        &self.env
    }

    fn derived_type(&self) -> &PartiqlType {
        &self.derived_type
    }
}

impl Default for TypeEnvContext {
    fn default() -> Self {
        TypeEnvContext {
            env: LocalTypeEnv::new(),
            derived_type: any!(),
        }
    }
}

impl From<(&LocalTypeEnv, &PartiqlType)> for TypeEnvContext {
    fn from(value: (&LocalTypeEnv, &PartiqlType)) -> Self {
        TypeEnvContext {
            env: value.0.clone(),
            derived_type: value.1.clone(),
        }
    }
}

/// Represents a Local Type Environment as opposed to the Global Type Environment in the Catalog.
type LocalTypeEnv = IndexMap<SymbolPrimitive, PartiqlType>;

#[derive(Debug, Clone)]
pub struct PlanTyper<'c> {
    typing_mode: TypingMode,
    catalog: &'c dyn Catalog,
    logical_plan: LogicalPlan<BindingsOp>,
    errors: Vec<TypingError>,
    type_env_stack: Vec<TypeEnvContext>,
    output: Option<PartiqlType>,
}

#[allow(dead_code)]
impl<'c> PlanTyper<'c> {
    /// Creates a new [PlanTyper] for the given Catalog and Intermediate Representation with `Strict` Typing Mode as the default Typing Mode.
    pub fn new(catalog: &'c dyn Catalog, ir: &LogicalPlan<BindingsOp>) -> Self {
        PlanTyper {
            typing_mode: TypingMode::Strict,
            catalog,
            logical_plan: ir.clone(),
            errors: Default::default(),
            type_env_stack: Default::default(),
            output: None,
        }
    }

    /// Creates a new [PlanTyper] for the given Catalog and Intermediate Representation with `Permissive` Typing Mode.
    pub fn new_permissive(catalog: &'c dyn Catalog, lg: &LogicalPlan<BindingsOp>) -> Self {
        PlanTyper {
            typing_mode: TypingMode::Permissive,
            catalog,
            logical_plan: lg.clone(),
            errors: Default::default(),
            type_env_stack: Default::default(),
            output: None,
        }
    }

    /// Returns the typing result for the Typer
    pub fn type_plan(&mut self) -> Result<PartiqlType, TypeErr> {
        let ops = self.sort()?;

        for idx in ops {
            let graph = self.to_stable_graph()?;
            if let Some(binop) = graph.node_weight(idx) {
                self.type_binop(binop)
            }
        }

        if self.errors.is_empty() {
            Ok(self.output.clone().unwrap_or(unknown!()))
        } else {
            Err(TypeErr {
                errors: self.errors.clone(),
            })
        }
    }

    fn type_binop(&mut self, op: &BindingsOp) {
        match op {
            BindingsOp::Scan(partiql_logical::Scan { expr, as_key, .. }) => {
                self.type_vexpr(expr, LookupOrder::GlobalLocal);
                let type_ctx = &self.local_type_env();
                for (name, ty) in type_ctx.env().iter() {
                    let derived_type = self.element_type(ty);
                    self.type_env_stack.push(ty_ctx![(
                        &ty_env![(name.clone(), derived_type.clone())],
                        ty
                    )]);

                    if !as_key.is_empty() {
                        self.type_env_stack.push(ty_ctx![(
                            &ty_env![(string_to_sym(as_key.as_str()), derived_type)],
                            ty
                        )]);
                    }
                }
            }
            BindingsOp::Project(partiql_logical::Project { exprs }) => {
                let mut fields = vec![];
                let derived_type_ctx = self.local_type_env();
                for (k, v) in exprs.iter() {
                    self.type_vexpr(v, LookupOrder::LocalGlobal);
                    if let Some(ty) = self.retrieve_type_from_local_ctx(k) {
                        fields.push(StructField::new(k.as_str(), ty.clone()));
                    }
                }

                let ty = PartiqlType::new_struct(StructType::new(BTreeSet::from([
                    StructConstraint::Fields(fields),
                ])));
                let derived_type = &self.derived_type(&derived_type_ctx);
                let schema = if derived_type.is_ordered_collection() {
                    PartiqlType::new_array(ArrayType::new(Box::new(ty)))
                } else if derived_type.is_unordered_collection() {
                    PartiqlType::new_bag(BagType::new(Box::new(ty)))
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
                if let Some(ty) = self.retrieve_type_from_local_ctx(OUTPUT_SCHEMA_KEY) {
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
        fn to_bindings(s: &StructType) -> Vec<(SymbolPrimitive, PartiqlType)> {
            s.fields()
                .into_iter()
                .map(|field| {
                    let sym = SymbolPrimitive {
                        value: field.name().to_string(),
                        case: CaseSensitivity::CaseInsensitive,
                    };

                    (sym, field.ty().clone())
                })
                .collect()
        }

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
                            let type_ctx = ty_ctx![(&ty_env![(sym, self.element_type(ty))], ty)];

                            self.type_env_stack.push(type_ctx);
                        } else {
                            self.errors.push(TypingError::NotYetImplemented(
                                "Local Lookup after unsuccessful Global lookup".to_string(),
                            ));
                        }
                    }
                    LookupOrder::LocalGlobal => {
                        for type_ctx in self.type_env_stack.clone().into_iter().rev() {
                            if let Some(ty) = type_ctx.env().get(&sym) {
                                let mut new_type_env = LocalTypeEnv::new();
                                if let TypeKind::Struct(s) = ty.kind() {
                                    // for field in s.fields() {
                                    //     let sym = SymbolPrimitive {
                                    //         value: field.name().to_string(),
                                    //         case: CaseSensitivity::CaseInsensitive,
                                    //     };
                                    //     new_type_env.insert(sym, field.ty().clone());
                                    // }
                                    to_bindings(&s).into_iter().for_each(|b| {
                                        new_type_env.insert(b.0, b.1);
                                    });
                                } else {
                                    new_type_env.insert(sym, ty.clone());
                                }

                                let derived_type = self.derived_type(&type_ctx);
                                let new_ty = self.element_type(&derived_type);
                                let type_ctx = ty_ctx![(&new_type_env, &new_ty)];
                                self.type_env_stack.push(type_ctx);
                                break;
                            }
                        }
                    }
                }
            }
            ValueExpr::Path(v, components) => {
                self.type_vexpr(v, LookupOrder::LocalGlobal);
                // let type_ctx = self.local_type_env();
                for component in components {
                    match component {
                        PathComponent::Key(key) => {
                            let var = ValueExpr::VarRef(key.clone());
                            self.type_vexpr(&var, LookupOrder::LocalGlobal);
                        }
                        PathComponent::Index(_) => {
                            self.errors.push(TypingError::NotYetImplemented(
                                "Typing [Index] [PathComponent]s".to_string(),
                            ))
                        }
                        PathComponent::KeyExpr(_) => {
                            self.errors.push(TypingError::NotYetImplemented(
                                "Typing [KeyExpr] [PathComponent]s".to_string(),
                            ))
                        }
                        PathComponent::IndexExpr(_) => {
                            self.errors.push(TypingError::NotYetImplemented(
                                "Typing [IndexExpr] [PathComponent]s".to_string(),
                            ))
                        }
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
                    _ => {
                        self.errors.push(TypingError::NotYetImplemented(
                            "Unsupported Literal".to_string(),
                        ));
                        todo!()
                    }
                };

                let ty = PartiqlType::new(kind);
                let new_type_env = IndexMap::from([(string_to_sym("_1"), ty.clone())]);
                self.type_env_stack.push(ty_ctx![(&new_type_env, &ty)]);
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
                    })
                }
            };

            let (s, d) = (add_node(s)?, add_node(d)?);
            graph.add_edge(s, d, *w);
        }

        Ok(graph)
    }

    fn element_type<'a>(&'a mut self, ty: &'a PartiqlType) -> PartiqlType {
        match ty.kind() {
            TypeKind::Bag(b) => b.element_type().clone(),
            TypeKind::Array(a) => a.element_type().clone(),
            TypeKind::Any => {
                self.errors
                    .push(TypingError::NotYetImplemented("[Any]".to_string()));
                unknown!()
            }
            TypeKind::AnyOf(_) => {
                self.errors
                    .push(TypingError::NotYetImplemented("[AnyOf]".to_string()));
                unknown!()
            }
            _ => ty.clone(),
        }
    }

    fn retrieve_type_from_local_ctx(&mut self, key: &str) -> Option<PartiqlType> {
        let type_ctx = self.local_type_env();
        let env = type_ctx.env().clone();
        let derived_type = self.derived_type(&type_ctx);

        if let Some(ty) = env.get(&string_to_sym(key)) {
            Some(ty.clone())
        } else if let TypeKind::Struct(s) = derived_type.kind() {
            if s.is_partial() {
                Some(any!())
            } else {
                match &self.typing_mode {
                    TypingMode::Permissive => Some(missing!()),
                    TypingMode::Strict => {
                        self.errors.push(TypingError::TypeCheck(format!(
                            "No Typing Information for {:?}",
                            &key
                        )));
                        None
                    }
                }
            }
        } else if derived_type.is_any() {
            Some(any!())
        } else {
            self.errors.push(TypingError::IllegalState(
                "Absent value found for derived_type".to_string(),
            ));
            None
        }
    }

    fn derived_type(&mut self, ty_ctx: &TypeEnvContext) -> PartiqlType {
        let ty = ty_ctx.derived_type();
        if let TypeKind::Unknown = ty.kind() {
            self.errors.push(TypingError::TypeCheck(format!(
                "A call to derived type resulted in [Unknown] type for [TypeContext]; [Unknown] type cannot be used for further type checking {:?}",
                ty_ctx
            )));
        }
        ty.clone()
    }

    fn local_type_env(&mut self) -> TypeEnvContext {
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
        // Closed schema with `Strict` typing mode.
        assert_query_typing(
            TypingMode::Strict,
            "SELECT customers.id, customers.name FROM customers",
            create_customer_schema(
                false,
                vec![
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("age", any!()),
                ],
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
                false,
                vec![
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("age", any!()),
                ],
            ),
            vec![
                StructField::new("id", int!()),
                StructField::new("name", str!()),
                StructField::new("age", any!()),
            ],
        )
        .expect("Type");
        // Closed Schema with `Permissive` typing mode and `age` non-existent projection.
        assert_query_typing(
            TypingMode::Permissive,
            "SELECT customers.id, customers.name, customers.age FROM customers",
            create_customer_schema(
                false,
                vec![
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                ],
            ),
            vec![
                StructField::new("id", int!()),
                StructField::new("name", str!()),
                StructField::new("age", missing!()),
            ],
        )
        .expect("Type");

        // Open Schema with `Strict` typing mode and `age` in nested attribute.
        let details_fields = struct_fields![("age", int!())];
        let details = r#struct![BTreeSet::from([details_fields])];

        assert_query_typing(
            TypingMode::Strict,
            "SELECT customers.id, customers.name, customers.details.age FROM customers",
            create_customer_schema(
                true,
                vec![
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("details", details.clone()),
                ],
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
            create_customer_schema(true,vec![
                StructField::new("id", int!()),
                StructField::new("name", str!()),
                StructField::new("details", details.clone()),
            ]),
            vec![
                StructField::new("id", int!()),
                StructField::new("name", str!()),
                StructField::new("age", int!()),
                StructField::new("bar", any!()),
            ],
        )
        .expect("Type");
    }

    #[test]
    fn simple_sfw_with_alias() {
        assert_query_typing(
            TypingMode::Strict,
            "SELECT c.id, customers.name FROM customers AS c",
            create_customer_schema(
                false,
                vec![
                    StructField::new("id", int!()),
                    StructField::new("name", str!()),
                    StructField::new("age", any!()),
                ],
            ),
            vec![
                StructField::new("id", int!()),
                StructField::new("name", str!()),
            ],
        )
        .expect("Type");
    }

    #[test]
    fn simple_sfw_err() {
        // Closed Schema with `Strict` typing mode and `age` non-existent projection.
        assert_err(
            assert_query_typing(
                TypingMode::Strict,
                "SELECT customers.id, customers.name, customers.age FROM customers",
                create_customer_schema(
                    false,
                    vec![
                        StructField::new("id", int!()),
                        StructField::new("name", str!()),
                    ],
                ),
                vec![],
            ),
            vec![TypingError::TypeCheck(format!(
                "No Typing Information for \"age\""
            ))],
        );

        // Closed Schema with `Strict` typing mode and `bar` non-existent projection from nested.
        let details_fields = struct_fields![("age", int!())];
        let details = r#struct![BTreeSet::from([details_fields])];
        assert_err(
            assert_query_typing(
                TypingMode::Strict,
                "SELECT customers.id, customers.name, customers.details.bar FROM customers",
                create_customer_schema(
                    false,
                    vec![
                        StructField::new("id", int!()),
                        StructField::new("name", str!()),
                        StructField::new("details", details),
                    ],
                ),
                vec![],
            ),
            vec![TypingError::TypeCheck(format!(
                "No Typing Information for \"bar\""
            ))],
        );
    }

    fn assert_err(result: Result<(), TypeErr>, expected_errors: Vec<TypingError>) {
        match result {
            Ok(_) => {
                panic!("Expected Error");
            }
            Err(e) => {
                assert_eq!(
                    e,
                    TypeErr {
                        errors: expected_errors,
                    }
                )
            }
        };
    }

    fn create_customer_schema(is_open: bool, fields: Vec<StructField>) -> PartiqlType {
        bag![r#struct![BTreeSet::from([
            StructConstraint::Fields(fields),
            StructConstraint::Open(is_open)
        ])]]
    }

    fn assert_query_typing(
        mode: TypingMode,
        query: &str,
        schema: PartiqlType,
        expected_fields: Vec<StructField>,
    ) -> Result<(), TypeErr> {
        let actual = type_query(mode, query, TypeEnvEntry::new("customers", &[], schema));

        match actual {
            Ok(actual) => match &actual.kind() {
                TypeKind::Bag(b) => {
                    if let TypeKind::Struct(s) = b.element_type().kind() {
                        let fields = s.fields();
                        let f: Vec<_> = expected_fields
                            .iter()
                            .filter(|f| !fields.contains(f))
                            .collect();
                        assert!(f.is_empty());
                        assert_eq!(expected_fields.len(), fields.len());
                        println!("query: {:?}", query);
                        println!("actual: {:?}", actual);
                        Ok(())
                    } else {
                        Err(TypeErr {
                            errors: vec![TypingError::TypeCheck(
                                "[Struct] type expected".to_string(),
                            )],
                        })
                    }
                }
                _ => Err(TypeErr {
                    errors: vec![TypingError::TypeCheck("[Bag] type expected".to_string())],
                }),
            },
            Err(e) => Err(e),
        }
    }

    fn type_query(
        mode: TypingMode,
        query: &str,
        type_env_entry: TypeEnvEntry,
    ) -> Result<PartiqlType, TypeErr> {
        let mut catalog = PartiqlCatalog::default();
        let _oid = catalog.add_type_entry(type_env_entry);

        let parsed = parse(query);
        let lg = lower(&parsed).expect("Logical plan");

        let mut typer = match mode {
            TypingMode::Permissive => PlanTyper::new_permissive(&catalog, &lg),
            TypingMode::Strict => PlanTyper::new(&catalog, &lg),
        };

        typer.type_plan()
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
