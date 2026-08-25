import * as fs from 'fs'
import * as path from 'path'

import { PreviewProduct } from './dispatch'

export interface PreviewOrigin {
  product: PreviewProduct
  root: string
}

export function previewOrigin(filePath: string): PreviewOrigin | undefined {
  const ext = path.extname(filePath).toLowerCase()
  if (ext === '.rocci') {
    return { product: 'rocci', root: findConfigRoot(filePath, 'rocci.toml') ?? filePath }
  }
  if (ext === '.rocdown') {
    return { product: 'rocdown', root: findConfigRoot(filePath, 'rocdown.toml') ?? filePath }
  }
  return undefined
}

export function sameOrigin(a: PreviewOrigin, b: PreviewOrigin): boolean {
  return a.product === b.product && resolvePath(a.root) === resolvePath(b.root)
}

export function belongsToOrigin(filePath: string, origin: PreviewOrigin): boolean {
  const next = previewOrigin(filePath)
  if (next && sameOrigin(next, origin)) {
    return true
  }
  const file = resolvePath(filePath)
  const root = resolvePath(origin.root)
  if (fs.existsSync(root) && fs.statSync(root).isFile()) {
    return path.dirname(file) === path.dirname(root)
  }
  return file === root || file.startsWith(root + path.sep)
}

function resolvePath(value: string): string {
  try {
    return fs.realpathSync(value)
  } catch {
    return path.resolve(value)
  }
}

export function reuseDecision(
  current: PreviewOrigin | undefined,
  next: PreviewOrigin
): 'start' | 'reuse' | 'restart' {
  if (!current) {
    return 'start'
  }
  return sameOrigin(current, next) ? 'reuse' : 'restart'
}

export function navigateUrl(currentUrl: string, filePath: string, origin: PreviewOrigin): string {
  const parsed = new URL(currentUrl)
  const base = `${parsed.protocol}//${parsed.host}`
  if (origin.product === 'rocci' || origin.root === filePath) {
    return `${base}${parsed.pathname || '/'}`
  }
  return `${base}${derivedSiteRoute(origin.root, filePath)}`
}

function derivedSiteRoute(root: string, filePath: string): string {
  const rel = path.relative(root, filePath).replace(/\\/g, '/')
  const withoutExt = rel.replace(/\.(rocdown|md|markdown)$/i, '')
  if (!withoutExt || withoutExt === 'index') {
    return '/'
  }
  return `/${withoutExt}/`
}

function findConfigRoot(filePath: string, configName: string): string | undefined {
  let dir = path.dirname(filePath)
  while (true) {
    if (fs.existsSync(path.join(dir, configName)) && !isProjectBoundary(dir)) {
      return dir
    }
    if (isProjectBoundary(dir)) {
      return undefined
    }
    const parent = path.dirname(dir)
    if (parent === dir) {
      return undefined
    }
    dir = parent
  }
}

function isProjectBoundary(dir: string): boolean {
  return fs.existsSync(path.join(dir, '.git'))
}
