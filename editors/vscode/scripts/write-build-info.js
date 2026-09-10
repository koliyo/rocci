const { execFileSync } = require('child_process')
const fs = require('fs')
const path = require('path')

const vscodeRoot = path.join(__dirname, '..')
const repoRoot = path.join(vscodeRoot, '..', '..')
let git = 'unknown'
try {
  git = execFileSync('git', ['-C', repoRoot, 'rev-parse', '--short', 'HEAD'], {
    encoding: 'utf8'
  }).trim()
} catch {
  git = 'unknown'
}

const payload = {
  git,
  builtAt: new Date().toISOString()
}
fs.writeFileSync(path.join(vscodeRoot, 'build-info.json'), `${JSON.stringify(payload, null, 2)}\n`)
