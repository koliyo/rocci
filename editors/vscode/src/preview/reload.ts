const NONCE_PARAM = '_r'

export function previewOriginUrl(url: string): string {
  const parsed = new URL(url)
  return `${parsed.protocol}//${parsed.host}`
}

export function liveReloadEventsUrl(previewUrl: string): string {
  return `${previewOriginUrl(previewUrl)}/__rocci/events`
}

export function withReloadNonce(url: string, nonce: number): string {
  const parsed = new URL(url)
  parsed.searchParams.set(NONCE_PARAM, String(nonce))
  return parsed.toString()
}

export function hasSseReloadEvent(buffer: string): boolean {
  return /(?:^|\n)event:\s*reload(?:\n|$)/.test(buffer)
}

export function countPreviewReadyLines(text: string): number {
  const matches = text.match(/^preview_ready\s+https?:\/\/\S+/gm)
  return matches?.length ?? 0
}
