// External scanner for tree-sitter-jaw.
//
// Decides whether the line after a comment, log or note continues it. That
// needs lookahead past the newline (and, for logs/notes, the marker's
// indentation), which tree-sitter's regex lexer cannot express. Mirrors the
// begin/end rules in editors/vscode/syntaxes/jaw.tmLanguage.json:
//
// - [^] / [*] comment: continues until the next non-blank line starts with a
//   JAW construct ([N], a marker, [ID] followed by — or :, or /).
// - [•] log / [!] note: continues while the next non-blank line is indented
//   deeper than the marker.

#include "tree_sitter/parser.h"

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

enum TokenType {
  LOG_MARKER,
  NOTE_MARKER,
  COMMENT_NEWLINE,
  MESSAGE_NEWLINE,
  ERROR_SENTINEL,
};

#define BULLET 0x2022
#define EM_DASH 0x2014

typedef struct {
  // Column of the most recent [•] / [!] marker.
  uint32_t marker_column;
} Scanner;

static inline void advance(TSLexer *lexer) { lexer->advance(lexer, false); }
static inline void skip(TSLexer *lexer) { lexer->advance(lexer, true); }

static inline bool is_space(int32_t c) { return c == ' ' || c == '\t'; }

static inline bool is_ident_char(int32_t c) {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
         (c >= '0' && c <= '9') || c == '_' ||
         (c >= 0x80 && c != BULLET && c != EM_DASH);
}

static void skip_spaces(TSLexer *lexer) {
  while (is_space(lexer->lookahead)) advance(lexer);
}

// [ ws* <c> ws* ] with the lexer on '['. Consumes what it reads.
static bool read_marker_tail(TSLexer *lexer, int32_t c) {
  skip_spaces(lexer);
  if (lexer->lookahead != c) return false;
  advance(lexer);
  skip_spaces(lexer);
  return lexer->lookahead == ']';
}

// Does the line at the lexer start a JAW construct that ends a comment?
static bool starts_construct(TSLexer *lexer) {
  if (lexer->lookahead == '/') return true;
  if (lexer->lookahead != '[') return false;
  advance(lexer);
  skip_spaces(lexer);

  int32_t c = lexer->lookahead;
  if (c >= '0' && c <= '9') {
    while (lexer->lookahead >= '0' && lexer->lookahead <= '9') advance(lexer);
    skip_spaces(lexer);
    return lexer->lookahead == ']';
  }
  if (c == '^' || c == '*' || c == '!' || c == '>' || c == '~' || c == '&' ||
      c == '+' || c == '-' || c == BULLET) {
    advance(lexer);
    skip_spaces(lexer);
    return lexer->lookahead == ']';
  }
  if (is_ident_char(c)) {
    while (is_ident_char(lexer->lookahead)) advance(lexer);
    skip_spaces(lexer);
    if (lexer->lookahead != ']') return false;
    advance(lexer);
    skip_spaces(lexer);
    return lexer->lookahead == EM_DASH || lexer->lookahead == ':';
  }
  return false;
}

static bool scan_newline(Scanner *s, TSLexer *lexer, const bool *valid) {
  while (is_space(lexer->lookahead) || lexer->lookahead == '\r') skip(lexer);
  if (lexer->lookahead != '\n') return false;
  advance(lexer);
  lexer->mark_end(lexer);

  // Peek past blank lines to the next line with content.
  for (;;) {
    skip_spaces(lexer);
    if (lexer->lookahead == '\r') advance(lexer);
    if (lexer->lookahead == '\n') {
      advance(lexer);
      continue;
    }
    break;
  }
  if (lexer->eof(lexer)) return false;

  if (valid[MESSAGE_NEWLINE]) {
    if (lexer->get_column(lexer) <= s->marker_column) return false;
    lexer->result_symbol = MESSAGE_NEWLINE;
    return true;
  }

  if (starts_construct(lexer)) return false;
  lexer->result_symbol = COMMENT_NEWLINE;
  return true;
}

static bool scan_marker(Scanner *s, TSLexer *lexer, const bool *valid) {
  while (is_space(lexer->lookahead)) skip(lexer);
  if (lexer->lookahead != '[') return false;
  uint32_t column = lexer->get_column(lexer);
  advance(lexer);
  skip_spaces(lexer);

  enum TokenType symbol;
  int32_t c;
  if (lexer->lookahead == BULLET && valid[LOG_MARKER]) {
    symbol = LOG_MARKER;
    c = BULLET;
  } else if (lexer->lookahead == '!' && valid[NOTE_MARKER]) {
    symbol = NOTE_MARKER;
    c = '!';
  } else {
    return false;
  }
  if (!read_marker_tail(lexer, c)) return false;
  advance(lexer);

  s->marker_column = column;
  lexer->result_symbol = symbol;
  return true;
}

void *tree_sitter_jaw_external_scanner_create(void) {
  return calloc(1, sizeof(Scanner));
}

void tree_sitter_jaw_external_scanner_destroy(void *payload) { free(payload); }

unsigned tree_sitter_jaw_external_scanner_serialize(void *payload,
                                                    char *buffer) {
  memcpy(buffer, payload, sizeof(Scanner));
  return sizeof(Scanner);
}

void tree_sitter_jaw_external_scanner_deserialize(void *payload,
                                                  const char *buffer,
                                                  unsigned length) {
  Scanner *s = payload;
  if (length == sizeof(Scanner)) {
    memcpy(s, buffer, sizeof(Scanner));
  } else {
    s->marker_column = 0;
  }
}

bool tree_sitter_jaw_external_scanner_scan(void *payload, TSLexer *lexer,
                                           const bool *valid_symbols) {
  Scanner *s = payload;
  if (valid_symbols[ERROR_SENTINEL]) return false;

  if (valid_symbols[COMMENT_NEWLINE] || valid_symbols[MESSAGE_NEWLINE]) {
    return scan_newline(s, lexer, valid_symbols);
  }
  if (valid_symbols[LOG_MARKER] || valid_symbols[NOTE_MARKER]) {
    return scan_marker(s, lexer, valid_symbols);
  }
  return false;
}
