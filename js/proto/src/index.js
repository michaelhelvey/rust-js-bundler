import fs from 'node:fs/promises'
import path from 'node:path'
import { inspect } from 'node:util'
import { parseImportStatement } from './parser.js'

const __dirname = path.dirname(new URL(import.meta.url).pathname)
const ENTRYPOINT = path.join(__dirname, '../..', 'example/src/index.js')

async function main() {
	const fileContents = await fs.readFile(ENTRYPOINT, 'utf8')
	const imports = Array.from(fileContents.matchAll(/import.*from.*/g)).map(
		match => match[0]
	)

	const parsed = imports.map(parseImportStatement)
	console.log(inspect(parsed, { depth: null }))
}

if (import.meta.url.endsWith(process.argv[1])) {
	await main()
}
