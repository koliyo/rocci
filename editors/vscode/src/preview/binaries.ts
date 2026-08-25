import { ExtensionContext, workspace } from 'vscode'

import { resolveTool } from '../tools/resolve'
import { PreviewProduct } from './dispatch'

const isDebug = process.env.VSCODE_DEBUG_MODE !== undefined

export function resolvePreviewBinary(
  context: ExtensionContext,
  product: PreviewProduct
): string | undefined {
  const setting = product === 'rocci' ? 'preview.rocciPath' : 'preview.rocdownPath'
  const configured = workspace.getConfiguration('rocci').get<string>(setting)
  return resolveTool(context, product, configured, isDebug)
}
