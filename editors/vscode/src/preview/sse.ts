import * as http from 'http'

import { hasSseReloadEvent, liveReloadEventsUrl } from './reload'

export class PreviewReloadStream {
  private request: http.ClientRequest | undefined
  private closed = false
  private timer: NodeJS.Timeout | undefined

  constructor(private readonly onReload: () => void) {}

  start(previewUrl: string): void {
    this.stop()
    this.closed = false
    this.connect(liveReloadEventsUrl(previewUrl))
  }

  stop(): void {
    this.closed = true
    if (this.timer) {
      clearTimeout(this.timer)
      this.timer = undefined
    }
    this.request?.destroy()
    this.request = undefined
  }

  private connect(url: string): void {
    if (this.closed) {
      return
    }
    const parsed = new URL(url)
    const req = http.get(
      {
        hostname: parsed.hostname,
        port: parsed.port,
        path: parsed.pathname,
        headers: { Accept: 'text/event-stream' }
      },
      res => {
        if (res.statusCode !== 200) {
          res.resume()
          this.scheduleReconnect(url)
          return
        }
        let buffer = ''
        res.setEncoding('utf8')
        res.on('data', (chunk: string) => {
          buffer += chunk
          if (hasSseReloadEvent(buffer)) {
            buffer = ''
            this.onReload()
          }
          if (buffer.length > 16_384) {
            buffer = buffer.slice(-2048)
          }
        })
        res.on('end', () => this.scheduleReconnect(url))
      }
    )
    req.on('error', () => this.scheduleReconnect(url))
    this.request = req
  }

  private scheduleReconnect(url: string): void {
    if (this.closed) {
      return
    }
    this.timer = setTimeout(() => this.connect(url), 1000)
  }
}
