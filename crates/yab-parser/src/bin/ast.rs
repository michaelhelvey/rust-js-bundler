use color_eyre::Result;
use yab_parser::ast;

fn main() -> Result<()> {
    /*
    // Represents the code:
    function foo(a) {
        return a + 1
    }

    foo(1);
    */
    let mut function = ast::FunctionDeclaration::new("foo".to_string());

    function.args_append(ast::Parameter::new(ast::Node::Identifier(
        ast::Identifier::new("a".to_string()),
    )));

    function.body_append(ast::Node::ReturnStatement(ast::ReturnStatement::new(
        ast::Node::BinaryExpression(ast::BinaryExpression::new(
            ast::Node::Identifier(ast::Identifier::new("a".to_string())),
            ast::Node::NumericLiteral(ast::NumericLiteral::new(1.0)),
            "+".to_string(),
        )),
    )));

    let mut program = ast::Program::default();
    program.append(ast::Node::FunctionDeclaration(function));
    program.append(ast::Node::ExpressionStatement(
        ast::ExpressionStatement::new(ast::Node::CallExpression(ast::CallExpression::new(
            "foo".to_string(),
            vec![ast::Node::NumericLiteral(ast::NumericLiteral::new(1.0))],
        ))),
    ));

    let program_node = ast::Node::Program(program);
    let pretty_program = serde_json::to_string_pretty(&program_node)?;
    println!("{}", &pretty_program);

    Ok(())
}
