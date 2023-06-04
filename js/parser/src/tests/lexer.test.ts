import { Lexer, Operators, TokenTrie, TokenType } from '../lexer'

const makeSource = (s: string) => ({ code: s })

describe('lexer', () => {
	it('returns no tokens for an empty source', () => {
		const source = makeSource('')
		const lexer = new Lexer(source)

		expect(lexer.tokenize()).toEqual([])
	})

	it('can tokenize a single-character operator', () => {
		const source = makeSource(`=`)
		const lexer = new Lexer(source)

		expect(lexer.tokenize()).toMatchObject([
			{
				type: TokenType.Operator,
				operator: Operators.get('='),
			},
		])
	})

	it('can tokenize a multi-character operator', () => {
		const source = makeSource(`==`)
		const lexer = new Lexer(source)

		expect(lexer.tokenize()).toMatchObject([
			{
				type: TokenType.Operator,
				operator: Operators.get('=='),
			},
		])
	})

	it('handles trailing characters', () => {
		const source = makeSource(`=`)
		const lexer = new Lexer(source)

		expect(lexer.tokenize()).toMatchObject([
			{
				type: TokenType.Operator,
				operator: Operators.get('='),
			},
		])
	})

	it('can parse multiple operators', () => {
		const source = makeSource(`= ==`)
		const lexer = new Lexer(source)

		expect(lexer.tokenize()).toMatchObject([
			{
				type: TokenType.Operator,
				operator: Operators.get('='),
			},
			{
				type: TokenType.Operator,
				operator: Operators.get('=='),
			},
		])
	})

	it('handles new lines and marks character positions', () => {
		const source = makeSource(
			`
=
	==
			`
		)
		const lexer = new Lexer(source)

		expect(lexer.tokenize()).toMatchObject([
			{
				type: TokenType.Operator,
				operator: Operators.get('='),
				location: {
					start: {
						line: 2,
						column: 1,
						index: 1,
					},
					end: {
						line: 2,
						column: 2,
						index: 2,
					},
				},
			},
			{
				type: TokenType.Operator,
				operator: Operators.get('=='),
				location: {
					start: {
						line: 3,
						column: 2,
						index: 4,
					},
					end: {
						line: 3,
						column: 4,
						index: 6,
					},
				},
			},
		])
	})
})

describe('TokenTrie', () => {
	it('can insert and retrieve a new token', () => {
		const trie = new TokenTrie()
		trie.insert('asdf')
		expect(trie.get('a')).toEqual(['asdf'])
	})

	it('can retrieve tokens for multiple prefixes', () => {
		const trie = new TokenTrie()
		trie.insert('a')
		trie.insert('as')
		trie.insert('ass')
		expect(trie.get('as')).toEqual(['as', 'ass'])
	})

	it('returns an empty array for token not found', () => {
		const trie = new TokenTrie()
		trie.insert('>')
		expect(trie.get('>>')).toEqual([])
	})
})
