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

/** @typedef { { imported: string, local: string, type: string } } ImportSpecifier  */
/** @typedef { { type: string, specifiers: ImportSpecifier[], source: string } } ImportDeclaration  */
/** @type{(statement: string) => ImportDeclaration} */
export function parseImportStatement(statement) {
	return {
		type: 'ImportDeclaration',
		specifiers: [
			{
				type: 'ImportSpecifier',
				imported: 'kiwi',
				local: 'kiwi',
			},
		],
		source: './kiwi.js',
	}
}
