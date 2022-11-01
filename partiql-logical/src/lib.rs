use partiql_value::{BindingsName, Value};
use std::collections::HashMap;

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub struct OpId(usize);

impl OpId {
    pub fn index(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub struct LogicalPlan<T> {
    nodes: Vec<T>,
    edges: Vec<(OpId, OpId)>,
}

impl<T> LogicalPlan<T> {
    pub fn new() -> Self {
        LogicalPlan {
            nodes: vec![],
            edges: vec![],
        }
    }

    pub fn add_operator(&mut self, op: T) -> OpId {
        self.nodes.push(op);
        OpId(self.operator_count())
    }

    pub fn add_flow(&mut self, src: OpId, dst: OpId) {
        assert!(src.index() <= self.operator_count());
        assert!(dst.index() <= self.operator_count());

        self.edges.push((src, dst));
    }

    pub fn extend_with_flows(&mut self, flows: &[(OpId, OpId)]) {
        flows.iter().for_each(|f| {
            assert!(f.0.index() <= self.operator_count());
            assert!(f.1.index() <= self.operator_count());

            self.edges.push(*f);
        });
    }

    pub fn operator_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn operators(&self) -> &Vec<T> {
        &self.nodes
    }

    pub fn flows(&self) -> &Vec<(OpId, OpId)> {
        &self.edges
    }
}

// TODO we should replace this enum with some identifier that can be looked up in a symtab/funcregistry?
#[derive(Clone, Debug)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum UnaryOp {}

// TODO we should replace this enum with some identifier that can be looked up in a symtab/funcregistry?
#[derive(Clone, Debug)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum BinaryOp {
    And,
    Or,
    Concat,
    Eq,
    Neq,
    Gt,
    Gteq,
    Lt,
    Lteq,

    // Arithmetic ops
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
}

#[derive(Clone, Debug)]
pub enum PathComponent {
    Key(String),
    Index(i64),
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum ValueExpr {
    // TODO other variants
    UnExpr(UnaryOp, Box<ValueExpr>),
    BinaryExpr(BinaryOp, Box<ValueExpr>, Box<ValueExpr>),
    Lit(Box<Value>),
    Path(Box<ValueExpr>, Vec<PathComponent>),
    VarRef(BindingsName),
}

// Bindings -> Bindings : Where, OrderBy, Offset, Limit, Join, SetOp, Select, Distinct, GroupBy, Unpivot, Let
// Values   -> Bindings : From
// Bindings -> Values   : Select Value

#[derive(Debug, Default)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum BindingsExpr {
    Scan(Scan),
    Unpivot(Unpivot),
    Filter(Filter),
    OrderBy,
    Offset,
    Limit,
    Join,
    SetOp,
    SelectValue(SelectValue),
    Project(Project),
    Distinct,
    GroupBy,
    #[default]
    Sink,
}

#[derive(Debug)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum BindingsToValueExpr {}

#[derive(Debug)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum ValueToBindingsExpr {}

/// [`Scan`] bridges from [`ValueExpr`]s to [`BindingExpr`]s
#[derive(Debug)]
pub struct Scan {
    pub expr: ValueExpr,
    pub as_key: String,
    pub at_key: String,
}

/// [`Scan`] bridges from [`ValueExpr`]s to [`BindingExpr`]s
#[derive(Debug)]
pub struct Unpivot {
    pub expr: ValueExpr,
    pub as_key: String,
    pub at_key: Option<String>,
}

#[derive(Debug)]
pub struct Filter {
    pub expr: ValueExpr,
}

#[derive(Debug)]
pub struct Project {
    pub exprs: HashMap<String, ValueExpr>,
}

#[derive(Debug)]
pub struct SelectValue {
    pub exprs: ValueExpr,
    pub out: Box<ValueExpr>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan() {
        let mut p: LogicalPlan<BindingsExpr> = LogicalPlan::new();
        let a = p.add_operator(BindingsExpr::OrderBy);
        let b = p.add_operator(BindingsExpr::Sink);
        let c = p.add_operator(BindingsExpr::Limit);
        let d = p.add_operator(BindingsExpr::GroupBy);
        let e = p.add_operator(BindingsExpr::Join);
        p.add_flow(a, b);
        p.add_flow(a, c);
        p.add_flow(b, c);
        p.extend_with_flows(&[(c, d), (d, e)]);
        assert_eq!(5, p.operators().len());
        assert_eq!(5, p.flows().len());
    }
}
