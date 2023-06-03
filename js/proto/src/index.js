import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { inspect } from 'node:util'
import { parseImportStatement } from './parser.js'

const dirname = path.dirname(fileURLToPath(import.meta.url))
const ENTRYPOINT = path.join(dirname, '../..', 'example/src/index.js')

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
