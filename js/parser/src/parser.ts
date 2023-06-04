// This TokenType thing is incredibly naive, because realistically we're going
// to need something that interns strings for keywords to improve equality
// checking (e.g. keywords should probably be Symbols), and then we have other
// kinds of things where (for example operators) we need to be able to tell what
// kind of operator something is. Same with numbers.  Or else parsing is going
// to be a pain.  So long-term this is going to need to be a runtime construct
// rather than an enum.  But this will get us to some simple expressions before

import { AllKeywords } from './keywords'

// we refactor.
export enum TokenType {
	KEYWORD = 'KEYWORD',
	STRING_LITERAL = 'STRING_LITERAL',
	TEMPLATE_LITERAL = 'TEMPLATE_LITERAL',
	// Not even bothering to deal with different number literal types for now.  No
	// floats for you, nerd.
	NUMBER_LITERAL = 'NUMBER_LITERAL',
	NULL_LITERAL = 'NULL_LITERAL',
	BOOLEAN_LITERAL = 'BOOLEAN_LITERAL',
	IDENTIFIER = 'IDENTIFIER',
	DIRECTIVE = 'DIRECTIVE',
	// All operators are equal :D
	OPERATOR = 'OPERATOR',
	// Puncuation
	PERIOD = 'PERIOD',
	COMMA = 'COMMA',
	OPEN_PAREN = 'OPEN_PAREN',
	CLOSE_PAREN = 'CLOSE_PAREN',
	OPEN_BRACE = 'OPEN_BRACE',
	CLOSE_BRACE = 'CLOSE_BRACE',
	OPEN_BRACKET = 'OPEN_BRACKET',
	CLOSE_BRACKET = 'CLOSE_BRACKET',
	SEMICOLON = 'SEMICOLON',
	COLON = 'COLON',
}

interface Position {
	line: number
	column: number
	index: number
}

interface Location {
	start: Position
	end: Position
	fileName?: string
	identifierName?: string
}

export interface Token {
	type: TokenType
	lexeme: string
	location: Location
}

export interface Source {
	code: string
	fileName?: string
	sourceType: 'script' | 'module'
}

export function tokenize(source: Source): Token[] {
	let index = 0
	let line = 1
	let column = 0

	const tokens: Token[] = []

	while (index < source.code.length) {
		if (!source.code[index]) {
			break
		}

		function pushToken(type: TokenType, lexeme: string) {
			tokens.push({
				type,
				lexeme,
				location: {
					start: {
						line,
						column,
						index,
					},
					end: {
						line,
						column: column + lexeme.length,
						index: index + lexeme.length,
					},
				},
			})
		}

		function advanceChar() {
			index++
			column++
		}

		const notEof = () => index < source.code.length

		if (source.code[index].match(/\s/)) {
			// Should match both \r\n and \n:
			if (source.code[index].match(/[\n]+/)) {
				line++
				column = 0
			} else {
				advanceChar()
			}

			continue
		}

		// Only handles integers for now:
		if (source.code[index].match(/[0-9]/)) {
			let lexeme = ''
			while (notEof() && source.code[index].match(/[0-9]/)) {
				lexeme += source.code[index]
				advanceChar()
			}

			pushToken(TokenType.NUMBER_LITERAL, lexeme)
			continue
		}

		if (source.code[index].match(/['"']/)) {
			let lexeme = ''
			// Eat initial delimeter
			advanceChar()

			while (notEof() && !source.code[index].match(/['"]/)) {
				lexeme += source.code[index]
				advanceChar()
			}

			if (!source.code[index]) {
				throw new Error('Unexpected EOF while parsing string literal')
			}

			// Eat final delimeter
			advanceChar()
			pushToken(TokenType.STRING_LITERAL, lexeme)

			continue
		}

		if (source.code[index].match(/[a-zA-Z$_]/)) {
			let lexeme = ''

			while (notEof() && source.code[index].match(/[a-zA-Z0-9$_]/)) {
				lexeme += source.code[index]
				advanceChar()
			}

			if (AllKeywords.has(lexeme)) {
				pushToken(TokenType.KEYWORD, lexeme)
			} else {
				pushToken(TokenType.IDENTIFIER, lexeme)
			}

			continue
		}

		// end of loop: if we have not processed a character, yet, throw:
		throw new Error(`Unhandled character: ${source.code[index]}`)
	}

	return tokens
}
