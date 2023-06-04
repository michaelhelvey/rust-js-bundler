/**
 * EcmaScript Lexer
 *
 * @see https://www.ecma-international.org/ecma-262/11.0/index.html#sec-ecmascript-language-lexical-grammar
 */

/**
 * A trie structure for efficiently getting all tokens which could be produced
 * by a given prefix.  Used to avoid expensive lookaheads and N+1 queries in our
 * operator and punctuation token lexing steps.
 */
export class TokenTrie {
	private readonly dictionary = new Map<string, TokenTrie>()
	constructor(private value?: string) {}

	public static fromTokenMap<T extends Map<string, any>>(map: T) {
		const root = new TokenTrie()
		for (const key of map.keys()) {
			root.insert(key)
		}

		return root
	}

	public insert(key: string) {
		if (!key.length) {
			throw new Error('TokenTrie: cannot insert() token with length 0')
		}

		let index = 0
		let root: TokenTrie = this

		while (index < key.length) {
			root = root.getOrCreate(key[index])
			index++
		}

		root.value = key
	}

	/**
	 * Gets all values from the trie matching a given prefix.
	 */
	public get(prefix: string): string[] {
		if (!prefix.length) {
			throw new Error('TokenTrie: cannot get() token with length 0')
		}

		let index = 0
		let root: TokenTrie | undefined = this

		while (index < prefix.length && root) {
			root = root.dictionary.get(prefix[index])
			index++
		}

		// If we get to the end and we still have a root, then we have results:
		if (root) {
			return root.all()
		}

		// Otherwise if we traversed to the end of the string and there is no
		// entry, then we have no values to give
		return []
	}

	/**
	 * Recurses down the trie, returning all values:
	 */
	private all(): string[] {
		let results = this.value ? [this.value] : []

		for (const node of this.dictionary.values()) {
			results = results.concat(...node.all())
		}

		return results
	}

	private getOrCreate(char: string): TokenTrie {
		if (!this.dictionary.has(char)) {
			this.dictionary.set(char, new TokenTrie())
		}

		// Safety: the has() check above.
		return this.dictionary.get(char) as TokenTrie
	}
}

/**
 * Operators
 */

type OperatorDefinition = {
	lexeme: Symbol
	precedence: number
}

export const Operators = new Map<string, OperatorDefinition>([
	['=', { lexeme: Symbol('='), precedence: 0 }],
	['==', { lexeme: Symbol('=='), precedence: 0 }],
])

const operatorTrie = TokenTrie.fromTokenMap(Operators)

/**
 * Punctuators
 */

type PunctuationDefinition = {
	lexeme: Symbol
}

export const Puncutation = new Map<string, PunctuationDefinition>([
	['{', { lexeme: Symbol('{') }],
])

const punctuationTrie = TokenTrie.fromTokenMap(Puncutation)

/**
 * Keywords
 */

type KeywordDefinition = {
	// A unique symbol representing the keyword
	lexeme: Symbol
	// Whether or not we have to be in strict mode to disallow the token as an
	// identifier.
	strict: boolean
	// Whether it is an identifier which can be optionally used as a keyword in
	// certain productions (e.g. `async`)
	contextOnly: boolean
}

export const Keywords = new Map<string, KeywordDefinition>([
	['let', { lexeme: Symbol('let'), strict: true, contextOnly: false }],
	['const', { lexeme: Symbol('const'), strict: false, contextOnly: false }],
])

/**
 * Lexer Definition
 */

interface Position {
	line: number
	column: number
	index: number
}

interface Location {
	start: Position
	end: Position
	fileName?: string
}

export enum TokenType {
	Identifier = 'Identifier',
	Keyword = 'Keyword',
	String = 'String',
	Operator = 'Operator',
	Punctuation = 'Punctuation',
	Number = 'Number',
}

export type Token = {
	location: Location
} & (
	| {
			// The most generic "stringly typed" sort of identifier (e.g. a
			// string, number, or identifier)
			type: TokenType
			lexeme: string
	  }
	| {
			type: TokenType.Operator
			operator: OperatorDefinition
	  }
	| {
			type: TokenType.Keyword
			keyword: KeywordDefinition
	  }
)

export class Lexer {
	private position: Position = {
		line: 1,
		column: 1,
		index: 0,
	}
	constructor(private readonly source: { code: string; fileName?: string }) {}

	public tokenize(): Token[] {
		let tokens: Token[] = []
		const currentChar = () => this.source.code[this.position.index]

		while (currentChar()) {
			if (currentChar().match(/\s/)) {
				if (currentChar().match(/\n/)) {
					this.position.index++
					this.position.column = 1
					this.position.line++
					continue
				} else {
					this.getc()
					continue
				}
			}

			// Operator Tokenization:
			let operatorMatches = operatorTrie.get(currentChar())
			if (operatorMatches.length) {
				let lexeme = currentChar()

				// Add a character.  Is it still an operator?
				while (operatorMatches.length > 0) {
					this.getc()
					if (currentChar() === undefined) {
						break
					}

					lexeme = lexeme + currentChar()
					operatorMatches = operatorTrie.get(lexeme)
				}

				if (operatorMatches.length === 0) {
					// If we've "gone too far", go back one:
					this.ungetc()
					lexeme = lexeme.slice(0, lexeme.length - 1)
				}

				// At this point, this.position points to the last character in
				// the current token:
				tokens.push({
					type: TokenType.Operator,
					operator: Operators.get(lexeme)!,
					location: this.getLexemeLocation(lexeme),
				})

				// Now that we are done processing the lexeme, we can consume
				// the last character so that on the next loop currentChar()
				// will point to the next character to be processed.
				this.getc()
				continue
			}

			throw new Error(`Unhandled character ${currentChar()}`)
		}

		return tokens
	}

	private getc() {
		this.position.index++
		this.position.column++
	}

	private ungetc() {
		this.position.index--
		this.position.column--
	}

	private getLexemeLocation(lexeme: string) {
		// Assumes that this.position points at the last character of the
		// current lexeme:
		return {
			start: {
				line: this.position.line,
				column: this.position.column - (lexeme.length - 1),
				index: this.position.index - (lexeme.length - 1),
			},
			end: {
				line: this.position.line,
				column: this.position.column + 1,
				index: this.position.index + 1,
			},
		}
	}
}
