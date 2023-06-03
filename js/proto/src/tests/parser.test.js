import assert from 'node:assert'
import { describe, it } from 'node:test'
import { parseImportStatement, tokenizeImportStatement } from '../parser.js'

describe('tokenizeImportStatement', () => {
	it('can tokenize a named import with a single local', () => {
		const stmt = `import {kiwi} from "./kiwi.js";`
		const tokens = tokenizeImportStatement(stmt)

		assert.deepEqual(tokens, [
			{ lexeme: 'import', type: 'keyword' },
			{ lexeme: '{', type: 'opening_brace' },
			{ lexeme: 'kiwi', type: 'identifier' },
			{ lexeme: '}', type: 'closing_brace' },
			{ lexeme: 'from', type: 'keyword' },
			{ lexeme: './kiwi.js', type: 'string' },
			{ lexeme: ';', type: 'semicolon' },
		])
	})

	it('should tokenize a named import with multiple locals', () => {
		const stmt = `import {kiwi, apple, banana} from "./kiwi.js";`
		const tokens = tokenizeImportStatement(stmt)

		assert.deepEqual(tokens, [
			{ lexeme: 'import', type: 'keyword' },
			{ lexeme: '{', type: 'opening_brace' },
			{ lexeme: 'kiwi', type: 'identifier' },
			{ lexeme: ',', type: 'comma' },
			{ lexeme: 'apple', type: 'identifier' },
			{ lexeme: ',', type: 'comma' },
			{ lexeme: 'banana', type: 'identifier' },
			{ lexeme: '}', type: 'closing_brace' },
			{ lexeme: 'from', type: 'keyword' },
			{ lexeme: './kiwi.js', type: 'string' },
			{ lexeme: ';', type: 'semicolon' },
		])
	})

	it('should throw if it encounter an unexpected token', () => {
		const stmt = `import & from './kiwi.js';`
		assert.throws(
			() => tokenizeImportStatement(stmt),
			`Unexpected token '&' in statement "import & from './kiwi.js'"; at position 8`
		)
	})
})

describe('parseImportStatment', () => {
	it('parses a named import with a single local', () => {
		const source = `import { kiwi } from "./kiwi.js";`
		assert.deepEqual(parseImportStatement(source), {
			type: 'ImportDeclaration',
			specifiers: [
				{
					type: 'ImportSpecifier',
					imported: 'kiwi',
					local: 'kiwi',
				},
			],
			source: './kiwi.js',
		})
	})

	it('parses a named import with multiple locals', () => {
		// Note: `import { default as foo }` is parsed in the same way.  The
		// "default" only has a special meaning at runtime; it's not expected for
		// parsers to understand the semantics of the word "default"
		const source = `import { kiwi as foulTastingGarbage, apple } from "./kiwi.js";`
		assert.deepEqual(parseImportStatement(source), {
			type: 'ImportDeclaration',
			specifiers: [
				{
					type: 'ImportSpecifier',
					imported: 'kiwi',
					local: 'foulTastingGarbage',
				},
				{
					type: 'ImportSpecifier',
					imported: 'apple',
					local: 'apple',
				},
			],
			source: './kiwi.js',
		})
	})

	it('parses a default import', () => {
		const source = `import kiwi from "./kiwi.js";`
		assert.deepEqual(parseImportStatement(source), {
			type: 'ImportDeclaration',
			specifiers: [
				{
					type: 'ImportDefaultSpecifier',
					local: 'kiwi',
				},
			],
			source: './kiwi.js',
		})
	})

	it('parses a namespace import', () => {
		const source = `import * as kiwi from "./kiwi.js";`
		assert.deepEqual(parseImportStatement(source), {
			type: 'ImportDeclaration',
			specifiers: [
				{
					type: 'ImportNamespaceSpecifier',
					local: 'kiwi',
				},
			],
			source: './kiwi.js',
		})
	})
})
