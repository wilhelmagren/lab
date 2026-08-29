use std::sync::Arc;

use arrow::datatypes::{DataType, Field, FieldRef};

use crate::schema::SchemaRef;

pub type LogicalPlanRef = Arc<dyn LogicalPlan>;

/// A logical plan represents a data transformation or action that returns a relation.
pub trait LogicalPlan: std::fmt::Display {
    /// Returns the schema of the data that will be produced by this logical plan.
    fn schema(&self) -> SchemaRef;
    /// Returns the children (inputs) of this logical plan. This method enables use of
    /// the visitor pattern to walk a query tree easily.
    fn children(&self) -> Vec<Arc<dyn LogicalPlan>>;
    /// Format the logical plan in human readable form.
    fn format(&self, indent: usize) -> String {
        let mut s = String::new();
        (0..indent).for_each(|_| s.push_str("\t"));
        s.push_str(&self.to_string());
        s.push_str("\n");
        self.children()
            .into_iter()
            .for_each(|c| s.push_str(&c.format(indent + 1)));
        s
    }
}

/*
          Expression type                 Examples
        ----------------------       -------------------------
         Literal Value	            "hello", 12.34, true
         Column Reference       	user_id, first_name, salary
         Math Expression	        salary * 0.1, price + tax
         Comparison Expression	    age >= 21, status != 'inactive'
         Boolean Expression	        age >= 21 AND country = 'US'
         Aggregate Expression	    MIN(salary), MAX(salary), SUM(amount), COUNT(*)
         Scalar Function	        UPPER(name), CONCAT(first_name, ' ', last_name)
         Aliased Expression	        salary * 1.1 AS new_salary
*/

pub type LogicalExprRef = Arc<dyn LogicalExpr>;

/// Loigcal Expression for use in logical query plans. The logical expression provides
/// information needed during the planning phase such as the name and dtype of the exprs.
pub trait LogicalExpr: std::fmt::Display {
    /// Return metadata about the value that will be produced by the this expression
    /// when evaluated against a particular input.
    fn to_field(&self, input: LogicalPlanRef) -> FieldRef;
}

#[derive(Debug)]
struct ColumnExpr {
    name: String,
}

impl std::fmt::Display for ColumnExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", &self.name)
    }
}

impl LogicalExpr for ColumnExpr {
    fn to_field(&self, input: LogicalPlanRef) -> FieldRef {
        // clone is cheap on arc :)))
        input.schema().fields().find(&self.name).unwrap().1.clone()
    }
}

// TODO: make LiteralLong, LiteralBool, LiteralDouble

#[derive(Debug)]
struct LiteralStringExpr {
    value: String,
}

impl std::fmt::Display for LiteralStringExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", &self.value)
    }
}

impl LogicalExpr for LiteralStringExpr {
    fn to_field(&self, _input: LogicalPlanRef) -> FieldRef {
        // a literal value that is not a NULL literal can never be NULL :PP
        Arc::new(Field::new(&self.value, DataType::Utf8, false))
    }
}

pub enum BooleanOp {
    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    And,
    Or,
}

impl std::fmt::Display for BooleanOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BooleanOp::Eq => write!(f, "="),
            BooleanOp::Neq => write!(f, "!="),
            BooleanOp::Gt => write!(f, ">"),
            BooleanOp::GtEq => write!(f, ">="),
            BooleanOp::Lt => write!(f, "<"),
            BooleanOp::LtEq => write!(f, "<="),
            BooleanOp::And => write!(f, "AND"),
            BooleanOp::Or => write!(f, "OR"),
        }
    }
}

pub struct BooleanBinaryExpr {
    name: String,
    op: BooleanOp,
    l: LogicalExprRef,
    r: LogicalExprRef,
}

impl LogicalExpr for BooleanBinaryExpr {
    fn to_field(&self, input: LogicalPlanRef) -> FieldRef {
        let lf = self.l.to_field(input.clone());
        let rf = self.r.to_field(input);
        Arc::new(Field::new(
            &self.name,
            DataType::Boolean,
            lf.is_nullable() || rf.is_nullable(),
        ))
    }
}

impl std::fmt::Display for BooleanBinaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.l, self.op, self.r)
    }
}

pub fn eq(l: LogicalExprRef, r: LogicalExprRef) -> BooleanBinaryExpr {
    BooleanBinaryExpr {
        name: "eq".into(),
        op: BooleanOp::Eq,
        l,
        r,
    }
}

pub fn neq(l: LogicalExprRef, r: LogicalExprRef) -> BooleanBinaryExpr {
    BooleanBinaryExpr {
        name: "neq".into(),
        op: BooleanOp::Neq,
        l,
        r,
    }
}

// TODO: implement all boolean operators on booleanbinaryexpr

pub enum MathOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
}

impl std::fmt::Display for MathOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            MathOp::Add => "+",
            MathOp::Subtract => "-",
            MathOp::Multiply => "*",
            MathOp::Divide => "/",
            MathOp::Modulus => "%",
        };
        write!(f, "{}", symbol)
    }
}

pub struct MathExpr {
    name: String,
    op: MathOp,
    l: LogicalExprRef,
    r: LogicalExprRef,
}

impl LogicalExpr for MathExpr {
    fn to_field(&self, input: LogicalPlanRef) -> FieldRef {
        let lf = self.l.to_field(input);
        Arc::new(Field::new(
            &self.name,
            lf.data_type().clone(),
            lf.is_nullable(),
        ))
    }
}

impl std::fmt::Display for MathExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.l, self.op, self.r)
    }
}

pub fn add(l: LogicalExprRef, r: LogicalExprRef) -> MathExpr {
    MathExpr {
        name: "add".into(),
        op: MathOp::Add,
        l,
        r,
    }
}

pub fn subtract(l: LogicalExprRef, r: LogicalExprRef) -> MathExpr {
    MathExpr {
        name: "subtract".into(),
        op: MathOp::Subtract,
        l,
        r,
    }
}

pub fn multiplty(l: LogicalExprRef, r: LogicalExprRef) -> MathExpr {
    MathExpr {
        name: "multiplty".into(),
        op: MathOp::Multiply,
        l,
        r,
    }
}

pub fn divide(l: LogicalExprRef, r: LogicalExprRef) -> MathExpr {
    MathExpr {
        name: "divide".into(),
        op: MathOp::Divide,
        l,
        r,
    }
}

pub fn modulus(l: LogicalExprRef, r: LogicalExprRef) -> MathExpr {
    MathExpr {
        name: "modulus".into(),
        op: MathOp::Modulus,
        l,
        r,
    }
}
