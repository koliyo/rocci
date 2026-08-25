export function canPreviewDocument(scheme: string, fsPath: string | undefined): boolean {
  return scheme === 'file' && Boolean(fsPath)
}

export type BrowserHost = 'simpleBrowser' | 'iframe'

export function chooseBrowserHost(hasSimpleBrowser: boolean): BrowserHost {
  return hasSimpleBrowser ? 'simpleBrowser' : 'iframe'
}

export function iframePreviewHtml(url: string): string {
  const escaped = url.replace(/&/g, '&amp;').replace(/"/g, '&quot;')
  return `<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8" />
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; frame-src http: https:; style-src 'unsafe-inline';" />
    <style>
      html, body, iframe { margin: 0; padding: 0; height: 100%; width: 100%; border: 0; }
    </style>
  </head>
  <body>
    <iframe src="${escaped}"></iframe>
  </body>
</html>`
}
