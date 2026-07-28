#[cfg(test)]
mod tests {
    use kyc_core::ast::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse(source: &str) -> Result<Program, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
    // -----------------------------------------------------------------------
    // Declarations
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_program() {
        let p = parse("").unwrap();
        assert!(p.declarations.is_empty());
    }

    #[test]
    fn test_function_no_args_no_return() {
        let p = parse("fn main():\n    pass\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_function_with_args_and_return() {
        let p = parse("fn add(x: i32, y: i32) i32:\n    x + y\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_import_decl() {
        let p = parse("import math\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_from_import() {
        let source = "from math import sqrt\n";
        match parse(source) {
            Ok(p) => assert_eq!(p.declarations.len(), 1),
            Err(e) => panic!("from_import parse failed: {}", e),
        }
    }

    // -----------------------------------------------------------------------
    // Variables and constants
    // -----------------------------------------------------------------------

    #[test]
    fn test_variable_declaration() {
        let p = parse("name = 42\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_constant_declaration() {
        let p = parse("MAX_SIZE = 1024\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Classes and structs
    // -----------------------------------------------------------------------

    #[test]
    fn test_class_with_fields() {
        let source = "\
class User:
    name: str
    age: i32
";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_abstract_class() {
        let source = "\
abs class Animal:
    fn speak() str
";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_struct() {
        let source = "\
struct Point:
    x: f64
    y: f64
";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_enum() {
        let source = "\
enum Color:
    Red
    Green
    Blue
";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_contract() {
        let source = "\
contract Drawable:
    fn draw()
";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_type_alias() {
        let p = parse("type Age = i32\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Statements
    // -----------------------------------------------------------------------

    #[test]
    fn test_if_statement() {
        let source = "\
fn test():\n\
    if x > 0:\n\
        print(\"pos\")\n";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
        // Verify the function body exists
        if let Decl::Function(f) = &p.declarations[0] {
            assert!(f.body.is_some());
        } else {
            panic!("expected function declaration");
        }
    }

    #[test]
    fn test_if_elif_else() {
        let source = "fn test():\n    if x > 0:\n        print(\"pos\")\n    elif x < 0:\n        print(\"neg\")\n    else:\n        print(\"zero\")\n";
        match parse(source) {
            Ok(_) => {},
            Err(e) => panic!("if_elif_else parse failed: {}", e),
        }
    }

    #[test]
    fn test_while_loop() {
        let source = "\
fn test():\n\
    while running:\n\
        process()\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_destructure_assign() {
        let source = "fn test():\n    (x, y) = (1, 2)\n    print(x)\n";
        let result = parse(source);
        match &result {
            Ok(_) => {},
            Err(e) => panic!("destructure assign parse failed: {}", e),
        }
    }

    #[test]
    fn test_destructure_mut() {
        let source = "fn test():\n    (x, y) := (1, 2)\n";
        let result = parse(source);
        match &result {
            Ok(_) => {},
            Err(e) => panic!("destructure mut parse failed: {}", e),
        }
    }

    #[test]
    fn test_for_loop() {
        let source = "\
fn test():\n\
    for item in items:\n\
        print(item)\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_for_with_index() {
        let source = "\
fn test():\n\
    items: ^{str} = {\"a\"}\n\
    for i, item in items:\n\
        print(i.to_str() + \": \" + item)\n";
        // Just check parse succeeds - any deeper inspection requires body unwrap
        parse(source).expect("for with index parse failed");
    }

    #[test]
    fn test_match_statement() {
        let source = "fn test():\n    match value:\n        1:\n            print(\"one\")\n        2:\n            print(\"two\")\n";
        match parse(source) {
            Ok(_) => {},
            Err(e) => panic!("match parse failed: {}", e),
        }
    }

    #[test]
    fn test_return_with_value() {
        let source = "fn add(a: i32, b: i32) i32:\n    a + b\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_defer_statement() {
        let source = "\
fn test():\n\
    defer cleanup()\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_guard_statement() {
        let source = "fn test():\n    guard valid else:\n        return\n";
        match parse(source) {
            Ok(_) => {},
            Err(e) => panic!("guard parse failed: {}", e),
        }
    }

    #[test]
    fn test_unsafe_block() {
        let source = "\
fn test():\n\
    unsafe:\n\
        ptr = addr\n";
        assert!(parse(source).is_ok());
    }

    // -----------------------------------------------------------------------
    // Expressions
    // -----------------------------------------------------------------------

    #[test]
    fn test_binary_expression() {
        let source = "\
fn test():\n\
    x = 1 + 2 * 3\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_function_call() {
        let source = "\
fn test():\n\
    print(\"hello\")\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_property_access() {
        let source = "\
fn test():\n\
    name = user.name\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_list_literal() {
        let source = "\
fn test():\n\
    items = [1, 2, 3]\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_dict_literal() {
        let source = "\
fn test():\n\
    config = {key: \"value\"}\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_dict_multi_entry() {
        let source = "\
fn test():\n\
    config = {name: \"Alice\", age: 30}\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_ternary_expression() {
        let source = "\
fn test():\n\
    result = x > 0 ? \"pos\" : \"neg\"\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_match_expression() {
        let source = "fn test():\n    result = match value:\n        1:\n            \"one\"\n        2:\n            \"two\"\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_or_pattern_match() {
        let source = "fn f():\n    match 1:\n        1 | 2:\n            0\n";
        match parse(source) {
            Ok(p) => assert!(p.declarations.len() >= 1, "expected 1+ decls, got {}", p.declarations.len()),
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_or_pattern_match_expr() {
        let source = "fn f():\n    result = match 1:\n        1 | 2:\n            0\n";
        match parse(source) {
            Ok(p) => assert!(p.declarations.len() >= 1, "expected 1+ decls, got {}", p.declarations.len()),
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_optional_chain() {
        let source = "\
fn test():\n\
    name = user?.name\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_spread_list() {
        let source = "\
fn test():\n\
    items = [...a, 4, 5]\n";
        assert!(parse(source).is_ok());
    }

    // -----------------------------------------------------------------------
    // Error cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_error_bad_expression() {
        let result = parse("fn test():\n    +\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_type() {
        let result = parse("fn f() :\n    x\n");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_unexpected_declaration() {
        let result = parse("if x:\n    y\n");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Integration — large file smoke test
    // -----------------------------------------------------------------------

    #[test]
    fn test_large_file() {
        let source = r##"
## Large synthetic test — 200 functions to stress the parser
"##.to_string();
        let mut body = String::new();
        for i in 0..200 {
            body.push_str(&format!("fn fn_{}(x: i32) i32:\n    return x + {}\n\n", i, i));
        }
        body.push_str("fn main() i32:\n    return fn_0(0)\n");
        let src = format!("{}{}", source, body);
        if let Err(e) = parse(&src) {
            panic!("Large file parse error: {}", e);
        }
    }
}
