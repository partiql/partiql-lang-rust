use partiql_ast::experimental::ast;

/// Convenience construct for [`ast::VarRef`]
// TODO remove in favor of improvements to partiql-ast
pub(crate) fn var_ref(ident: String, case_sensitive: bool, qualified: bool) -> ast::VarRef {
    let sensitivity = if case_sensitive {
        ast::CaseSensitivityKind::CaseSensitive
    } else {
        ast::CaseSensitivityKind::CaseInsensitive
    };
    let qualified = if qualified {
        ast::ScopeQualifierKind::Qualified
    } else {
        ast::ScopeQualifierKind::Unqualified
    };
    ast::VarRef {
        name: ast::SymbolPrimitive { value: ident },
        case: ast::CaseSensitivity { kind: sensitivity },
        qualifier: ast::ScopeQualifier { kind: qualified },
    }
}
