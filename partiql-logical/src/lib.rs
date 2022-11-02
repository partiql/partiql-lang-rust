use partiql_value::{BindingsName, Value};
use petgraph::prelude::StableGraph;
use petgraph::Directed;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LogicalPlan(pub StableGraph<BindingsExpr, (), Directed>);

#[derive(Debug)]
pub struct OpId(usize);

impl OpId {
    fn index(&self) -> usize {
        self.0
    }
}

impl LogicalPlan {
    pub fn new() -> Self {
        LogicalPlan(StableGraph::<BindingsExpr, (), Directed>::new())
    }
}

#[derive(Debug)]
pub struct LgPlan<'a, T> {
    nodes: Vec<T>,
    edges: Vec<Vec<&'a OpId>>,
    node_count: usize,
}

impl<'b, T> LgPlan<'b, T> {
    fn new() -> Self {
        LgPlan {
            nodes: vec![],
            edges: vec![],
            node_count: 0,
        }
    }

    fn add_operator(&mut self, op: T) -> OpId {
        self.nodes.push(op);
        self.edges.push(vec![]);
        self.node_count += 1;
        OpId(self.node_count)
    }
    fn add_relation<'a: 'b>(&mut self, src: &'a OpId, dst: &'a OpId) {
        let src_index = src.index() - 1;
        let dst_index = dst.index() - 1;
        self.nodes.get(src_index).expect("No such operator exists");
        self.nodes.get(dst_index).expect("No such operator exists");

        // let mut edge:&mut Vec<&OpId> = self.edges.get(src_index).expect("some").borrow_mut();
        self.edges[src_index].extend(vec![dst]);
    }
    fn operators(&self) -> &Vec<T> {
        &self.nodes
    }
    fn relations(&self) -> &Vec<Vec<&OpId>> {
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
    From(From),
    Scan(Scan),
    Unpivot,
    Where(Where),
    OrderBy,
    Offset,
    Limit,
    Join,
    SetOp,
    SelectValue(SelectValue),
    Select(Select),
    Project(Project),
    Distinct(Distinct),
    GroupBy,
    #[default]
    Output,
}

#[derive(Debug)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum BindingsToValueExpr {}

#[derive(Debug)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum ValueToBindingsExpr {}

/// [`From`] bridges from [`ValueExpr`]s to [`BindingExpr`]s
#[derive(Debug)]
pub struct From {
    pub expr: ValueExpr,
    pub as_key: String,
    pub at_key: Option<String>,
    pub out: Box<BindingsExpr>,
}

#[derive(Debug)]
pub struct Scan {
    pub expr: ValueExpr,
    pub as_key: String,
    pub at_key: Option<String>,
}

#[derive(Debug)]
pub struct Where {
    pub expr: ValueExpr,
    pub out: Box<BindingsExpr>,
}

#[derive(Debug)]
pub struct Select {
    pub exprs: HashMap<String, ValueExpr>,
    pub out: Box<BindingsExpr>,
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

#[derive(Debug)]
pub struct Distinct {
    pub out: Box<BindingsExpr>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan() {
        let mut p:LgPlan<BindingsExpr> = LgPlan::new();
        let a = p.add_operator(BindingsExpr::OrderBy);
        let b = p.add_operator(BindingsExpr::Output);
        let c = p.add_operator(BindingsExpr::Limit);
        p.add_relation(&a, &b);
        p.add_relation(&a, &c);
        p.add_relation(&b, &c);
        assert_eq!(3, p.operators().len());
        assert_eq!(3, p.relations().len());
    }
}
