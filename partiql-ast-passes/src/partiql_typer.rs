use crate::error::{AstTransformError, AstTransformationError};
use partiql_ast::ast::{
    AstNode, AstTypeMap, Bag, Expr, List, Lit, NodeId, Query, QuerySet, Struct, TopLevelQuery,
};
use partiql_ast::visit::{Traverse, Visit, Visitor};
use partiql_catalog::Catalog;
use partiql_types::{ArrayType, BagType, PartiqlType, StructType, TypeKind};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AstPartiqlTyper<'c> {
    id_stack: Vec<NodeId>,
    container_stack: Vec<Vec<PartiqlType>>,
    errors: Vec<AstTransformError>,
    type_map: AstTypeMap<PartiqlType>,
    catalog: &'c dyn Catalog,
}

impl<'c> AstPartiqlTyper<'c> {
    pub fn new(catalog: &'c dyn Catalog) -> Self {
        AstPartiqlTyper {
            id_stack: Default::default(),
            container_stack: Default::default(),
            errors: Default::default(),
            type_map: Default::default(),
            catalog,
        }
    }

    pub fn type_nodes(
        mut self,
        query: &AstNode<TopLevelQuery>,
    ) -> Result<AstTypeMap<PartiqlType>, AstTransformationError> {
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

impl<'c, 'ast> Visitor<'ast> for AstPartiqlTyper<'c> {
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
            QuerySet::BagOp(_) => {
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
            Lit::Null => TypeKind::Null,
            Lit::Missing => TypeKind::Missing,
            Lit::Int8Lit(_) => TypeKind::Int,
            Lit::Int16Lit(_) => TypeKind::Int,
            Lit::Int32Lit(_) => TypeKind::Int,
            Lit::Int64Lit(_) => TypeKind::Int,
            Lit::DecimalLit(_) => TypeKind::Decimal,
            Lit::NumericLit(_) => TypeKind::Decimal,
            Lit::RealLit(_) => TypeKind::Float64,
            Lit::FloatLit(_) => TypeKind::Float64,
            Lit::DoubleLit(_) => TypeKind::Float64,
            Lit::BoolLit(_) => TypeKind::Bool,
            Lit::IonStringLit(_) => todo!(),
            Lit::CharStringLit(_) => TypeKind::String,
            Lit::NationalCharStringLit(_) => TypeKind::String,
            Lit::BitStringLit(_) => todo!(),
            Lit::HexStringLit(_) => todo!(),
            Lit::StructLit(_) => TypeKind::Struct(StructType::new_any()),
            Lit::ListLit(_) => TypeKind::Array(ArrayType::new_any()),
            Lit::BagLit(_) => TypeKind::Bag(BagType::new_any()),
            Lit::TypedLit(_, _) => todo!(),
        };

        let ty = PartiqlType::new(kind);
        let id = *self.current_node();
        if let Some(c) = self.container_stack.last_mut() {
            c.push(ty.clone());
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

        let ty = PartiqlType::new_struct(StructType::new_any());
        self.type_map.insert(id, ty.clone());

        if let Some(c) = self.container_stack.last_mut() {
            c.push(ty);
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
        let ty = PartiqlType::new_bag(BagType::new_any());

        self.type_map.insert(id, ty.clone());
        if let Some(s) = self.container_stack.last_mut() {
            s.push(ty);
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
        let ty = PartiqlType::new_array(ArrayType::new_any());

        self.type_map.insert(id, ty.clone());
        if let Some(s) = self.container_stack.last_mut() {
            s.push(ty);
        }
        Traverse::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use partiql_catalog::PartiqlCatalog;
    use partiql_types::{PartiqlType, TypeKind};

    #[test]
    fn simple_test() {
        assert_matches!(run_literal_test("NULL"), TypeKind::Null);
        assert_matches!(run_literal_test("MISSING"), TypeKind::Missing);
        assert_matches!(run_literal_test("Missing"), TypeKind::Missing);
        assert_matches!(run_literal_test("true"), TypeKind::Bool);
        assert_matches!(run_literal_test("false"), TypeKind::Bool);
        assert_matches!(run_literal_test("1"), TypeKind::Int);
        assert_matches!(run_literal_test("1.5"), TypeKind::Decimal);
        assert_matches!(run_literal_test("'hello world!'"), TypeKind::String);
        assert_matches!(run_literal_test("[1, 2 , {'a': 2}]"), TypeKind::Array(_));
        assert_matches!(run_literal_test("<<'1', {'a': 11}>>"), TypeKind::Bag(_));
        assert_matches!(
            run_literal_test("{'a': 1, 'b': 3, 'c': [1, 2]}"),
            TypeKind::Struct(_)
        );
    }

    #[test]
    fn simple_err_test() {
        assert!(type_statement("{'a': 1, a.b: 3}", &PartiqlCatalog::default()).is_err());
    }

    fn run_literal_test(q: &str) -> TypeKind {
        let out = type_statement(q, &PartiqlCatalog::default()).expect("type map");
        let values: Vec<&PartiqlType> = out.values().collect();
        values.last().unwrap().kind().clone()
    }

    fn type_statement(
        q: &str,
        catalog: &dyn Catalog,
    ) -> Result<AstTypeMap<PartiqlType>, AstTransformationError> {
        let parsed = partiql_parser::Parser::default()
            .parse(q)
            .expect("Expect successful parse");

        let typer = AstPartiqlTyper::new(catalog);
        let q = &parsed.ast;
        typer.type_nodes(q)
    }
}
