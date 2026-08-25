const ANSI = /\u001b\[[0-9;]*[A-Za-z]/g
const LOOPBACK =
  /https?:\/\/127\.0\.0\.1(?::\d+)?(?:\/[^\s]*)?/
const PREVIEW_READY = /^preview_ready\s+(https?:\/\/[^\s]+)\s*$/m
const INSPECTOR_READY = /^inspector_ready\s+(https?:\/\/[^\s]+)\s*$/m

export function stripAnsi(text: string): string {
  return text.replace(ANSI, '')
}

export function parsePreviewUrl(text: string): string | undefined {
  const plain = stripAnsi(text)
  const ready = plain.match(PREVIEW_READY)
  if (ready?.[1]) {
    return ready[1]
  }
  return plain.match(LOOPBACK)?.[0]
}

export function parseInspectorUrl(text: string): string | undefined {
  const ready = stripAnsi(text).match(INSPECTOR_READY)
  return ready?.[1]
}
