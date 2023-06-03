/**
 * @fileoverview Parser for ESM import statements.
 *
 * @see https://tc39.es/ecma262/#sec-modules
 */

const keywords = new Set(['import', 'from', 'as'])
// Note: javascript identifiers can _contain_ digits, but cannot _start_
// with a digit.  We're not enforcing that here: we can let the bundler's
// parser be a little naive since we're not planning to actually execute the
// code.
const IDENTIFIER_TOKEN = /[\w_$\d]/
const WHITESPACE_TOKEN = /\s/
// Note: '`' is not valid in a JS import statement, though it is a valid string
// delimiter in general.
const STRING_DELIMITER = /['"]/

/** @typedef {{ lexeme: string, type: string }} Token */
/** @type {(statement: string) => Token[]} */
export function tokenizeImportStatement(statement) {
	let position = 0
	let tokens = []

	while (position < statement.length) {
		// Skip whitespace
		if (statement[position].match(WHITESPACE_TOKEN)) {
			position++
			continue
		}

		if (statement[position].match(IDENTIFIER_TOKEN)) {
			let lexeme = ''
			while (statement[position].match(IDENTIFIER_TOKEN)) {
				lexeme += statement[position]
				position++
			}

			if (keywords.has(lexeme)) {
				tokens.push({ lexeme, type: 'keyword' })
			} else {
				tokens.push({ lexeme, type: 'identifier' })
			}

			continue
		}

		if (statement[position].match(STRING_DELIMITER)) {
			let lexeme = ''
			// eat the opening delimiter
			position++
			while (!statement[position].match(STRING_DELIMITER)) {
				lexeme += statement[position]
				position++
			}

			tokens.push({ lexeme, type: 'string' })
			// eat the closing delimiter
			position++
			continue
		}

		// Match the single-character tokens:
		switch (statement[position]) {
			case '{':
				tokens.push({ lexeme: '{', type: 'opening_brace' })
				position++
				continue
			case '}':
				tokens.push({ lexeme: '}', type: 'closing_brace' })
				position++
				continue
			case ',':
				tokens.push({ lexeme: ',', type: 'comma' })
				position++
				continue
			case ';':
				tokens.push({ lexeme: ';', type: 'semicolon' })
				position++
				continue
			case '*':
				tokens.push({ lexeme: '*', type: 'star' })
				position++
				continue
			default:
				throw new Error(
					`Unexpected token '${
						statement[position]
					}' in statement ${statement} at position ${position + 1}`
				)
		}
	}

	return tokens
}

/** @type {(tokens: Token[], type: string, matcher?: (lexeme: string) => boolean) => Token} */
function match(tokens, type, matcher) {
	if (tokens[0].type === type) {
		if (matcher && !matcher(tokens[0].lexeme)) {
			throw new Error(`Unexpected token '${tokens[0].lexeme}'`)
		}

		return tokens.shift()
	}

	throw new Error(`Unexpected token '${tokens[0].lexeme}', expected ${type}`)
}

/** @type {(tokens: Token[]) => Token} */
function peek(tokens) {
	return tokens[0]
}

/** @typedef { { imported?: string, local?: string, type: string } } ImportSpecifier  */
class ImportDeclaration {
	constructor() {
		this.type = 'ImportDeclaration'
		/** @type {ImportSpecifier[]} */
		this.specifiers = []
		this.source = ''
	}

	toDict() {
		return {
			type: this.type,
			specifiers: this.specifiers,
			source: this.source,
		}
	}
}

/**
 * Parses a namespace import specifier. e.g. the `* as fruit` in `import * as
 * fruit from './kiwi.js'`
 *
 * @param {Token[]} tokens
 * @returns {ImportSpecifier}
 */
function parseNamespaceImportSpecifier(tokens) {
	match(tokens, 'star')
	match(tokens, 'keyword', lexeme => lexeme === 'as')
	const identifier = match(tokens, 'identifier')
	return {
		type: 'ImportNamespaceSpecifier',
		local: identifier.lexeme,
	}
}

/**
 * Parses a single named import specifier. e.g. the `kiwi as fruit` in `import {
 * kiwi as fruit } from './kiwi.js'`
 *
 * @param {Token[]} tokens
 * @returns {ImportSpecifier}
 */
function parseNamedImportSpecifier(tokens) {
	const imported = match(tokens, 'identifier')
	let local = imported

	if (peek(tokens).type === 'keyword' && peek(tokens).lexeme === 'as') {
		match(tokens, 'keyword', lexeme => lexeme === 'as')
		local = match(tokens, 'identifier')
	}

	return {
		type: 'ImportSpecifier',
		imported: imported.lexeme,
		local: local.lexeme,
	}
}

/**
 * Parses a default import specifier. e.g. the `kiwi` in `import kiwi from from
 * './kiwi.js'`
 *
 * @param {Token[]} tokens
 * @returns {ImportSpecifier}
 */
function parseDefaultImportSpecifier(tokens) {
	const identifier = match(tokens, 'identifier')
	return {
		type: 'ImportDefaultSpecifier',
		local: identifier.lexeme,
	}
}

/**
 * Parses a comma-separated list of named import specifiers.
 *
 * @param {Token[]} tokens
 * @returns {ImportSpecifier[]}
 */
function parseNamedImports(tokens) {
	/** @type {ImportSpecifier[]} */
	let specifiers = []

	match(tokens, 'opening_brace')
	while (peek(tokens).type !== 'closing_brace') {
		/** @type {Partial<ImportSpecifier>} */
		specifiers.push(parseNamedImportSpecifier(tokens))

		if (peek(tokens).type === 'comma') {
			match(tokens, 'comma')
		}
	}

	match(tokens, 'closing_brace')

	return specifiers
}

/** @type {(tokens: Token[]) => ImportDeclaration} */
export function parser(tokens) {
	const declaration = new ImportDeclaration()
	match(tokens, 'keyword', lexeme => lexeme === 'import')

	if (peek(tokens).type === 'star') {
		declaration.specifiers.push(parseNamespaceImportSpecifier(tokens))
	} else if (peek(tokens).type === 'opening_brace') {
		declaration.specifiers.push(...parseNamedImports(tokens))
	} else if (peek(tokens).type === 'identifier') {
		declaration.specifiers.push(parseDefaultImportSpecifier(tokens))
	}

	match(tokens, 'keyword', lexeme => lexeme === 'from')
	declaration.source = match(tokens, 'string').lexeme

	return declaration
}

/** @type{(statement: string) => ImportDeclaration} */
export function parseImportStatement(statement) {
	const tokens = tokenizeImportStatement(statement)
	return parser(tokens)
}
