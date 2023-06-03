import { parse } from '@babel/parser'
import { inspect } from 'util'

const source = `
export function foo() {
  return 'foo'
}
`

const ast = parse(source, {
	sourceType: 'module',
	strictMode: true,
})

console.log(inspect(ast, { depth: null }))
