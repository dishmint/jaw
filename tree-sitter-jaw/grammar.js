/**
 * @file Tree-sitter grammar for JAW (Just A Word)
 * @license MIT
 *
 * Line-oriented: each line is a comment, log, note, loop header, or a generic
 * line of inline tokens. Nesting (function bodies, branches) is not modelled;
 * highlighting only needs the tokens. Comments, logs and notes swallow the
 * plain-text lines that follow them, matching the parser's continuation rule
 * (a following line that does not start with `[` or `/`).
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const IDENT = /[\p{L}_][\p{L}\p{N}_]*/;

const marker = (ch) => token(seq('[', /[ \t]*/, ch, /[ \t]*/, ']'));

module.exports = grammar({
  name: 'jaw',

  extras: (_) => [/[ \t\r\f﻿]/],

  word: ($) => $.identifier,

  rules: {
    source_file: ($) =>
      seq(repeat(seq(optional($._item), $._newline)), optional($._item)),

    _item: ($) => choice($.comment, $.log, $.note, $.loop, $.line),

    _newline: (_) => /\n/,

    // A following line of plain text, newline included.
    continuation: (_) => token(prec(1, /\n[ \t]*[^\s\[\/][^\n]*/)),

    // [^] code comment / [*] general comment
    comment: ($) =>
      seq(
        field('marker', choice($.code_comment_marker, $.general_comment_marker)),
        repeat(choice($.variable_ref, '[', $._comment_text)),
        repeat($.continuation),
      ),

    code_comment_marker: (_) => marker('^'),
    general_comment_marker: (_) => marker('*'),
    _comment_text: (_) => /[^\[\n]+/,

    // [•] — log
    log: ($) =>
      seq(field('marker', $.log_marker), repeat($._message), repeat($.continuation)),

    // [!] — important note
    note: ($) =>
      seq(field('marker', $.note_marker), repeat($._message), repeat($.continuation)),

    log_marker: (_) => marker('•'),
    note_marker: (_) => marker('!'),

    _message: ($) =>
      choice(
        $.variable_ref,
        $.step_ref,
        $.function_ref,
        $.decorator,
        $.em_dash,
        alias('@', $.operator),
        alias('/', $.operator),
        '[',
        ']',
        $.text,
      ),

    text: (_) => /[^\s\[\]#\/@—\n][^\[\]#\/@\n]*/,

    // [~] — loop header; `in` is a keyword only here.
    loop: ($) =>
      seq(field('marker', $.loop_marker), repeat(choice($._inline, 'in'))),

    loop_marker: (_) => marker('~'),

    line: ($) => repeat1($._inline),

    _inline: ($) =>
      choice(
        $.inline_assign,
        $.parallel_marker,
        $.true_marker,
        $.false_marker,
        $.return_marker,
        $.step_ref,
        $.variable_ref,
        $.function_ref,
        $.decorator,
        $.em_dash,
        $.operator,
        $.number,
        $.identifier,
        '[',
        ']',
        '(',
        ')',
        ',',
        ':',
        '.',
        ';',
        $._misc,
      ),

    // [X]: description = value — the description runs to `,`, `=` or end of line.
    inline_assign: ($) =>
      prec.right(1, seq($.variable_ref, ':', optional($.description))),

    description: ($) =>
      prec.right(
        repeat1(
          choice(
            $.variable_ref,
            $.step_ref,
            $.function_ref,
            alias('@', $.operator),
            '[',
            ']',
            $._description_text,
          ),
        ),
      ),

    _description_text: (_) => /[^\s\[\],=#\/@\n][^\[\],=#\/@\n]*/,

    parallel_marker: (_) => marker('&'),
    true_marker: (_) => marker('+'),
    false_marker: (_) => marker('-'),
    return_marker: (_) => marker('>'),

    // [1] — a step marker at line start, a step reference elsewhere.
    step_ref: (_) => token(seq('[', /[ \t]*/, /\d+/, /[ \t]*/, ']')),

    variable_ref: (_) => token(seq('[', /[ \t]*/, IDENT, /[ \t]*/, ']')),

    function_ref: (_) => token(seq('/', IDENT)),

    decorator: ($) =>
      seq(
        '#',
        field('name', alias(token.immediate(/[\p{L}\p{N}_]+/), $.identifier)),
        optional(
          seq(
            token.immediate(':'),
            field('value', alias(token.immediate(/[^\s#]+/), $.value)),
          ),
        ),
      ),

    em_dash: (_) => '—',

    operator: (_) =>
      choice(
        '<<', '>>', '+=', '-=', '*=', '/=', '==', '!=', '<=', '>=', '&&', '||',
        '<', '>', '+', '-', '*', '/', '%', '=', '!', '?', '|', '@', '^', '&',
      ),

    number: (_) => /\d+(\.\d+)?/,

    identifier: (_) => IDENT,

    _misc: (_) => /[^\s\p{L}\p{N}_\[\]()\/#,:;.<>=+\-*%?|@!&^—]+/,
  },
});
