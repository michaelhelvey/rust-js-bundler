/**
 * @fileoverview Keywords and reserved words in Javascript.
 *
 * @see https://tc39.es/ecma262/#sec-keywords-and-reserved-words
 */

// Never allowed as identifiers:
export const AlwaysKeywords = new Set([
	'break',
	'case',
	'catch',
	'class',
	'const',
	'continue',
	'debugger',
	'default',
	'delete',
	'do',
	'else',
	'enum',
	'export',
	'extends',
	'finally',
	'for',
	'function',
	'if',
	'import',
	'in',
	'instanceof',
	'new',
	'return',
	'super',
	'switch',
	'this',
	'throw',
	'try',
	'typeof',
	'var',
	'void',
	'while',
	'with',
])

// Contextually disallowed as identifiers in strict mode:
export const StrictModeKeywords = new Set([
	'let',
	'static',
	'implements',
	'interface',
	'package',
	'protected',
	'private',
])

// Always allowed as identifiers, but can also be keywords in certain
// productions where identifiers are not allowed:
export const ContextualKeywords = new Set([
	'as',
	'async',
	'from',
	'get',
	'meta',
	'of',
	'set',
	'target',
])

export const AllKeywords = new Set([...StrictModeKeywords, ...AlwaysKeywords])
