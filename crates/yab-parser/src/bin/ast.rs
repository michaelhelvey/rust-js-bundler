use color_eyre::Result;
use yab_parser::ast::{self};

fn main() -> Result<()> {
    /*
    // Represents the code:
    function foo(a) {
        return a + 1
    }

    foo(1);
    */
    let mut function = ast::FunctionDeclaration::new("foo".to_string());
    function.args_append(ast::FunctionParam::new(
        ast::FunctionArgumentPattern::Identifier(ast::Identifier::new("a".to_string())),
    ));

    function.body_append(ast::Statement::ReturnStatement(ast::ReturnStatement::new(
        ast::Expression::BinaryExpression(ast::BinaryExpression::new(
            ast::Expression::Identifier(ast::Identifier::new("a".to_string())),
            ast::Expression::NumericLiteral(ast::NumericLiteral::new(1.0)),
            "+".to_string(),
        )),
    )));

    let mut program = ast::Program::new();
    program.append(ast::Statement::FunctionDeclaration(function));
    program.append(ast::Statement::ExpressionStatement(
        ast::ExpressionStatement::new(ast::Expression::CallExpression(ast::CallExpression::new(
            "foo".to_string(),
            vec![ast::Expression::NumericLiteral(ast::NumericLiteral::new(
                1.0,
            ))],
        ))),
    ));

    println!("{}", serde_json::to_string_pretty(&program)?);
    Ok(())
}
