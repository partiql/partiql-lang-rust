use crate::batch::TypeInfo;
use crate::functions::VectorizedFn;
use std::collections::HashMap;

/// Binary operator types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    // Comparison
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Neq,
    // Logical
    And,
    Or,
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
}

/// Unary operator types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Not,
    Neg,
}

/// Operation type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpType {
    Binary(BinaryOp),
    Unary(UnaryOp),
}

/// Function key for lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FnKey {
    op_type: OpType,
    signature: Vec<TypeInfo>,
}

/// Registry of vectorized functions
pub struct VectorizedFnRegistry {
    functions: HashMap<FnKey, Box<dyn VectorizedFn>>,
}

impl VectorizedFnRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Create registry with all default functions
    pub fn default() -> Self {
        let mut registry = Self::new();
        // TODO: Register all default functions
        registry
    }

    /// Register a function
    pub fn register(
        &mut self,
        op: OpType,
        signature: Vec<TypeInfo>,
        func: Box<dyn VectorizedFn>,
    ) {
        let key = FnKey {
            op_type: op,
            signature,
        };
        self.functions.insert(key, func);
    }

    /// Resolve binary operator to vectorized function
    pub fn resolve_binary_op(
        &self,
        op: BinaryOp,
        lhs_type: TypeInfo,
        rhs_type: TypeInfo,
    ) -> Option<&dyn VectorizedFn> {
        let key = FnKey {
            op_type: OpType::Binary(op),
            signature: vec![lhs_type, rhs_type],
        };
        self.functions.get(&key).map(|b| b.as_ref())
    }

    /// Resolve unary operator to vectorized function
    pub fn resolve_unary_op(
        &self,
        op: UnaryOp,
        operand_type: TypeInfo,
    ) -> Option<&dyn VectorizedFn> {
        let key = FnKey {
            op_type: OpType::Unary(op),
            signature: vec![operand_type],
        };
        self.functions.get(&key).map(|b| b.as_ref())
    }
}

impl Default for VectorizedFnRegistry {
    fn default() -> Self {
        Self::default()
    }
}
