import * as path from 'path'

export type PreviewProduct = 'rocci' | 'rocdown'

export interface PreviewArgv {
  product: PreviewProduct
  args: string[]
}

export function previewArgv(filePath: string): PreviewArgv | undefined {
  const ext = path.extname(filePath).toLowerCase()
  if (ext === '.rocdown') {
    return {
      product: 'rocdown',
      args: ['view', '--no-window', '--port', 'auto', '--verbose', filePath]
    }
  }
  if (ext === '.rocci') {
    return {
      product: 'rocci',
      args: ['run', '--no-window', '--port', 'auto', '--verbose', filePath]
    }
  }
  return undefined
}
