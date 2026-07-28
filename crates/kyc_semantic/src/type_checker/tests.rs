#[cfg(test)]
mod tests {
    use kyc_core::ast::*;
    use kyc_core::diagnostic::{Diagnostic, ErrorCode};
    use crate::type_checker::TypeChecker;

    fn check(source: &str) -> Vec<Diagnostic> {
        let mut lexer = kyc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        let program = parser.parse().unwrap();
        let mut tc = TypeChecker::new();
        tc.check_program(&program);
        tc.reporter.diagnostics().to_vec()
    }

    fn check_has_error(source: &str, code: ErrorCode) -> bool {
        let diags = check(source);
        diags.iter().any(|d| d.code == code)
    }

    #[test]
    fn test_undefined_symbol() {
        let diags = check("fn main():\n    x = y\n");
        assert!(!diags.is_empty(), "Expected errors for undefined symbol");
    }

    #[test]
    fn test_valid_variable() {
        let diags = check("fn main():\n    x = 1\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    #[test]
    fn test_immutable_param_assign() {
        let diags = check("fn foo(x: i32):\n    x = 2\n");
        assert!(diags.iter().any(|d| d.code == ErrorCode::E0007),
            "Expected AssignToImmutable error, got: {:?}", diags);
    }

    #[test]
    fn test_mutable_variable() {
        let diags = check("fn main():\n    x: ^i32 = 1\n    x = 2\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    #[test]
    fn test_function_call() {
        let diags = check("fn foo(x: i32):\n    return x\nfn main():\n    foo(1)\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    #[test]
    fn test_undefined_in_expr() {
        let diags = check("fn main():\n    x = y + 1\n");
        assert!(diags.iter().any(|d| d.code == ErrorCode::E0009),
            "Expected UndefinedSymbol error, got: {:?}", diags);
    }

    #[test]
    fn test_mutable_var_assign_in_fn() {
        let diags = check("fn foo():\n    x: ^i32 = 1\n    x = 2\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    #[test]
    fn test_if_statement_valid() {
        let diags = check("fn main():\n    if true:\n        x = 1\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    #[test]
    fn test_import_no_errors() {
        let diags = check("import console\nfn main():\n    console.print(1)\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    // -----------------------------------------------------------------------
    // While / Loop
    // -----------------------------------------------------------------------

    #[test]
    fn test_while_statement_nested_assign() {
        let diags = check("fn main():\n    x: ^i32 = 1\n    while x < 10:\n        x = x + 1\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    #[test]
    fn test_assign_inside_while() {
        let diags = check("fn main():\n    x: ^i32 = 1\n    while true:\n        x = 2\n        break\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    #[test]
    fn test_while_condition_variable_ref() {
        let diags = check("fn main():\n    x: ^i32 = 1\n    while x < 5:\n        x = x + 1\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    #[test]
    fn test_while_body_var_read() {
        let diags = check("fn main():\n    x: ^i32 = 10\n    while x > 0:\n        print(x)\n        x = x - 1\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    #[test]
    fn test_while_body_expr_read() {
        let diags = check("fn main():\n    x: ^i32 = 0\n    while x < 5:\n        y = x + 1\n        x = y\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    // -----------------------------------------------------------------------
    // Const
    // -----------------------------------------------------------------------

    #[test]
    fn test_const_fn_no_errors() {
        let diags = check("const fn add(a: i32, b: i32) i32:\n    return a + b\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    // -----------------------------------------------------------------------
    // Ternary
    // -----------------------------------------------------------------------

    #[test]
    fn test_ternary_type_unification() {
        let diags = check("fn main():\n    x = true ? 1 : 2\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }

    // -----------------------------------------------------------------------
    // Dict
    // -----------------------------------------------------------------------

    #[test]
    fn test_dict_type_inference() {
        let diags = check("fn main():\n    d = {\"a\": 1, \"b\": 2}\n");
        assert!(diags.is_empty(), "Unexpected errors: {:?}", diags);
    }
}
