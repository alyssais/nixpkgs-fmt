//! This module contains specific `super::dsl` rules for formatting nix language.
use rnix::{
    types::{Operation, SetEntry, TypedNode, With},
    SyntaxElement, SyntaxKind,
    SyntaxKind::*,
    T,
};

use crate::{
    dsl::{self, IndentDsl, IndentValue::*, SpacingDsl},
    pattern::Pattern,
    tree_utils::{has_newline, prev_sibling},
};

#[rustfmt::skip]
pub(crate) fn spacing() -> SpacingDsl {
    let mut dsl = SpacingDsl::default();

    dsl
        .test("{ a=92; }", "{ a = 92; }")
        .inside(NODE_SET_ENTRY).before(T![=]).single_space()
        .inside(NODE_SET_ENTRY).after(T![=]).single_space_or_optional_newline()

        .test("{ a = 92 ; }", "{ a = 92; }")
        .inside(NODE_SET_ENTRY).before(T![;]).no_space_or_optional_newline()
        .inside(NODE_SET_ENTRY).before(T![;]).when(after_literal).no_space()
        .inside(NODE_SET_ENTRY).before(T![;]).when(after_multiline_binop).single_space_or_newline()

        .test("a==  b", "a == b")
        .test("a!=  b", "a != b")
        .test("a++  b", "a ++ b")
        .test("a+  b", "a + b")
        .test("a  -   b", "a - b")
        .test("a*  b", "a * b")
        .test("a/  b", "a / b")
        .inside(NODE_OPERATION).after(BIN_OPS).single_space()
        .inside(NODE_OPERATION).before(BIN_OPS).single_space_or_newline()

        .test("foo . bar . baz", "foo.bar.baz")
        .inside(NODE_INDEX_SET).around(T![.]).no_space()

        .test("{} :92", "{}: 92")
        .inside(NODE_LAMBDA).before(T![:]).no_space()
        .inside(NODE_LAMBDA).after(T![:]).single_space_or_optional_newline()

        .test("[1 2 3]", "[ 1 2 3 ]")
        .inside(NODE_LIST).after(T!["["]).single_space_or_newline()
        .inside(NODE_LIST).before(T!["]"]).single_space_or_newline()
        .test("[ ]", "[]")
        .inside(NODE_LIST).between(T!["["], T!["]"]).no_space()
        .inside(NODE_LIST).between(VALUES, VALUES).single_space_or_newline()
        .inside(NODE_LIST).between(VALUES, TOKEN_COMMENT).single_space_or_optional_newline()

        .test("( 92 )", "(92)")
        .inside(NODE_PAREN).after(T!["("]).no_space_or_newline()
        .inside(NODE_PAREN).before(T![")"]).no_space_or_newline()

        .test("{foo = 92;}", "{ foo = 92; }")
        .inside(NODE_SET).after(T!["{"]).single_space_or_newline()
        .inside(NODE_SET).before(T!["}"]).single_space_or_newline()
        .test("{ }", "{}")
        .inside(NODE_SET).between(T!["{"], T!["}"]).no_space()
        .inside(NODE_SET).between(NODE_SET_ENTRY, NODE_SET_ENTRY).single_space_or_newline()
        .inside(NODE_SET).between(NODE_SET_ENTRY, TOKEN_COMMENT).single_space_or_optional_newline()

        .test("{arg}: 92", "{ arg }: 92")
        .inside(NODE_PATTERN).after(T!["{"]).single_space_or_newline()
        .inside(NODE_PATTERN).before(T!["}"]).single_space_or_newline()
        .test("{ }: 92", "{}: 92")
        .inside(NODE_PATTERN).between(T!["{"], T!["}"]).no_space()

        .test("{ foo,bar }: 92", "{ foo, bar }: 92")
        .inside(NODE_PATTERN).after(T![,]).single_space()
        .inside(NODE_PATTERN).before(T![,]).no_space_or_newline()

        .test("{ inherit( x )  y  z  ; }", "{ inherit (x) y z; }")
        .inside(NODE_INHERIT).around(NODE_INHERIT_FROM).single_space_or_optional_newline()
        .inside(NODE_INHERIT).before(T![;]).no_space_or_newline()
        .inside(NODE_INHERIT).before(NODE_IDENT).single_space_or_optional_newline()
        .inside(NODE_INHERIT_FROM).after(T!["("]).no_space()
        .inside(NODE_INHERIT_FROM).before(T![")"]).no_space()

        .test("let   foo = bar;in  92", "let foo = bar; in 92")
        .inside(NODE_LET_IN).after(T![let]).single_space_or_newline()
        .inside(NODE_LET_IN).around(T![in]).single_space_or_newline()

        .test("{a?3}: a", "{ a ? 3 }: a")
        .inside(NODE_PAT_ENTRY).around(T![?]).single_space()

        .test("f  x", "f x")
        .inside(NODE_APPLY).between(VALUES, VALUES).single_space_or_optional_newline()

        // special-cased rules for leading and trailing whitespace
        .rule(dsl::SpacingRule {
            pattern: NODE_ROOT.into(),
            space: dsl::Space { loc: dsl::SpaceLoc::Before, value: dsl::SpaceValue::None }
        })
        .rule(dsl::SpacingRule {
            pattern: NODE_ROOT.into(),
            space: dsl::Space { loc: dsl::SpaceLoc::After, value: dsl::SpaceValue::Newline }
        })
        ;

    dsl
}

fn after_literal(node: &SyntaxElement) -> bool {
    let prev = prev_sibling(node);
    return if let Some(body) = prev.clone().and_then(With::cast).and_then(|w| w.body()) {
        is_literal(body.kind())
    } else {
        prev.map(|it| is_literal(it.kind())) == Some(true)
    };

    fn is_literal(kind: SyntaxKind) -> bool {
        kind == NODE_SET || kind == NODE_LIST
    }
}

fn after_multiline_binop(node: &SyntaxElement) -> bool {
    let prev = prev_sibling(node);
    return if let Some(op) = prev.and_then(Operation::cast) {
        has_newline(op.node())
    } else {
        false
    };
}

fn p(p: impl Into<Pattern>) -> Pattern {
    p.into()
}

#[rustfmt::skip]
pub(crate) fn indentation() -> IndentDsl {
    let mut dsl = IndentDsl::default();
    dsl
        .anchor(NODE_PAT_ENTRY)
        .anchor(Pattern::from(rhs_of_binop))

        .rule("Indent list content")
            .inside(NODE_LIST)
            .not_matching([T!["["], T!["]"]])
            .set(Indent)
            .test(r#"
                [
                92
                ]
            "#, r#"
                [
                  92
                ]
            "#)

        .rule("Indent parenthesized expressions")
            .inside(NODE_PAREN)
            .not_matching([T!["("], T![")"]])
            .set(Indent)
            .test(r#"
                (
                92
                )
            "#, r#"
                (
                  92
                )
            "#)

        .rule("Indent attribute set content")
            .inside(NODE_SET)
            .not_matching([T!["{"], T!["}"]])
            .set(Indent)
            .test(r#"
                {
                foo = bar;
                }
            "#, r#"
                {
                  foo = bar;
                }
            "#)

        .rule("Indent let bindings")
            .inside(NODE_LET_IN)
            .not_matching([T![let], T![in]])
            .set(Indent)
            .test(r#"
                let
                x = 1;
                inherit z;
                  in
                  x
            "#, r#"
                let
                  x = 1;
                  inherit z;
                in
                  x
            "#)

        .rule("Indent attribute value")
            .inside(NODE_SET_ENTRY)
            .not_matching(T![;])
            .set(Indent)
            .test(r#"
                {
                  foo =
                  92;
                }
            "#, r#"
                {
                  foo =
                    92;
                }
            "#)

        .rule("Indent semicolon in attribute")
            .inside(NODE_SET_ENTRY)
            .when_anchor(set_entry_with_single_line_value)
            .matching(T![;])
            .set(Indent)
            .test(r#"
                {
                  foo = 92
                  ;

                  bar = [
                    1
                  ]
                  ++ [ 2 ]
                    ;
                }
            "#, r#"
                {
                  foo = 92
                    ;

                  bar = [
                    1
                  ]
                  ++ [ 2 ]
                  ;
                }
            "#)

        .rule("Indent concatenation to first element")
            .inside(NODE_OPERATION)
            .when_anchor(set_entry_with_single_line_value)
            .matching(BIN_OPS)
            .set(Indent)
            .test(r#"
                {
                  foo = []
                  ++ []
                  ;
                }
            "#, r#"
                {
                  foo = []
                    ++ []
                    ;
                }
            "#)

        .rule("Indent lambda parameters")
            .inside(NODE_PATTERN)
            .not_matching([T!["{"], T!["}"], T![,]])
            .set(Indent)
            .test(r#"
                {
                # comment
                foo ? bar
                , baz
                }: foo
            "#, r#"
                {
                  # comment
                  foo ? bar
                , baz
                }: foo
            "#)

        .rule("Indent lambda body")
            .inside(p(NODE_LAMBDA) & p(not_on_top_level))
            .not_matching([NODE_LAMBDA, TOKEN_COMMENT])
            .set(Indent)
            .test(r#"
                {}:
                  {
                foo =
                  # describe bar
                  bar:
                  # describe baz
                  baz:
                  fnbody;
                }
            "#, r#"
                {}:
                {
                  foo =
                    # describe bar
                    bar:
                    # describe baz
                    baz:
                      fnbody;
                }
            "#)

        .rule("Indent apply arg")
            .inside(NODE_APPLY)
            .set(Indent)
            .test(r#"
                foo
                bar baz
            "#, r#"
                foo
                  bar baz
            "#)

        .rule("Indent with body in attribute")
            .inside(NODE_WITH)
            .when_anchor(set_entry_with_single_line_value)
            .set(Indent)
            .test(r#"
                with foo;
                  {
                  bar = with baz;
                  body;
                  }
            "#, r#"
                with foo;
                {
                  bar = with baz;
                    body;
                }
            "#)

        .rule("Indent or default")
            .inside(NODE_OR_DEFAULT)
            .set(Indent)
            .test(r#"
                {
                  x = foo or
                  bar;
                }
            "#, r#"
                {
                  x = foo or
                    bar;
                }
            "#)

        .rule("Indent if-then-else")
            .inside(NODE_IF_ELSE)
            .not_matching([T![if], T![then], T![else]])
            .set(Indent)
            .test(r#"
                if
                foo
                then
                bar
                else
                baz
            "#, r#"
                if
                  foo
                then
                  bar
                else
                  baz
            "#)

        .rule("Indent inherit parts")
            .inside(NODE_INHERIT)
            .set(Indent)
            .test(r#"
                {
                  inherit
                  (builtins)
                  # comment
                  toString
                  ;
                }
            "#, r#"
                {
                  inherit
                    (builtins)
                    # comment
                    toString
                    ;
                }
            "#)
    ;

    dsl
}

fn not_on_top_level(element: &SyntaxElement) -> bool {
    !on_top_level(element)
}

fn on_top_level(element: &SyntaxElement) -> bool {
    let parent = match element.parent() {
        None => return true,
        Some(it) => it,
    };
    match parent.kind() {
        NODE_ROOT => true,
        NODE_LAMBDA => on_top_level(&parent.into()),
        _ => false,
    }
}

fn set_entry_with_single_line_value(entry: &SyntaxElement) -> bool {
    fn find(entry: SyntaxElement) -> Option<bool> {
        let entry = entry.into_node().and_then(SetEntry::cast)?;
        let mut value = entry.value()?;
        if Operation::cast(value.clone()).is_none() {
            return Some(true);
        }
        while let Some(op) = Operation::cast(value.clone()) {
            value = op.value1()?
        }
        Some(!has_newline(&value))
    }
    find(entry.clone()) == Some(true)
}

fn rhs_of_binop(rhs: &SyntaxElement) -> bool {
    fn find(rhs: &SyntaxElement) -> Option<bool> {
        let op = rhs.parent().and_then(Operation::cast)?;
        Some(&op.value2()? == rhs.as_node()?)
    }
    find(rhs) == Some(true)
}

static VALUES: &[SyntaxKind] = &[
    NODE_LAMBDA,
    NODE_IDENT,
    NODE_INDEX_SET,
    NODE_LET_IN,
    NODE_LIST,
    NODE_OPERATION,
    NODE_PAREN,
    NODE_SET,
    NODE_STRING,
    NODE_VALUE,
    NODE_APPLY,
    NODE_IF_ELSE,
];

static BIN_OPS: &[SyntaxKind] = &[
    T!["//"],
    T![++],
    T![+],
    T![-],
    T![*],
    T![/],
    T![==],
    T![=>],
    T![<],
    T![>],
    T![<=],
    T![!=],
    T![||],
    T![&&],
];

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use crate::{
        reformat_string,
        rules::{indentation, spacing},
    };

    #[test]
    fn smoke() {
        TestCase {
            name: None,
            before: "{
foo = x:
92;
}"
            .into(),
            after: "{
  foo = x:
    92;
}
"
            .into(),
        }
        .run()
        .map_err(|e| panic!(e))
        .unwrap();
    }

    /// For convenience, tests in this module are specified inline with a
    /// `.test` dsl methods, right next to the corresponding rule definition.
    /// This test extracts such test cases and checks them.
    #[test]
    fn test_inline_spacing_tests() {
        let rules = spacing();
        let tests: Vec<TestCase> = rules
            .tests
            .iter()
            .map(|&(before, after)| {
                let before = before.to_string();
                let after = format!("{}\n", after);
                TestCase::from_before_after(before, after)
            })
            .collect();
        run(&tests)
    }

    #[test]
    fn test_inline_indentation_tests() {
        let rules = indentation();
        let tests: Vec<TestCase> = rules
            .tests
            .iter()
            .map(|&(before, after)| {
                let before = unindent::unindent(before);
                let after = unindent::unindent(after);
                TestCase::from_before_after(before, after)
            })
            .collect();
        run(&tests)
    }

    #[test]
    fn test_bad_good_tests() {
        let test_data = {
            let dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(dir).join("test_data")
        };
        let tests = TestCase::collect_from_dir(&test_data);
        run(&tests);
    }

    #[test]
    fn test_syntax_errors_tests() {
        let test_data = {
            let dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(dir).join("test_data").join("syntax_errors")
        };
        let tests = TestCase::collect_from_dir(&test_data);
        run(&tests);
    }

    #[derive(Debug)]
    struct TestCase {
        name: Option<String>,
        before: String,
        after: String,
    }

    impl TestCase {
        fn from_before_after(before: String, after: String) -> TestCase {
            TestCase { name: None, before, after }
        }

        fn collect_from_dir(dir: &Path) -> Vec<TestCase> {
            let mut res = vec![];
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let file_name = entry.file_name();
                let before_name = file_name.to_str().unwrap();
                if before_name.ends_with(".bad.nix") {
                    let after_name = before_name.replace(".bad.", ".good.");
                    let test_case = TestCase {
                        name: Some(after_name.to_string()),
                        before: fs::read_to_string(dir.join(before_name)).unwrap(),
                        after: fs::read_to_string(dir.join(&after_name)).unwrap_or_else(|_err| {
                            panic!("{} not found", after_name);
                        }),
                    };
                    res.push(test_case);
                }
            }
            assert!(res.len() > 0);
            res
        }

        fn run(&self) -> Result<(), String> {
            let name = self.name.as_ref().map(|it| it.as_str()).unwrap_or("");
            let expected = &self.after;
            let actual = &reformat_string(&self.before);
            if expected != actual {
                return Err(format!(
                    "\n\nAssertion failed: wrong formatting\
                     \nTest: {}\n\
                     \nBefore:\n{}\n\
                     \nAfter:\n{}\n\
                     \nExpected:\n{}\n",
                    name, self.before, actual, self.after,
                ));
            }
            let second_round = &reformat_string(actual);
            if actual != second_round {
                return Err(format!(
                    "\n\nAssertion failed: formatting is not idempotent\
                     \nTest: {}\n\
                     \nBefore:\n{}\n\
                     \nAfter:\n{}\n",
                    name, actual, second_round,
                ));
            }
            Ok(())
        }
    }

    fn run(tests: &[TestCase]) {
        let mut n_failed = 0;
        for test in tests {
            if let Err(msg) = test.run() {
                n_failed += 1;
                eprintln!("{}", msg)
            }
        }
        if n_failed > 0 {
            panic!("{} failed test cases out of {} total", n_failed, tests.len())
        }
    }
}
