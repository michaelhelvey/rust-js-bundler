import { Source, TokenType, tokenize } from '../parser'

const makeSource = (source: string): Source => {
	return {
		code: source,
		fileName: 'script.js',
		sourceType: 'script',
	}
}

describe('tokenizer', () => {
	it('skips whitespace', () => {
		const source = `        `
		expect(tokenize(makeSource(source))).toEqual([])
	})

	it('can tokenize a single number', () => {
		const source = `1`
		expect(tokenize(makeSource(source))).toMatchObject([
			{
				type: TokenType.NUMBER_LITERAL,
				lexeme: '1',
			},
		])
	})

	it('can tokenize a single string', () => {
		const source = `'hello world'`
		expect(tokenize(makeSource(source))).toMatchObject([
			{
				type: TokenType.STRING_LITERAL,
				lexeme: `hello world`,
			},
		])
	})

	it('throws unexpected EOF when string is not closed', () => {
		const source = `'hello world`
		expect(() => tokenize(makeSource(source))).toThrowError(
			'Unexpected EOF while parsing string literal'
		)
	})

	it('can process an identifier', () => {
		const source = `hello`
		expect(tokenize(makeSource(source))).toMatchObject([
			{
				type: TokenType.IDENTIFIER,
				lexeme: 'hello',
			},
		])
	})

	it('can process a keyword', () => {
		const source = 'const'
		expect(tokenize(makeSource(source))).toMatchObject([
			{
				type: TokenType.KEYWORD,
				lexeme: 'const',
			},
		])
	})

	it('can process operators', () => {
		const source = '+'
		expect(tokenize(makeSource(source))).toMatchObject([
			{
				type: TokenType.OPERATOR,
				lexeme: '+',
			},
		])
	})
})
