#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Node {
    Program(Program),
    BinaryExpression(BinaryExpression),
    CallExpression(CallExpression),
    NumericLiteral(NumericLiteral),
    Identifier(Identifier),
    FunctionDeclaration(FunctionDeclaration),
    ReturnStatement(ReturnStatement),
    ExpressionStatement(ExpressionStatement),
    Paramter(Parameter),
}

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
pub struct Parameter {
    pat: Box<Node>,
}

impl Parameter {
    pub fn new(pat: Node) -> Self {
        Self { pat: Box::new(pat) }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VariableDeclarator {
    id: Identifier,
    init: Option<Node>,
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
    expression: Box<Node>,
}

impl ExpressionStatement {
    pub fn new(expr: Node) -> Self {
        Self {
            expression: Box::new(expr),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BinaryExpression {
    lhs: Box<Node>,
    rhs: Box<Node>,
    operator: String,
}

impl BinaryExpression {
    pub fn new(lhs: Node, rhs: Node, operator: String) -> Self {
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
    arguments: Vec<Node>,
}

impl CallExpression {
    pub fn new(callee: String, arguments: Vec<Node>) -> Self {
        Self {
            callee: Identifier::new(callee),
            arguments,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReturnStatement {
    argument: Box<Node>,
}

impl ReturnStatement {
    pub fn new(expr: Node) -> Self {
        Self {
            argument: Box::new(expr),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct BlockStatement {
    statements: Vec<Node>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionDeclaration {
    identifier: Identifier,
    params: Vec<Parameter>,
    body: BlockStatement,
}

impl FunctionDeclaration {
    pub fn new(ident: String) -> Self {
        Self {
            identifier: Identifier::new(ident),
            params: Vec::new(),
            body: BlockStatement::default(),
        }
    }

    pub fn args_append(&mut self, argument: Parameter) {
        self.params.push(argument)
    }

    pub fn body_append(&mut self, stmt: Node) {
        self.body.statements.push(stmt)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VariableDeclaration {
    declarations: Vec<VariableDeclarator>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Program {
    body: Vec<Node>,
}

impl Program {
    pub fn append(&mut self, stmt: Node) {
        self.body.push(stmt);
    }
}
