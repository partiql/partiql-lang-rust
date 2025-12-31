use crate::batch::{LogicalType, SourceTypeDef};
use crate::error::PlanError;
use crate::expr::{CompiledExpr, ConstantValue, ExprInput, ExprOp};
use partiql_logical::{BinaryOp, PathComponent, UnaryOp, ValueExpr};
use smallvec::smallvec;

/// Compiles PartiQL logical expressions into flat sequences of vectorized operations
pub struct ExpressionCompiler<'a> {
    /// Input schema for resolving column names to indices
    input_schema: &'a SourceTypeDef,
    /// Accumulated compiled expressions (output)
    compiled_exprs: Vec<CompiledExpr>,
    /// Type of each scratch register
    scratch_types: Vec<LogicalType>,
    /// Next available scratch register index
    next_scratch: usize,
}

impl<'a> ExpressionCompiler<'a> {
    /// Create a new expression compiler with the given input schema
    pub fn new(input_schema: &'a SourceTypeDef) -> Self {
        Self {
            input_schema,
            compiled_exprs: Vec::new(),
            scratch_types: Vec::new(),
            next_scratch: 0,
        }
    }

    /// Main entry point - compiles an expression tree into a flat list of operations
    ///
    /// Returns:
    /// - Vec<CompiledExpr>: The flat list of compiled expressions
    /// - Vec<LogicalType>: The type of each scratch register
    /// - usize: The index of the output register containing the final result
    pub fn compile(
        mut self,
        expr: &ValueExpr,
    ) -> Result<(Vec<CompiledExpr>, Vec<LogicalType>, usize), PlanError> {
        let (result_input, _result_type) = self.compile_expr(expr)?;

        // Determine the output register
        let output_register = match result_input {
            ExprInput::Scratch(idx) => idx,
            ExprInput::InputCol(idx) => {
                // Need to copy input column to scratch register
                let reg_idx = self.allocate_scratch(self.input_schema.get_field(idx)
                    .ok_or_else(|| PlanError::General(format!("Column index {} not found", idx)))?
                    .type_info);
                self.compiled_exprs.push(CompiledExpr {
                    op: ExprOp::Identity,
                    inputs: smallvec![ExprInput::InputCol(idx)],
                    output: reg_idx,
                });
                reg_idx
            }
            ExprInput::Constant(val) => {
                // Need to materialize constant to scratch register
                let ty = match &val {
                    ConstantValue::Int64(_) => LogicalType::Int64,
                    ConstantValue::Boolean(_) => LogicalType::Boolean,
                    _ => return Err(PlanError::General("Unsupported constant type".to_string())),
                };
                let reg_idx = self.allocate_scratch(ty);
                self.compiled_exprs.push(CompiledExpr {
                    op: ExprOp::Identity,
                    inputs: smallvec![ExprInput::Constant(val)],
                    output: reg_idx,
                });
                reg_idx
            }
        };

        Ok((self.compiled_exprs, self.scratch_types, output_register))
    }

    /// Recursively compile an expression
    ///
    /// Returns the input source (InputCol/Scratch/Constant) and its type
    fn compile_expr(&mut self, expr: &ValueExpr) -> Result<(ExprInput, LogicalType), PlanError> {
        match expr {
            // Literals: materialize into scratch registers immediately
            ValueExpr::Lit(lit) => {
                let (constant, ty) = match lit.as_ref() {
                    partiql_logical::Lit::Int64(v) => {
                        (ConstantValue::Int64(*v), LogicalType::Int64)
                    }
                    partiql_logical::Lit::Bool(v) => {
                        (ConstantValue::Boolean(*v), LogicalType::Boolean)
                    }
                    _ => {
                        return Err(PlanError::General(format!(
                            "Unsupported literal type: {:?}",
                            lit
                        )))
                    }
                };
                Ok((ExprInput::Constant(constant), ty))
            }

            // VarRef: resolve to input column
            ValueExpr::VarRef(_name, _var_type) => {
                let col_idx = self.resolve_column(expr)?;
                let col_type = self.input_schema.get_field(col_idx)
                    .ok_or_else(|| PlanError::General(format!("Column {} not found", col_idx)))?
                    .type_info;
                Ok((ExprInput::InputCol(col_idx), col_type))
            }

            // Path: resolve to input column (extract attribute name)
            ValueExpr::Path(_base, _components) => {
                let col_idx = self.resolve_column(expr)?;
                let col_type = self.input_schema.get_field(col_idx)
                    .ok_or_else(|| PlanError::General(format!("Column {} not found", col_idx)))?
                    .type_info;
                Ok((ExprInput::InputCol(col_idx), col_type))
            }

            // DynamicLookup: try to resolve the first option
            ValueExpr::DynamicLookup(lookups) => {
                if let Some(first) = lookups.first() {
                    self.compile_expr(first)
                } else {
                    Err(PlanError::General(
                        "Empty DynamicLookup expression".to_string(),
                    ))
                }
            }

            // Binary expressions
            ValueExpr::BinaryExpr(op, left, right) => {
                // Recursively compile left and right
                let (left_input, left_type) = self.compile_expr(left)?;
                let (right_input, right_type) = self.compile_expr(right)?;

                // Map logical operator to physical operator
                let expr_op = self.map_binary_op(op, left_type, right_type)?;

                // Determine output type
                let output_type = self.get_binary_output_type(op, left_type, right_type)?;

                // Allocate scratch register for result
                let output_reg = self.allocate_scratch(output_type);

                // Add compiled expression
                self.compiled_exprs.push(CompiledExpr {
                    op: expr_op,
                    inputs: smallvec![left_input, right_input],
                    output: output_reg,
                });

                Ok((ExprInput::Scratch(output_reg), output_type))
            }

            // Unary expressions
            ValueExpr::UnExpr(op, inner) => {
                // Recursively compile inner expression
                let (inner_input, inner_type) = self.compile_expr(inner)?;

                // Map logical operator to physical operator
                let expr_op = self.map_unary_op(op, inner_type)?;

                // Determine output type (same as input for Pos/Neg, same for Not)
                let output_type = match op {
                    UnaryOp::Pos | UnaryOp::Neg => inner_type,
                    UnaryOp::Not => LogicalType::Boolean,
                };

                // Allocate scratch register for result
                let output_reg = self.allocate_scratch(output_type);

                // Add compiled expression
                self.compiled_exprs.push(CompiledExpr {
                    op: expr_op,
                    inputs: smallvec![inner_input],
                    output: output_reg,
                });

                Ok((ExprInput::Scratch(output_reg), output_type))
            }

            _ => Err(PlanError::General(format!(
                "Unsupported expression type: {:?}",
                expr
            ))),
        }
    }

    /// Resolve a column name from a VarRef or Path expression
    fn resolve_column(&self, expr: &ValueExpr) -> Result<usize, PlanError> {
        let attr_name = Self::extract_attribute_name(expr).ok_or_else(|| {
            PlanError::General(format!("Cannot extract attribute name from {:?}", expr))
        })?;

        self.input_schema.get_column_index(&attr_name)
    }

    /// Extract attribute name from VarRef or Path expression
    /// Examples:
    /// - VarRef("a") → "a"
    /// - Path(VarRef("v"), [Key("a")]) → "a"
    fn extract_attribute_name(expr: &ValueExpr) -> Option<String> {
        match expr {
            ValueExpr::VarRef(name, _) => Some(bindings_name_to_string(name)),

            ValueExpr::Path(_base, components) => {
                // Extract the first key component
                if let Some(PathComponent::Key(key_name)) = components.first() {
                    Some(bindings_name_to_string(key_name))
                } else {
                    None
                }
            }

            ValueExpr::DynamicLookup(lookups) => {
                // Try to extract from the first lookup option
                lookups.first().and_then(Self::extract_attribute_name)
            }

            _ => None,
        }
    }

    /// Allocate a new scratch register with the given type
    fn allocate_scratch(&mut self, ty: LogicalType) -> usize {
        let idx = self.next_scratch;
        self.scratch_types.push(ty);
        self.next_scratch += 1;
        idx
    }

    /// Map a logical binary operator to a physical expression operator
    fn map_binary_op(
        &self,
        op: &BinaryOp,
        left_type: LogicalType,
        right_type: LogicalType,
    ) -> Result<ExprOp, PlanError> {
        match (op, left_type, right_type) {
            // Arithmetic operations (i64 → i64)
            (BinaryOp::Add, LogicalType::Int64, LogicalType::Int64) => Ok(ExprOp::AddI64),
            (BinaryOp::Sub, LogicalType::Int64, LogicalType::Int64) => Ok(ExprOp::SubI64),
            (BinaryOp::Mul, LogicalType::Int64, LogicalType::Int64) => Ok(ExprOp::MulI64),
            (BinaryOp::Div, LogicalType::Int64, LogicalType::Int64) => Ok(ExprOp::DivI64),

            // Comparison operations (i64 → boolean)
            (BinaryOp::Lt, LogicalType::Int64, LogicalType::Int64) => Ok(ExprOp::LtI64),
            (BinaryOp::Gt, LogicalType::Int64, LogicalType::Int64) => Ok(ExprOp::GtI64),
            (BinaryOp::Eq, LogicalType::Int64, LogicalType::Int64) => Ok(ExprOp::EqI64),
            (BinaryOp::Neq, LogicalType::Int64, LogicalType::Int64) => Ok(ExprOp::NeI64),
            (BinaryOp::Gteq, LogicalType::Int64, LogicalType::Int64) => Ok(ExprOp::GeI64),
            (BinaryOp::Lteq, LogicalType::Int64, LogicalType::Int64) => Ok(ExprOp::LeI64),

            // Logical operations (boolean → boolean)
            (BinaryOp::And, LogicalType::Boolean, LogicalType::Boolean) => Ok(ExprOp::AndBool),
            (BinaryOp::Or, LogicalType::Boolean, LogicalType::Boolean) => Ok(ExprOp::OrBool),

            _ => Err(PlanError::General(format!(
                "Unsupported binary operation: {:?} with types {:?}, {:?}",
                op, left_type, right_type
            ))),
        }
    }

    /// Map a logical unary operator to a physical expression operator
    fn map_unary_op(&self, op: &UnaryOp, inner_type: LogicalType) -> Result<ExprOp, PlanError> {
        match (op, inner_type) {
            (UnaryOp::Not, LogicalType::Boolean) => Ok(ExprOp::NotBool),
            // Pos is identity, Neg would need a negate operation (not implemented yet)
            _ => Err(PlanError::General(format!(
                "Unsupported unary operation: {:?} with type {:?}",
                op, inner_type
            ))),
        }
    }

    /// Determine the output type of a binary operation
    fn get_binary_output_type(
        &self,
        op: &BinaryOp,
        left_type: LogicalType,
        right_type: LogicalType,
    ) -> Result<LogicalType, PlanError> {
        match op {
            // Arithmetic operations return the same type
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod
            | BinaryOp::Exp => {
                if left_type == right_type {
                    Ok(left_type)
                } else {
                    Err(PlanError::General(format!(
                        "Type mismatch in arithmetic: {:?} vs {:?}",
                        left_type, right_type
                    )))
                }
            }
            // Comparison operations return boolean
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Gteq
            | BinaryOp::Lteq => Ok(LogicalType::Boolean),
            // Logical operations return boolean
            BinaryOp::And | BinaryOp::Or => Ok(LogicalType::Boolean),
            _ => Err(PlanError::General(format!(
                "Unsupported binary operation: {:?}",
                op
            ))),
        }
    }
}

/// Convert BindingsName to String
fn bindings_name_to_string(name: &partiql_value::BindingsName) -> String {
    match name {
        partiql_value::BindingsName::CaseSensitive(s) => s.to_string(),
        partiql_value::BindingsName::CaseInsensitive(s) => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::Field;

    #[test]
    fn test_compile_literal() {
        let schema = SourceTypeDef::new(vec![]);
        let compiler = ExpressionCompiler::new(&schema);

        let expr = ValueExpr::Lit(Box::new(partiql_logical::Lit::Int64(42)));
        let (compiled, types, output) = compiler.compile(&expr).unwrap();

        // Should create one identity operation to materialize the constant
        assert_eq!(compiled.len(), 1);
        assert_eq!(types.len(), 1);
        assert_eq!(types[0], LogicalType::Int64);
        assert_eq!(output, 0);
    }

    #[test]
    fn test_compile_binary_expr() {
        let schema = SourceTypeDef::new(vec![
            Field {
                name: "a".to_string(),
                type_info: LogicalType::Int64,
            },
            Field {
                name: "b".to_string(),
                type_info: LogicalType::Int64,
            },
        ]);
        let compiler = ExpressionCompiler::new(&schema);

        // a + b
        let expr = ValueExpr::BinaryExpr(
            BinaryOp::Add,
            Box::new(ValueExpr::VarRef(
                partiql_value::BindingsName::CaseInsensitive("a".into()),
                partiql_logical::VarRefType::Local,
            )),
            Box::new(ValueExpr::VarRef(
                partiql_value::BindingsName::CaseInsensitive("b".into()),
                partiql_logical::VarRefType::Local,
            )),
        );

        let (compiled, types, output) = compiler.compile(&expr).unwrap();

        // Should create one AddI64 operation
        assert_eq!(compiled.len(), 1);
        assert_eq!(compiled[0].op, ExprOp::AddI64);
        assert_eq!(types.len(), 1);
        assert_eq!(types[0], LogicalType::Int64);
        assert_eq!(output, 0);
    }

    #[test]
    fn test_compile_nested_expr() {
        let schema = SourceTypeDef::new(vec![
            Field {
                name: "a".to_string(),
                type_info: LogicalType::Int64,
            },
            Field {
                name: "b".to_string(),
                type_info: LogicalType::Int64,
            },
        ]);
        let compiler = ExpressionCompiler::new(&schema);

        // a < b + 10
        let expr = ValueExpr::BinaryExpr(
            BinaryOp::Lt,
            Box::new(ValueExpr::VarRef(
                partiql_value::BindingsName::CaseInsensitive("a".into()),
                partiql_logical::VarRefType::Local,
            )),
            Box::new(ValueExpr::BinaryExpr(
                BinaryOp::Add,
                Box::new(ValueExpr::VarRef(
                    partiql_value::BindingsName::CaseInsensitive("b".into()),
                    partiql_logical::VarRefType::Local,
                )),
                Box::new(ValueExpr::Lit(Box::new(partiql_logical::Lit::Int64(10)))),
            )),
        );

        let (compiled, types, output) = compiler.compile(&expr).unwrap();

        // Should create two operations: AddI64 and LtI64
        assert_eq!(compiled.len(), 2);
        assert_eq!(compiled[0].op, ExprOp::AddI64);
        assert_eq!(compiled[1].op, ExprOp::LtI64);
        assert_eq!(types.len(), 2);
        assert_eq!(types[0], LogicalType::Int64); // b + 10 result
        assert_eq!(types[1], LogicalType::Boolean); // a < (b + 10) result
        assert_eq!(output, 1);
    }
}
