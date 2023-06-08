#![allow(dead_code)]
use serde::{Deserialize, Serialize};

// Expressions evaluate to something
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Expression {
    BinaryExpression(BinaryExpression),
    CallExpression(CallExpression),
    NumericLiteral(NumericLiteral),
    Identifier(Identifier),
}

// Statements do not evaluate to anything, they just mutate the interpreter
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Statement {
    FunctionDeclaration(FunctionDeclaration),
    ReturnStatement(ReturnStatement),
    ExpressionStatement(ExpressionStatement),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum FunctionArgumentPattern {
    Identifier(Identifier),
    ObjectPattern(ObjectPattern),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ObjectPattern {/* TODO */}

#[derive(Debug, Deserialize, Serialize)]
pub struct Identifier {
    value: String,
}

impl Identifier {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionParam {
    pat: FunctionArgumentPattern,
}

impl FunctionParam {
    pub fn new(pat: FunctionArgumentPattern) -> Self {
        Self { pat }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VariableDeclarator {
    id: Identifier,
    init: Option<Expression>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NumericLiteral {
    value: f64,
}

impl NumericLiteral {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExpressionStatement {
    expression: Box<Expression>,
}

impl ExpressionStatement {
    pub fn new(expr: Expression) -> Self {
        Self {
            expression: Box::new(expr),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BinaryExpression {
    lhs: Box<Expression>,
    rhs: Box<Expression>,
    operator: String,
}

impl BinaryExpression {
    pub fn new(lhs: Expression, rhs: Expression, operator: String) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            operator,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CallExpression {
    callee: Identifier,
    arguments: Vec<Expression>,
}

impl CallExpression {
    pub fn new(callee: String, arguments: Vec<Expression>) -> Self {
        Self {
            callee: Identifier::new(callee),
            arguments,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReturnStatement {
    argument: Box<Expression>,
}

impl ReturnStatement {
    pub fn new(expr: Expression) -> Self {
        Self {
            argument: Box::new(expr),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockStatement {
    statements: Vec<Statement>,
}

impl BlockStatement {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionDeclaration {
    identifier: Identifier,
    params: Vec<FunctionParam>,
    body: BlockStatement,
}

impl FunctionDeclaration {
    pub fn new(ident: String) -> Self {
        Self {
            identifier: Identifier::new(ident),
            params: Vec::new(),
            body: BlockStatement::new(),
        }
    }

    pub fn args_append(&mut self, argument: FunctionParam) {
        self.params.push(argument)
    }

    pub fn body_append(&mut self, stmt: Statement) {
        self.body.statements.push(stmt)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VariableDeclaration {
    declarations: Vec<VariableDeclarator>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Program {
    body: Vec<Statement>,
}

impl Program {
    pub fn new() -> Self {
        Self { body: Vec::new() }
    }

    pub fn append(&mut self, stmt: Statement) {
        self.body.push(stmt);
    }
}
