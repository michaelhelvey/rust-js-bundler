import path from 'node:path'

const __dirname = path.dirname(new URL(import.meta.url).pathname)
const ENTRYPOINT = path.join(__dirname, '../..', 'example/src/index.js')

console.log(ENTRYPOINT)
