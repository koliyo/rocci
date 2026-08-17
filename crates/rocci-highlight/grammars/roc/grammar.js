/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

/**
 * Tree-sitter grammar definition for Roc.
 * Derived from upstream faldor20/tree-sitter-roc (pinned by Zed Roc).
 */

const PREC = {
  PATTERN: 0,
  FIELD_ACCESS_START: 1,
  WHERE_IMPLEMENTS: 1,
  TAG: 1,
  FUNCTION_START: 1,
  PART: 1,
  PREFIX_EXPR: 1,
  TYPEALIAS: 2,
  CASE_OF_BRANCH: 6,
  FUNC: 10,
  IMPORT: 20,
  ARGS: 20,
};

module.exports = grammar({
  name: "roc",

  externals: ($) => [
    $._newline,
    $._end_newline,
    $._indent,
    $._dedent,
    $.comment,
    "]",
    ")",
    "}",
    "except",
  ],

  extras: ($) => [
    $.line_comment,
    $.doc_comment,
    /[ \s\f\uFEFF\u2060\u200B]|\\\r?n/,
  ],

  conflicts: ($) => [
    [$._pattern, $._atom_expr],
    [$._atomic_pattern, $._atom_expr],
    [$.body_expression, $.record_expr],
    [$.tag_pattern, $.tag_expr],
    [$.record_field_pattern, $.record_field_expr],
    [$.record_field_pattern, $.record_field_expr, $.annotation_pre_colon],
    [$.record_field_expr, $.annotation_pre_colon],
    [$.record_expr, $.body_expression, $.record_pattern],
    [$._tags_only],
    [$.identifier_pattern, $.long_identifier],
    [$.list_pattern, $.list_expr],
    [$._module_elem, $.value_declaration],
    [$._module_elem, $.var_declaration],
  ],

  words: ($) => /\s+/,
  word: ($) => $._lower_identifier,

  inline: ($) => [
    $._non_atomic_type,
    $.field_name,
    $.bound_variable,
    $.operator,
    $.suffix_operator,
    $.inferred,
  ],

  rules: {
    file: ($) => seq(optional($._header), repeat1($._module_elem)),

    _module_elem: ($) =>
      choice(
        $.annotation_type_def,
        $.alias_type_def,
        $.opaque_type_def,
        $.nominal_type_def,
        $.expect,
        $.value_declaration,
        $.var_declaration,
        $.expr_body,
        $.import_expr,
        $.import_file_expr,
      ),

    expect: ($) => prec(1, seq("expect", field("body", $.expr_body))),

    value_declaration: ($) =>
      seq(
        optional(seq($.annotation_type_def)),
        alias($._assignment_pattern, $.decl_left),
        "=",
        field("body", alias($.expr_body_terminal, $.expr_body)),
      ),

    var_declaration: ($) =>
      seq(
        optional(seq($.annotation_type_def)),
        "var",
        field("name", $.identifier),
        "=",
        field("body", alias($.expr_body_terminal, $.expr_body)),
      ),

    body_expression: ($) =>
      seq(
        "{",
        repeat(choice($.value_declaration, $.var_declaration, $._expr_inner)),
        "}",
      ),

    expr_body: ($) => $._expr_inner,
    expr_body_terminal: ($) => $._expr_inner,

    _atom_expr: ($) =>
      choice(
        $.anon_fun_expr,
        $.const,
        $.record_expr,
        $.record_builder_expr,
        $._variable_expr,
        $.parenthesized_expr,
        $.body_expression,
        $.operator_as_function_expr,
        $.tag_expr,
        $.tuple_expr,
        $.list_expr,
        $.field_access_expr,
        $.todo_expr,
        $.function_call_pnc_expr,
        $.suffix_op_expr,
        $.prefixed_expression,
      ),

    _expr_inner: ($) =>
      choice(
        $.bin_op_expr,
        $._atom_expr,
        $.for_expr,
        $.if_expr,
        $.match_expr,
        $.early_return_expr,
        $.dbg_expr,
      ),

    prefixed_expression: ($) =>
      prec(
        PREC.PREFIX_EXPR,
        seq(
          choice("!", "*", "-", "^"),
          choice(
            $.const,
            $.parenthesized_expr,
            $.field_access_expr,
            $._variable_expr,
            $.function_call_pnc_expr,
          ),
        ),
      ),

    dbg_expr: ($) => seq("dbg", alias($.expr_body_terminal, $.expr_body)),

    for_expr: ($) =>
      seq(
        "for",
        field("pattern", $._pattern),
        "in",
        field("iterable", $._expr_inner),
        field("body", $._expr_inner),
      ),

    early_return_expr: ($) => seq("return", field("body", $.expr_body)),

    _variable_expr: ($) =>
      alias($.long_identifier, $.variable_expr),

    long_identifier: ($) =>
      prec.right(
        2,
        seq(
          repeat(seq($.module, token.immediate("."))),
          $.identifier,
        ),
      ),

    _long_upper_identifier: ($) =>
      prec.right(
        0,
        seq(
          repeat(seq($.module, token.immediate("."))),
          $.module,
        ),
      ),

    identifier: ($) =>
      prec(
        100,
        seq(
          optional(choice("$")),
          optional(choice("_")),
          $._lower_identifier,
          optional(token.immediate("!")),
        ),
      ),

    _lower_identifier: ($) => /[\p{Ll}][\p{XID_Continue}]*/,
    _upper_identifier: ($) => /[\p{Lu}][\p{XID_Continue}]*/,

    tag: ($) => $._long_upper_identifier,
    module: ($) => $._upper_identifier,
  },
});
