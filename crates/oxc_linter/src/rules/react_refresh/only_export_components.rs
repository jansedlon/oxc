use lazy_static::lazy_static;
use regex::Regex;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

lazy_static! {
    static ref POSSIBLE_REACT_EXPORT_RE: Regex = Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap();
    static ref STRICT_REACT_EXPORT_RE: Regex =
        Regex::new(r"^[A-Z][a-zA-Z0-9]*[a-z]+[a-zA-Z0-9]*$").unwrap();
    static ref REACT_HOCS: [&'static str; 2] = ["with", "forwardRef"];
}

#[derive(Debug, Clone, Default)]
pub struct OnlyExportComponents {}

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
    OnlyExportComponents,
    correctness, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
);

fn report_export_all(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-react-refresh(only-export-components): This rule can't verify that `export *` only exports components.")
        .with_label(span0)
}

// fn report_named_exports(span0: Span) -> OxcDiagnostic {
//     OxcDiagnostic::warn("eslint-plugin-react-refresh(only-export-components): Fast refresh only works when afile only exports components.")
//     .with_help("Use a new file to share constants or functions between components.")
//         .with_label(span0)
// }

// fn report_anonymous_export(span0: Span) -> OxcDiagnostic {
//     OxcDiagnostic::warn("eslint-plugin-react-refresh(only-export-components): Fast refresh can't handle anonymous components.")
//     .with_help("Add a name to your export.")
//         .with_label(span0)
// }

// fn report_local_components(span0: Span) -> OxcDiagnostic {
//     OxcDiagnostic::warn("eslint-plugin-react-refresh(only-export-components): Fast refresh only works when a file only exports components.")
//         .with_help("Move your component(s) to a separate file.")
//         .with_label(span0)
// }

// fn report_no_export(span0: Span) -> OxcDiagnostic {
//     OxcDiagnostic::warn("eslint-plugin-react-refresh(only-export-components): Fast refresh only works when a file has exports.")
//     .with_help("Move you component(s) to a separate file")
//         .with_label(span0)
// }

// #[derive(Debug, Clone)]
// enum BindingPatterOrIdentifier<'a> {
//     BindingIdentifier(&'a BindingIdentifier<'a>),
//     BindingPattern(&'a BindingPattern<'a>),
//     IdentifierReference(&'a IdentifierReference<'a>),
// }

impl Rule for OnlyExportComponents {
    fn run_once(&self, ctx: &LintContext) {
        let module_record = ctx.module_record();

        // If there's no export / import, there's no point in running this rule
        if module_record.not_esm {
            return;
        }

        // We only care about TSX / JSX files
        match module_record.resolved_absolute_path.extension() {
            Some(ext) => {
                if ext != "tsx" && ext != "jsx" {
                    return;
                }
            }
            None => {
                return;
            }
        }

        // No export * are allowed
        for star_export in &module_record.star_export_entries {
            // println!("{:#?}", star_export);
            let Some(module_request) = &star_export.module_request else {
                continue;
            };

            let requested_modules = module_record.requested_modules.get(module_request.name());

            if let Some(requested_modules) = requested_modules {
                for requested_module in requested_modules {
                    // println!("Requested module {:#?}", requested_module);
                    if !requested_module.is_import && !requested_module.is_type {
                        println!("should fail");
                        ctx.diagnostic(report_export_all(star_export.span));
                    }
                }
            }
        }

        // let mut may_have_react_export = false;
        // let mut react_is_in_scope = false;
        // let mut local_components: Vec<IdentifierReference> = vec![];
        // let mut non_component_exports: Vec<BindingPatterOrIdentifier> = vec![];

        //     for node in ctx.nodes().iter() {
        //         match node.kind() {
        //             AstKind::ExportDefaultDeclaration(export_default_declaration) => {
        //                 // match export_default_declaration.declaration {
        //                 //     ExportDefaultDeclarationKind::Identifier(identifier_reference) => {
        //                 //         handle_export_identifier(
        //                 //             BindingPatterOrIdentifier::IdentifierReference(
        //                 //                 identifier_reference,
        //                 //             ),
        //                 //             None,
        //                 //             None,
        //                 //             &mut non_component_exports,
        //                 //             &mut may_have_react_export,
        //                 //             &self.allow_export_names,
        //                 //             self.allow_constant_export,
        //                 //         );
        //                 //     }
        //                 // }
        //             }
        //             _ => {}
        //         }
        //     }
    }

    // fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    //     let mut run_data = OnlyExportComponentsRun::default_from_rule(self);

    //     // println!("{:#?}", node);
    //     match node.kind() {
    //         // ✅
    //         AstKind::ExportAllDeclaration(export_all) => {
    //             if export_all.export_kind == ImportOrExportKind::Type {
    //                 return;
    //             }

    //             run_data.has_exports = true;

    //             ctx.diagnostic(report_export_all(export_all.span));
    //         }
    //         // 🚧
    //         AstKind::ExportDefaultDeclaration(_export_default) => {
    //             //     run_data.has_exports = true;

    //             //     /*
    //             //      * The origin eslint rule also matches `VariableDeclaration` but that doesn't seem to be valid syntax?
    //             //      * https://tc39.es/ecma262/#prod-ExportDeclaration
    //             //      */
    //             //     match &export_default.declaration {
    //             //         ExportDefaultDeclarationKind::FunctionDeclaration(declaration) => {
    //             //             handle_export_declaration(
    //             //                 ExportDeclaration::FunctionDeclaration(declaration),
    //             //                 &mut run_data,
    //             //             );
    //             //         }
    //             //         ExportDefaultDeclarationKind::CallExpression(declaration) => {
    //             //             handle_export_declaration(
    //             //                 ExportDeclaration::CallExpression(declaration),
    //             //                 &mut run_data,
    //             //             );
    //             //         }
    //             //         ExportDefaultDeclarationKind::Identifier(identifier_reference) => {
    //             //             handle_export_identifier(
    //             //                 &HandleExportIdentifier::IdentifierReference(identifier_reference),
    //             //                 None,
    //             //                 None,
    //             //                 &mut run_data,
    //             //             )
    //             //         }
    //             //         ExportDefaultDeclarationKind::ArrowFunctionExpression(expression) => {
    //             //             ctx.diagnostic(report_anonymous_export(expression.span));
    //             //         }
    //             //         _ => {}
    //             //     }
    //         }
    //         // 🚧
    //         AstKind::ExportNamedDeclaration(_named_declaration) => {
    //             //     run_data.has_exports = true;

    //             //     if let Some(_) = named_declaration.declaration {
    //             //         handle_export_declaration(
    //             //             ExportDeclaration::NamedDeclaration(named_declaration),
    //             //             &mut run_data,
    //             //         );
    //             //     }

    //             //     // for specifier in &named_declaration.specifiers {
    //             //     //     let default_identifier = "default".to_string();
    //             //     //     let new_identifier = match specifier.exported.name().to_string() {
    //             //     //         default_identifier => specifier.local,
    //             //     //         _ => specifier.exported,
    //             //     //     };

    //             //     // handle_export_identifier(new_identifier, None, None, &mut run_data)
    //             //     // }
    //             // }
    //             // AstKind::VariableDeclaration(variable_declaration) => {
    //             //     for variable in variable_declaration.declarations {
    //             //         handle_local_identifier(Some(&variable.id), &mut run_data);
    //             //     }
    //         }
    //         // 🚧
    //         AstKind::VariableDeclaration(variable_declaration) => {
    //             for variable in &variable_declaration.declarations {
    //                 let variable_id = &variable.id;

    //                 // handle_local_identifier(
    //                 //     BindingPatterOrIdentifier::BindingPattern(variable_id.clone()),
    //                 //     &mut run_data,
    //                 // );
    //             }
    //         }
    //         // ✅
    //         AstKind::Function(function_declaration) => {
    //             if let Some(function_declaration_id) = &function_declaration.id {
    //                 handle_local_identifier(
    //                     BindingPatterOrIdentifier::BindingIdentifier(function_declaration_id),
    //                     &mut run_data,
    //                 );
    //             }
    //         }
    //         // ✅
    //         AstKind::ImportDeclaration(import_declaration) => {
    //             if import_declaration.source.value.to_string() == "React" {
    //                 run_data.react_is_in_scope = true;
    //             }
    //         }
    //         _ => {}
    //     }

    //     // if run_data.check_js && !run_data.react_is_in_scope {
    //     // return;
    //     // }

    //     // if run_data.has_exports {
    //     //     if run_data.may_have_react_export {

    //     //     } else if run_data.loca
    //     // }
    // }
}

// fn handle_local_identifier<'a>(
//     identifier_node: &'a BindingPatterOrIdentifier,
//     local_components: &mut Vec<&'a BindingPatterOrIdentifier<'a>>,
// ) {
//     match identifier_node {
//         BindingPatterOrIdentifier::BindingIdentifier(identifier) => {
//             if POSSIBLE_REACT_EXPORT_RE.is_match(identifier.name.as_str()) {
//                 local_components.push(&identifier_node);
//             }
//         }
//         _ => {}
//     }
// }

// fn handle_export_identifier<'a>(
//     identifier_node: BindingPatterOrIdentifier<'a>,
//     is_function: Option<bool>,
//     init: Option<Expression>,
//     non_component_exports: &mut Vec<BindingPatterOrIdentifier<'a>>,
//     may_have_react_export: &mut bool,
//     allow_export_names: &Vec<String>,
//     allow_constant_export: bool,
// ) {
//     let BindingPatterOrIdentifier::BindingIdentifier(identifier_node) = identifier_node else {
//         non_component_exports.push(identifier_node);
//         return;
//     };

//     let identifier_node_name = identifier_node.name.to_string();

//     if allow_export_names.contains(&identifier_node_name) {
//         return;
//     }

//     match init {
//         Some(Expression::StringLiteral(_))
//         | Some(Expression::TemplateLiteral(_))
//         | Some(Expression::BinaryExpression(_)) => {
//             return;
//         }
//         _ => {
//             if is_function.is_some_and(|is_function| is_function) {
//                 if POSSIBLE_REACT_EXPORT_RE.is_match(&identifier_node_name) {
//                     *may_have_react_export = true;
//                 } else {
//                     let binding_identifier =
//                         BindingPatterOrIdentifier::BindingIdentifier(&identifier_node);

//                     non_component_exports.push(binding_identifier);
//                 }
//             }
//         }
//     }
// }

// enum ExportDeclaration<'a> {
//     FunctionDeclaration(&'a oxc_allocator::Box<'a, oxc_ast::ast::Function<'a>>),
//     CallExpression(&'a oxc_allocator::Box<'a, oxc_ast::ast::CallExpression<'a>>),
//     TSEnumDeclaration(&'a oxc_allocator::Box<'a, oxc_ast::ast::TSEnumDeclaration<'a>>),
//     Declaration(&'a oxc_allocator::Box<'a, oxc_ast::ast::Declaration<'a>>),
//     NamedDeclaration(&'a ExportNamedDeclaration<'a>),
//     IdentifierReference(&'a oxc_allocator::Box<'a, oxc_ast::ast::IdentifierReference<'a>>),
// }

// fn handle_export_declaration<'a>(
//     declaration: ExportDeclaration<'a>,
//     run_data: &'a mut OnlyExportComponentsRun<'a>,
// ) -> bool {
//     // match declaration {
//     //     ExportDeclaration::FunctionDeclaration(function) => {
//     //         if let Some(id) = &function.id {
//     //             handle_export_identifier(&id, Some(true), None, run_data);
//     //         }
//     //     }
//     //     ExportDeclaration::CallExpression(call_expression) => {
//     //         if let Some(callee_name) = call_expression.callee_name() {
//     //             if REACT_HOCS.contains(&callee_name) {
//     //                 let first_argument = call_expression.arguments.get(0);

//     //                 if let Some(first_argument) = first_argument {
//     //                     if let Argument::FunctionExpression(expression) = first_argument {
//     //                         if let Some(expression_id) = &expression.id {
//     //                             handle_export_identifier(expression_id, Some(true), None, run_data);
//     //                         }
//     //                     }
//     //                 }
//     //             }
//     //         }
//     //     }
//     //     ExportDeclaration::TSEnumDeclaration(declaration) => {
//     //         run_data.non_component_exports.push(&declaration.id);
//     //     }
//     // }

//     true
// }

// enum HandleExportIdentifier<'a> {
//     IdentifierReference(&'a oxc_allocator::Box<'a, IdentifierReference<'a>>),
// }

// fn handle_export_identifier<'a>(
//     identifier: &'a HandleExportIdentifier<'a>,
//     is_function: Option<bool>,
//     init: Option<Expression>,
//     run_data: &'a mut OnlyExportComponentsRun<'a>,
// ) {
//     // let identifier_name = identifier.name.to_string();
//     //
//     // /*
//     //  * If there is any specific allowed export names, just ignore it.
//     //  * Examples are `loader`, `action`, ... from Remix.run
//     //  */
//     // if run_data.rule.allow_export_names.contains(&identifier_name) {
//     //     return;
//     // }
//     //
//     // /*
//     //  * If contant exports are allowed,
//     //  * eg. `export const hello = "world"`
//     //  * also ignore it
//     //  */
//     // if run_data.rule.allow_constant_export {
//     //     match init {
//     //         Some(Expression::StringLiteral(_)) => {
//     //             return;
//     //         }
//     //         Some(Expression::TemplateLiteral(_)) => {
//     //             return;
//     //         }
//     //         Some(Expression::BinaryExpression(_)) => {
//     //             return;
//     //         }
//     //         _ => {}
//     //     }
//     // }
//     //
//     // if is_function.is_some() && is_function.unwrap() == true {
//     //     if POSSIBLE_REACT_EXPORT_RE.is_match(&identifier_name) {
//     //         run_data.may_have_react_export = true;
//     //     } else {
//     //         run_data.non_component_exports.push(&identifier);
//     //     }
//     // } else {
//     //     if let Some(init) = init {
//     //         match init {
//     //             Expression::ArrayExpression(_)
//     //             | Expression::AwaitExpression(_)
//     //             | Expression::BinaryExpression(_)
//     //             | Expression::ChainExpression(_)
//     //             | Expression::ConditionalExpression(_)
//     //             | Expression::StringLiteral(_)
//     //             | Expression::LogicalExpression(_)
//     //             | Expression::ObjectExpression(_)
//     //             | Expression::TemplateLiteral(_)
//     //             | Expression::ThisExpression(_)
//     //             | Expression::UnaryExpression(_)
//     //             | Expression::UpdateExpression(_) => {
//     //                 run_data.non_component_exports.push(&identifier);
//     //
//     //                 return;
//     //             }
//     //             _ => {}
//     //         }
//     //     }
//     //
//     //     if !run_data.may_have_react_export && POSSIBLE_REACT_EXPORT_RE.is_match(&identifier_name) {
//     //         run_data.may_have_react_export = true;
//     //     }
//     //
//     //     if !STRICT_REACT_EXPORT_RE.is_match(&identifier_name) {
//     //         run_data.non_component_exports.push(&identifier);
//     //     }
//     // }
// }

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // (r"export function Foo() {};", None),
        // (r"function Foo() {}; export { Foo };", None),
        // (r"function foo() {}; export default Foo;", None),
        // (r"export default function Foo() {}", None),
        // (r"export const Foo = () => {};", None),
        // (r"export const Foo2 = () => {}", None),
        // (r"export function CMS() {};", None),
        // (r"export const SVG = forwardRef(() => <svg />);", None),
        // (r"export const CMS = () => {};", None),
        // (r"const Foo = () => {}; export { Foo };", None),
        // (r"const Foo = () => {}; export default Foo;", None),
        // (r"const foo = 4; export const Bar = () => {}; export const Baz = () => {};", None),
        // (r"const foo = () => {}; export const Bar = () => {}; export const Baz = () => {};", None),
        // (r"export const Foo = () => {}; export const Bar= styled.div`padding-bottom: 6px`;", None),
        // (r"export const foo = 3;", None),
        // (r"const foo = 3; const bar = 'Hello'; export { foo, bar };", None),
        // (r"export const foo = () => {};", None),
        // (r"export default function foo () {};", None),
        // (r"export default memo(function Foo () {});", None),
        // (r"export type * from './module';", None),
        // (r"export const foo = () => {}; export const Bar = () => {};", None),
        // (r"export const foo = () => {}; export const Bar = () => {};", Some(serde_json::json!([{ "checkJS": true }]))),
        // (r"export const foo = 4; export const Bar = () => {};", Some(serde_json::json!([{ "allowConstantExport": true }]))),
        // (r"export const CONSTANT = 'Hello world'; export const Foo = () => {};", Some(serde_json::json!([{ "allowConstantExport": true }]))),
        // (r"const foo = 'world'; export const CONSTANT = `Hello ${foo}`; export const Foo = () => {};", Some(serde_json::json!([{ "allowConstantExport": true }]))),
        // (r"export const loader = () => {}; export const Bar = () => {};", Some(serde_json::json!([{ "allowExportNames": ["loader", "meta"] }]))),
        // (r"export function loader() {}; export const Bar = () => {};", Some(serde_json::json!([{ "allowExportNames": ["loader", "meta"] }]))),
        // (r"export const loader = () => {}; export const meta = { title: 'Home' };", Some(serde_json::json!([{ "allowExportNames": ["loader", "meta"] }]))),
        // (r"export { App as default }; const App = () => <>Text</>;", None)
    ];

    let fail = vec![
        (r"export enum Tab { Home, Settings }; export const Bar = () => {};", None),
        // (r"export * from 'react';", None),
    ];

    Tester::new(OnlyExportComponents::NAME, pass, fail)
        .with_react_refresh_plugin(true)
        .test_and_snapshot();
}
