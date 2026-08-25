import * as http from 'http'

import { hasSseReloadEvent, liveReloadEventsUrl } from './reload'

export class PreviewReloadStream {
  private request: http.ClientRequest | undefined
  private closed = false
  private timer: NodeJS.Timeout | undefined
  private url: string | undefined

  constructor(
    private readonly onReload: () => void,
    private readonly log: (message: string) => void
  ) {}

  start(previewUrl: string): void {
    this.stop()
    this.closed = false
    this.url = liveReloadEventsUrl(previewUrl)
    this.log(`watch ${this.url}`)
    this.connect(this.url)
  }

  stop(): void {
    this.closed = true
    this.url = undefined
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
        if (res.statusCode === 404) {
          this.log(`watch missing ${url}`)
          res.resume()
          return
        }
        if (res.statusCode !== 200) {
          this.log(`watch ${res.statusCode} ${url}`)
          res.resume()
          this.scheduleReconnect(url)
          return
        }
        this.log('watch connected')
        let buffer = ''
        res.setEncoding('utf8')
        res.on('data', (chunk: string) => {
          buffer += chunk
          if (hasSseReloadEvent(buffer)) {
            buffer = ''
            this.log('watch reload')
            this.onReload()
          }
          if (buffer.length > 16_384) {
            buffer = buffer.slice(-2048)
          }
        })
        res.on('end', () => {
          this.log('watch ended')
          this.scheduleReconnect(url)
        })
      }
    )
    req.on('error', err => {
      this.log(`watch error ${err.message}`)
      this.scheduleReconnect(url)
    })
    this.request = req
  }

  private scheduleReconnect(url: string): void {
    if (this.closed) {
      return
    }
    this.timer = setTimeout(() => this.connect(url), 1000)
  }
}
