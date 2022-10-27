use partiql_value::{BindingsName, Value};
use petgraph::Graph;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LogicalPlan(pub Graph<BindingsExpr, ()>);

impl LogicalPlan {
    pub fn new() -> Self {
        LogicalPlan(Graph::<BindingsExpr, ()>::new())
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
