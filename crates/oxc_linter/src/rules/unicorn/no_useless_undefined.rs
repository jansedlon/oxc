use itertools::Itertools;
use lazy_static::lazy_static;
use oxc_ast::{
    ast::{
        Argument, CallExpression, Expression, ReturnStatement, Statement, TSType, TSTypeAnnotation,
    },
    AstKind,
};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use regex::Regex;

use crate::{context::LintContext, rule::Rule};

static COMPARE_FUNCTION_NAMES: &'static [&str] = &[
    "is",
    "equal",
    "notEqual",
    "strictEqual",
    "notStrictEqual",
    "propertyVal",
    "notPropertyVal",
    "not",
    "include",
    "property",
    "toBe",
    "toHaveBeenCalledWith",
    "toContain",
    "toContainEqual",
    "toEqual",
    "same",
    "notSame",
    "strictSame",
    "strictNotSame",
];

#[derive(Debug, Clone)]
pub struct NoUselessUndefined {
    check_arguments: bool,
    check_arrow_function_body: bool,
}

impl Default for NoUselessUndefined {
    fn default() -> Self {
        Self { check_arguments: true, check_arrow_function_body: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoUselessUndefined,
    correctness, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
);

lazy_static! {
    static ref SET_REG: Regex = Regex::new(r"^set[A-Z]").expect("Failed to parse regex");
}

impl Rule for NoUselessUndefined {
    fn run_once<'a>(&self, ctx: &LintContext<'a>) {
        // println!("{:#?}", ctx.nodes().iter().collect_vec());

        for node in ctx.nodes().iter() {
            println!("{:#?}", node);
            match node.kind() {
                // `return undefined;`
                AstKind::Function(function) => {
                    let Some(body) = &function.body else {
                        continue;
                    };

                    for statement in &body.statements {
                        let Statement::ReturnStatement(return_statement) = statement else {
                            continue;
                        };

                        let Some(Expression::Identifier(identifier)) = &return_statement.argument
                        else {
                            continue;
                        };

                        if identifier.name == "undefined" {
                            match &function.return_type {
                                Some(_) => {
                                    continue;
                                }
                                None => {}
                            }
                        }

                        ctx.diagnostic(
                            OxcDiagnostic::warn("Dot not use useless `undefined`.")
                                .with_label(return_statement.span),
                        );
                    }
                    // let Some(Expression::Identifier(identifier)) = &return_statement.argument
                    // else {
                    //     return;
                    // };

                    // if identifier.name == "undefined" {
                    //     ctx.scopes();
                    //     identifier.
                    //     ctx.diagnostic(
                    //         OxcDiagnostic::warn("Dot not use useless `undefined`.")
                    //             .with_label(return_statement.span),
                    //     );
                    // }
                }

                // `yield undefined;`
                AstKind::YieldExpression(yield_expression) => {
                    let Some(Expression::Identifier(argument)) = &yield_expression.argument else {
                        return;
                    };

                    if argument.name != "undefined" {
                        return;
                    }

                    if yield_expression.delegate {
                        return;
                    }

                    ctx.diagnostic(
                        OxcDiagnostic::warn("Dot not use useless `undefined`.")
                            .with_label(yield_expression.span),
                    )
                }
                // `() => undefined`
                AstKind::ArrowFunctionExpression(arrow_function_expression) => {
                    if !self.check_arrow_function_body {
                        return;
                    }

                    for statement in &arrow_function_expression.body.statements {
                        match statement {
                            Statement::ReturnStatement(return_statement) => {
                                let Some(Expression::Identifier(argument)) =
                                    &return_statement.argument
                                else {
                                    continue;
                                };
                                if argument.name != "undefined" {
                                    continue;
                                }

                                match &arrow_function_expression.return_type {
                                    Some(_) => {
                                        continue;
                                    }
                                    None => {}
                                }

                                ctx.diagnostic(
                                    OxcDiagnostic::warn("Dot not use useless `undefined`.")
                                        .with_label(return_statement.span),
                                );
                            }
                            Statement::ExpressionStatement(expression_statement) => {
                                let Expression::Identifier(identifier_reference) =
                                    &expression_statement.expression
                                else {
                                    continue;
                                };

                                if identifier_reference.name != "undefined" {
                                    continue;
                                }
                                ctx.diagnostic(
                                    OxcDiagnostic::warn("Dot not use useless `undefined`.")
                                        .with_label(identifier_reference.span),
                                );
                            }
                            _ => {}
                        }
                    }
                }

                // `let foo = undefined` / `var foo = undefined`
                AstKind::VariableDeclaration(variable_declaration) => {
                    if variable_declaration.kind.is_const() {
                        return;
                    }

                    for declaration in &variable_declaration.declarations {
                        if declaration.kind.is_const() {
                            continue;
                        }

                        let Some(Expression::Identifier(identifier)) = &declaration.init else {
                            continue;
                        };

                        if identifier.name == "undefined" {
                            ctx.diagnostic(
                                OxcDiagnostic::warn("Dot not use useless `undefined`.")
                                    .with_label(identifier.span),
                            );
                        }
                    }
                }

                // `const { foo = undefined } = {};`
                AstKind::AssignmentPattern(assignment_pattern) => {
                    let Expression::Identifier(identifier) = &assignment_pattern.right else {
                        return;
                    };

                    if identifier.name == "undefined" {
                        ctx.diagnostic(
                            OxcDiagnostic::warn("Dot not use useless `undefined`.")
                                .with_label(identifier.span),
                        );
                    }
                }

                AstKind::CallExpression(call_expression) => {
                    if !self.check_arguments {
                        return;
                    }

                    if should_ignore(&call_expression.callee) {
                        return;
                    }

                    let argument_nodes = &call_expression.arguments;

                    if is_function_bind_call(&call_expression) && argument_nodes.len() != 1 {
                        return;
                    }

                    let mut undefined_arguments = vec![];

                    for argument in argument_nodes.iter().rev() {
                        if let Argument::Identifier(identifier) = argument {
                            if identifier.name == "undefined" {
                                undefined_arguments.insert(0, identifier);
                            } else {
                                break;
                            }
                        }
                    }

                    if undefined_arguments.len() == 0 {
                        return;
                    }

                    let first_undefined_argument = undefined_arguments.first();
                    let last_undefined_argument = undefined_arguments.last();

                    let span = LabeledSpan::new(
                        Some("Do not use useless `undefined`".to_string()),
                        first_undefined_argument.unwrap().span.start as usize,
                        last_undefined_argument.unwrap().span.end as usize,
                    );

                    ctx.diagnostic(
                        OxcDiagnostic::warn("Dot not use useless `undefined`.").with_label(span),
                    );
                }
                _ => {}
            }
        }
    }
}

fn is_function_bind_call(call_expression: &CallExpression<'_>) -> bool {
    if call_expression.optional {
        return false;
    }

    match &call_expression.callee {
        Expression::StaticMemberExpression(static_member_expression) => {
            if static_member_expression.property.name == "bind" {
                return true;
            }

            return false;
        }
        _ => return false,
    }
}

fn should_ignore(callee: &Expression) -> bool {
    let name = match callee {
        Expression::Identifier(identifier) => identifier.name.to_string(),
        Expression::StaticMemberExpression(static_member_expression) => {
            static_member_expression.property.name.to_string()
        }
        _ => return false,
    };

    return COMPARE_FUNCTION_NAMES.contains(&name.as_str())
        // `array.push(undefined)`
        || name == "push"
        // `array.unshift(undefined)`
        || name == "unshift"
        // `array.includes(undefined)`
        || name == "includes"

        // `set.add(undefined)`
        || name == "add"
        // `set.has(undefined)`
        || name == "has"

        // `map.set(foo, undefined)`
        || name == "set"

        // `React.createContext(undefined)`
        || name == "createContext"
        // `setState(undefined)`
        || SET_REG.is_match(name.as_str())

        // https://vuejs.org/api/reactivity-core.html#ref
        || name == "ref'";
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        // ("function foo() {return;}", None, None, None),
        // ("const foo = () => {};", None, None, None),
        // ("let foo;", None, None, None),
        // ("var foo;", None, None, None),
        // ("const foo = undefined;", None, None, None),
        // ("foo();", None, None, None),
        // ("foo(bar,);", None, None, None),
        // ("foo(undefined, bar);", None, None, None),
        // ("const {foo} = {};", None, None, None),
        // ("function foo({bar} = {}) {}", None, None, None),
        // ("function foo(bar) {}", None, None, None),
        // ("function* foo() {yield* undefined;}", None, None, None),
        // ("if (Object.is(foo, undefined)){}", None, None, None),
        // ("t.is(foo, undefined)", None, None, None),
        // ("assert.equal(foo, undefined, message)", None, None, None),
        // ("assert.notEqual(foo, undefined, message)", None, None, None),
        // ("assert.strictEqual(foo, undefined, message)", None, None, None),
        // ("assert.notStrictEqual(foo, undefined, message)", None, None, None),
        // (r#"assert.propertyVal(foo, "bar", undefined, message)"#, None, None, None),
        // (r#"assert.notPropertyVal(foo, "bar", undefined, message)"#, None, None, None),
        // ("expect(foo).not(undefined)", None, None, None),
        // (r#"expect(foo).to.have.property("bar", undefined)"#, None, None, None),
        // ("expect(foo).toBe(undefined)", None, None, None),
        // ("expect(foo).toContain(undefined)", None, None, None),
        // ("expect(foo).toContainEqual(undefined)", None, None, None),
        // ("expect(foo).toEqual(undefined)", None, None, None),
        // ("t.same(foo, undefined)", None, None, None),
        // ("t.notSame(foo, undefined)", None, None, None),
        // ("t.strictSame(foo, undefined)", None, None, None),
        // ("t.strictNotSame(foo, undefined)", None, None, None),
        // ("expect(someFunction).toHaveBeenCalledWith(1, 2, undefined);", None, None, None),
        // ("set.add(undefined);", None, None, None),
        // ("map.set(foo, undefined);", None, None, None),
        // ("array.push(foo, undefined);", None, None, None),
        // ("array.push(undefined);", None, None, None),
        // ("array.unshift(foo, undefined);", None, None, None),
        // ("array.unshift(undefined);", None, None, None),
        // ("createContext(undefined);", None, None, None),
        // ("React.createContext(undefined);", None, None, None),
        // ("setState(undefined)", None, None, None),
        // ("setState?.(undefined)", None, None, None),
        // ("props.setState(undefined)", None, None, None),
        // ("props.setState?.(undefined)", None, None, None),
        // ("array.includes(undefined)", None, None, None),
        // ("set.has(undefined)", None, None, None),
        // ("foo.bind(bar, undefined)", None, None, None),
        // ("foo.bind(...bar, undefined)", None, None, None),
        // ("foo.bind(...[], undefined)", None, None, None),
        // ("foo.bind(...[undefined], undefined)", None, None, None),
        // ("foo.bind(bar, baz, undefined)", None, None, None),
        // ("foo?.bind(bar, undefined)", None, None, None),
        // ("foo(undefined, undefined);", Some(serde_json::json!(optionsIgnoreArguments)), None, None),
        // ("foo.bind(undefined);", Some(serde_json::json!(optionsIgnoreArguments)), None, None),
        // (
        //     "const foo = () => undefined",
        //     Some(serde_json::json!(optionsIgnoreArrowFunctionBody)),
        //     None,
        //     None,
        // ),
        // ("prerenderPaths?.add(entry)", None, None, None),
        // (
        //     r#"
        // 				function getThing(): string | undefined {
        // 					if (someCondition) {
        // 						return "hello world";
        // 					}

        // 					return undefined;
        // 				}
        // 			"#,
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     r#"
        // 				function getThing(): string | undefined {
        // 					if (someCondition) {
        // 						return "hello world";
        // 					} else if (anotherCondition) {
        // 						return undefined;
        // 					}

        // 					return undefined;
        // 				}
        // 			"#,
        //     None,
        //     None,
        //     None,
        // ),
        // ("const foo = (): undefined => {return undefined;}", None, None, None),
        // ("const foo = (): undefined => undefined;", None, None, None),
        // ("const foo = (): string => undefined;", None, None, None),
        // ("const foo = function (): undefined {return undefined}", None, None, None),
        // ("export function foo(): undefined {return undefined}", None, None, None),
        // (
        //     "
        // 				const object = {
        // 					method(): undefined {
        // 						return undefined;
        // 					}
        // 				}
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 				class A {
        // 					method(): undefined {
        // 						return undefined;
        // 					}
        // 				}
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 				const A = class A {
        // 					method(): undefined {
        // 						return undefined
        // 					}
        // 				};
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 				class A {
        // 					static method(): undefined {
        // 						return undefined
        // 					}
        // 				}
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 				class A {
        // 					get method(): undefined {
        // 						return undefined;
        // 					}
        // 				}
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 				class A {
        // 					static get method(): undefined {
        // 						return undefined;
        // 					}
        // 				}
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 				class A {
        // 					#method(): undefined {
        // 						return undefined;
        // 					}
        // 				}
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 				class A {
        // 					private method(): undefined {
        // 						return undefined;
        // 					}
        // 				}
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // ("createContext<T>(undefined);", None, None, None),
        // ("React.createContext<T>(undefined);", None, None, None),
        // Oxlint doesn't support vue?
        // (
        //     "
        // 				<script>
        // 				import {ref} from 'vue';

        // 				export default {
        // 					setup() {
        // 						return {foo: ref(undefined)};
        // 					}
        // 				};
        // 				</script>
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 				<script setup>
        // 				import * as vue from 'vue';
        // 				const foo = vue.ref(undefined);
        // 				</script>
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
    ];

    let fail = vec![
        // ("function foo() {return undefined;}", None, None, None),
        ("const foo = () => undefined;", None, None, None),
        // ("const foo = () => {return undefined;};", None, None, None),
        // ("function foo() {return       undefined;}", None, None, None),
        // ("function foo() {return /* comment */ undefined;}", None, None, None),
        // ("function* foo() {yield undefined;}", None, None, None),
        // ("function* foo() {yield                 undefined;}", None, None, None),
        // ("let a = undefined;", None, None, None),
        // ("let a = undefined, b = 2;", None, None, None),
        // ("var a = undefined;", None, None, None),
        // ("var a = undefined, b = 2;", None, None, None),
        // ("foo(undefined);", None, None, None),
        // ("foo(undefined, undefined);", None, None, None),
        // ("foo(undefined,);", None, None, None),
        // ("foo(undefined, undefined,);", None, None, None),
        // ("foo(bar, undefined);", None, None, None),
        // ("foo(bar, undefined, undefined);", None, None, None),
        // ("foo(undefined, bar, undefined);", None, None, None),
        // ("foo(bar, undefined,);", None, None, None),
        // ("foo(undefined, bar, undefined,);", None, None, None),
        // ("foo(bar, undefined, undefined,);", None, None, None),
        // ("foo(undefined, bar, undefined, undefined,);", None, None, None),
        // (
        //     "
        // 					foo(
        // 						undefined,
        // 						bar,
        // 						undefined,
        // 						undefined,
        // 						undefined,
        // 						undefined,
        // 					)
        // 				",
        //     None,
        //     None,
        //     None,
        // ),
        // ("const {foo = undefined} = {};", None, None, None),
        // ("const [foo = undefined] = [];", None, None, None),
        // ("function foo(bar = undefined) {}", None, None, None),
        // ("function foo({bar = undefined}) {}", None, None, None),
        // ("function foo({bar = undefined} = {}) {}", None, None, None),
        // ("function foo([bar = undefined]) {}", None, None, None),
        // ("function foo([bar = undefined] = []) {}", None, None, None),
        // ("return undefined;", None, None, None), // {				"parserOptions": {					"sourceType": "script",					"ecmaFeatures": {						"globalReturn": true,					},				},			},
        // (
        //     "
        // 					function foo():undefined {
        // 						function nested() {
        // 							return undefined;
        // 						}

        // 						return nested();
        // 					}
        // 				",
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 				foo(
        // 					undefined,
        // 					bar,
        // 					undefined,
        // 					undefined,
        // 					undefined,
        // 					undefined,
        // 				)
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // ("function foo([bar = undefined] = []) {}", None, None, None),
        // ("foo(bar, undefined, undefined);", None, None, None),
        // ("let a = undefined, b = 2;", None, None, None),
        // (
        //     "
        // 				function foo() {
        // 					return /* */ (
        // 						/* */
        // 						(
        // 							/* */
        // 							undefined
        // 							/* */
        // 						)
        // 						/* */
        // 					) /* */ ;
        // 				}
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 				function * foo() {
        // 					yield /* */ (
        // 						/* */
        // 						(
        // 							/* */
        // 							undefined
        // 							/* */
        // 						)
        // 						/* */
        // 					) /* */ ;
        // 				}
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 				const foo = () => /* */ (
        // 					/* */
        // 					(
        // 						/* */
        // 						undefined
        // 						/* */
        // 					)
        // 					/* */
        // 				);
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // ("foo.bind(undefined)", None, None, None),
        // ("bind(foo, undefined)", None, None, None),
        // ("foo.bind?.(bar, undefined)", None, None, None),
        // ("foo[bind](bar, undefined)", None, None, None),
        // ("foo.notBind(bar, undefined)", None, None, None),
        // (
        //     "
        // 				<script>
        // 				import {nextTick} from 'vue';
        // 				const foo = nextTick(undefined);
        // 				</script>
        // 			",
        //     None,
        //     None,
        //     None,
        // ),
        // ("function f(foo: Type = undefined) {}", None, None, None),
        // ("function f(foo?: Type = undefined) {}", None, None, None),
        // ("const f = function(foo: Type = undefined) {}", None, None, None),
        // ("const f = (foo: Type = undefined) => {}", None, None, None),
        // ("const f = {method(foo: Type = undefined){}}", None, None, None),
        // ("const f = class {method(foo: Type = undefined){}}", None, None, None),
        // ("function f(foo = undefined) {}", None, None, None),
        // ("function a({foo} = undefined) {}", None, None, Some(PathBuf::from("'foo.ts'"))),
    ];

    // let fix = vec![
    //     ("function foo() {return undefined;}", "function foo() {return;}", None),
    //     ("const foo = () => undefined;", "const foo = () => {};", None),
    //     ("const foo = () => {return undefined;};", "const foo = () => {return;};", None),
    //     ("function foo() {return       undefined;}", "function foo() {return;}", None),
    //     (
    //         "function foo() {return /* comment */ undefined;}",
    //         "function foo() {return /* comment */;}",
    //         None,
    //     ),
    //     ("function* foo() {yield undefined;}", "function* foo() {yield;}", None),
    //     ("function* foo() {yield                 undefined;}", "function* foo() {yield;}", None),
    //     ("let a = undefined;", "let a;", None),
    //     ("let a = undefined, b = 2;", "let a, b = 2;", None),
    //     ("var a = undefined;", "var a;", None),
    //     ("var a = undefined, b = 2;", "var a, b = 2;", None),
    //     ("foo(undefined);", "foo();", None),
    //     ("foo(undefined, undefined);", "foo();", None),
    //     ("foo(undefined,);", "foo();", None),
    //     ("foo(undefined, undefined,);", "foo();", None),
    //     ("foo(bar, undefined);", "foo(bar);", None),
    //     ("foo(bar, undefined, undefined);", "foo(bar);", None),
    //     ("foo(undefined, bar, undefined);", "foo(undefined, bar);", None),
    //     ("foo(bar, undefined,);", "foo(bar,);", None),
    //     ("foo(undefined, bar, undefined,);", "foo(undefined, bar,);", None),
    //     ("foo(bar, undefined, undefined,);", "foo(bar,);", None),
    //     ("foo(undefined, bar, undefined, undefined,);", "foo(undefined, bar,);", None),
    //     (
    //         "
    // 						foo(
    // 							undefined,
    // 							bar,
    // 							undefined,
    // 							undefined,
    // 							undefined,
    // 							undefined,
    // 						)
    // 					",
    //         "
    // 						foo(
    // 							undefined,
    // 							bar,
    // 						)
    // 					",
    //         None,
    //     ),
    //     ("const {foo = undefined} = {};", "const {foo} = {};", None),
    //     ("const [foo = undefined] = [];", "const [foo] = [];", None),
    //     ("function foo(bar = undefined) {}", "function foo(bar) {}", None),
    //     ("function foo({bar = undefined}) {}", "function foo({bar}) {}", None),
    //     ("function foo({bar = undefined} = {}) {}", "function foo({bar} = {}) {}", None),
    //     ("function foo([bar = undefined]) {}", "function foo([bar]) {}", None),
    //     ("function foo([bar = undefined] = []) {}", "function foo([bar] = []) {}", None),
    //     ("return undefined;", "return;", None),
    //     (
    //         "
    // 						function foo():undefined {
    // 							function nested() {
    // 								return undefined;
    // 							}

    // 							return nested();
    // 						}
    // 					",
    //         "
    // 						function foo():undefined {
    // 							function nested() {
    // 								return;
    // 							}

    // 							return nested();
    // 						}
    // 					",
    //         None,
    //     ),
    // ];
    Tester::new(NoUselessUndefined::NAME, pass, fail).test_and_snapshot();
}
