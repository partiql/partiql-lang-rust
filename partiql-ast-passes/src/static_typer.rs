use crate::error::{AstTransformError, AstTransformationError};
use partiql_ast::ast::{
    AstNode, AstTypeMap, Bag, Expr, List, Lit, NodeId, Query, QuerySet, Struct,
};
use partiql_ast::visit::{Traverse, Visit, Visitor};
use partiql_catalog::Catalog;
use partiql_types::{ArrayType, BagType, StaticType, StaticTypeKind, StructType};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AstStaticTyper<'c> {
    id_stack: Vec<NodeId>,
    container_stack: Vec<Vec<StaticType>>,
    errors: Vec<AstTransformError>,
    type_map: AstTypeMap<StaticType>,
    catalog: &'c dyn Catalog,
}

impl<'c> AstStaticTyper<'c> {
    pub fn new(catalog: &'c dyn Catalog) -> Self {
        AstStaticTyper {
            id_stack: Default::default(),
            container_stack: Default::default(),
            errors: Default::default(),
            type_map: Default::default(),
            catalog,
        }
    }

    pub fn type_nodes(
        mut self,
        query: &AstNode<Query>,
    ) -> Result<AstTypeMap<StaticType>, AstTransformationError> {
        query.visit(&mut self);
        if self.errors.is_empty() {
            Ok(self.type_map)
        } else {
            Err(AstTransformationError {
                errors: self.errors,
            })
        }
    }

    #[inline]
    fn current_node(&self) -> &NodeId {
        self.id_stack.last().unwrap()
    }
}

impl<'c, 'ast> Visitor<'ast> for AstStaticTyper<'c> {
    fn enter_ast_node(&mut self, id: NodeId) -> Traverse {
        self.id_stack.push(id);
        Traverse::Continue
    }

    fn exit_ast_node(&mut self, id: NodeId) -> Traverse {
        assert_eq!(self.id_stack.pop(), Some(id));
        Traverse::Continue
    }

    fn enter_query(&mut self, _query: &'ast Query) -> Traverse {
        Traverse::Continue
    }

    fn exit_query(&mut self, _query: &'ast Query) -> Traverse {
        Traverse::Continue
    }

    fn enter_query_set(&mut self, _query_set: &'ast QuerySet) -> Traverse {
        match _query_set {
            QuerySet::SetOp(_) => {
                todo!()
            }
            QuerySet::Select(_) => {}
            QuerySet::Expr(_) => {}
            QuerySet::Values(_) => {
                todo!()
            }
            QuerySet::Table(_) => {
                todo!()
            }
        }
        Traverse::Continue
    }

    fn exit_query_set(&mut self, _query_set: &'ast QuerySet) -> Traverse {
        Traverse::Continue
    }

    fn enter_expr(&mut self, _expr: &'ast Expr) -> Traverse {
        Traverse::Continue
    }

    fn exit_expr(&mut self, _expr: &'ast Expr) -> Traverse {
        Traverse::Continue
    }

    fn enter_lit(&mut self, _lit: &'ast Lit) -> Traverse {
        // Currently we're assuming no-schema, hence typing to arbitrary sized scalars.
        // TODO type to the corresponding scalar with the introduction of schema
        let kind = match _lit {
            Lit::Null => StaticTypeKind::Null,
            Lit::Missing => StaticTypeKind::Missing,
            Lit::Int8Lit(_) => StaticTypeKind::Int,
            Lit::Int16Lit(_) => StaticTypeKind::Int,
            Lit::Int32Lit(_) => StaticTypeKind::Int,
            Lit::Int64Lit(_) => StaticTypeKind::Int,
            Lit::DecimalLit(_) => StaticTypeKind::Decimal,
            Lit::NumericLit(_) => StaticTypeKind::Decimal,
            Lit::RealLit(_) => StaticTypeKind::Float64,
            Lit::FloatLit(_) => StaticTypeKind::Float64,
            Lit::DoubleLit(_) => StaticTypeKind::Float64,
            Lit::BoolLit(_) => StaticTypeKind::Bool,
            Lit::IonStringLit(_) => todo!(),
            Lit::CharStringLit(_) => StaticTypeKind::String,
            Lit::NationalCharStringLit(_) => StaticTypeKind::String,
            Lit::BitStringLit(_) => todo!(),
            Lit::HexStringLit(_) => todo!(),
            Lit::StructLit(_) => StaticTypeKind::Struct(StructType::unconstrained()),
            Lit::ListLit(_) => StaticTypeKind::Array(ArrayType::array()),
            Lit::BagLit(_) => StaticTypeKind::Bag(BagType::bag()),
            Lit::TypedLit(_, _) => todo!(),
        };

        let ty = StaticType::new(kind);
        let id = *self.current_node();
        if let Some(c) = self.container_stack.last_mut() {
            c.push(ty.clone())
        }
        self.type_map.insert(id, ty);
        Traverse::Continue
    }

    fn enter_struct(&mut self, _struct: &'ast Struct) -> Traverse {
        self.container_stack.push(vec![]);
        Traverse::Continue
    }

    fn exit_struct(&mut self, _struct: &'ast Struct) -> Traverse {
        let id = *self.current_node();
        let fields = self.container_stack.pop();

        // Such type checking will very likely move to a common module
        // TODO move to a more appropriate place for re-use.
        if let Some(f) = fields {
            // We already fail during parsing if the struct has wrong number of key-value pairs, e.g.:
            // {'a', 1, 'b'}
            // However, adding this check here.
            let is_malformed = f.len() % 2 > 0;
            if is_malformed {
                self.errors.push(AstTransformError::IllegalState(
                    "Struct key-value pairs are malformed".to_string(),
                ));
            }

            let has_invalid_keys = f.chunks(2).map(|t| &t[0]).any(|t| !t.is_string());
            if has_invalid_keys || is_malformed {
                self.errors.push(AstTransformError::IllegalState(
                    "Struct keys can only resolve to `String` type".to_string(),
                ));
            }
        }

        let ty = StaticType::new_struct(StructType::unconstrained());
        self.type_map.insert(id, ty.clone());

        if let Some(c) = self.container_stack.last_mut() {
            c.push(ty)
        }

        Traverse::Continue
    }

    fn enter_bag(&mut self, _bag: &'ast Bag) -> Traverse {
        self.container_stack.push(vec![]);
        Traverse::Continue
    }

    fn exit_bag(&mut self, _bag: &'ast Bag) -> Traverse {
        // TODO add schema validation of BAG elements, e.g. for Schema Bag<Int> if there is at least
        // one element that isn't INT there is a type checking error.

        // TODO clarify if we need to record the internal types of bag literal or stick w/Schema?
        self.container_stack.pop();

        let id = *self.current_node();
        let ty = StaticType::new_bag(BagType::bag());

        self.type_map.insert(id, ty.clone());
        if let Some(s) = self.container_stack.last_mut() {
            s.push(ty)
        }
        Traverse::Continue
    }

    fn enter_list(&mut self, _list: &'ast List) -> Traverse {
        self.container_stack.push(vec![]);
        Traverse::Continue
    }

    fn exit_list(&mut self, _list: &'ast List) -> Traverse {
        // TODO clarify if we need to record the internal types of array literal or stick w/Schema?
        // one element that isn't INT there is a type checking error.

        // TODO clarify if we need to record the internal types of array literal or stick w/Schema?
        self.container_stack.pop();

        let id = *self.current_node();
        let ty = StaticType::new_array(ArrayType::array());

        self.type_map.insert(id, ty.clone());
        if let Some(s) = self.container_stack.last_mut() {
            s.push(ty)
        }
        Traverse::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use partiql_ast::ast;
    use partiql_catalog::PartiqlCatalog;
    use partiql_types::{StaticType, StaticTypeKind};

    #[test]
    fn simple_test() {
        assert_matches!(run_literal_test("NULL"), StaticTypeKind::Null);
        assert_matches!(run_literal_test("MISSING"), StaticTypeKind::Missing);
        assert_matches!(run_literal_test("Missing"), StaticTypeKind::Missing);
        assert_matches!(run_literal_test("true"), StaticTypeKind::Bool);
        assert_matches!(run_literal_test("false"), StaticTypeKind::Bool);
        assert_matches!(run_literal_test("1"), StaticTypeKind::Int);
        assert_matches!(run_literal_test("1.5"), StaticTypeKind::Decimal);
        assert_matches!(run_literal_test("'hello world!'"), StaticTypeKind::String);
        assert_matches!(
            run_literal_test("[1, 2 , {'a': 2}]"),
            StaticTypeKind::Array(_)
        );
        assert_matches!(
            run_literal_test("<<'1', {'a': 11}>>"),
            StaticTypeKind::Bag(_)
        );
        assert_matches!(
            run_literal_test("{'a': 1, 'b': 3, 'c': [1, 2]}"),
            StaticTypeKind::Struct(_)
        );
    }

    #[test]
    fn simple_err_test() {
        assert!(type_statement("{'a': 1, a.b: 3}").is_err());
    }

    fn run_literal_test(q: &str) -> StaticTypeKind {
        let out = type_statement(q).expect("type map");
        let values: Vec<&StaticType> = out.values().collect();
        values.last().unwrap().kind().clone()
    }

    fn type_statement(q: &str) -> Result<AstTypeMap<StaticType>, AstTransformationError> {
        let parsed = partiql_parser::Parser::default()
            .parse(q)
            .expect("Expect successful parse");

        let catalog = PartiqlCatalog::default();
        let typer = AstStaticTyper::new(&catalog);
        if let ast::Expr::Query(q) = parsed.ast.as_ref() {
            typer.type_nodes(&q)
        } else {
            panic!("Typing statement other than `Query` are unsupported")
        }
    }
}
