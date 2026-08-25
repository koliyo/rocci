import * as fs from 'fs'
import * as path from 'path'
import Mocha from 'mocha'

function collectTestFiles(dir: string): string[] {
  const results: string[] = []
  const list = fs.readdirSync(dir)
  for (const file of list) {
    const filePath = path.join(dir, file)
    const stat = fs.statSync(filePath)
    if (stat.isDirectory()) {
      results.push(...collectTestFiles(filePath))
    } else if (file.endsWith('.test.js') && !filePath.includes(`${path.sep}unit${path.sep}`)) {
      results.push(filePath)
    }
  }
  return results
}

export function run(): Promise<void> {
  const mocha = new Mocha({
    ui: 'tdd',
    color: true,
    timeout: 30000
  })

  const testsRoot = path.resolve(__dirname, '..')

  return new Promise((resolve, reject) => {
    try {
      const files = collectTestFiles(testsRoot)
      for (const f of files) {
        mocha.addFile(f)
      }

      mocha.run(failures => {
        if (failures > 0) {
          reject(new Error(`${failures} tests failed.`))
        } else {
          resolve()
        }
      })
    } catch (err) {
      reject(err)
    }
  })
}
