export const INSPECTOR_STATE_KEY = 'rocci.preview.inspector'

export type DockSide = 'right' | 'bottom'
export type InspectorTab = 'performance' | 'source' | 'console'
export type InspectorView = 'source' | 'ast' | 'roc' | 'html'

export type InspectorPrefs = {
  open: boolean
  dock: DockSide
  right: string
  bottom: string
  tab: InspectorTab
  view: InspectorView
}

export type InspectorTuple = {
  origin: string
  path: string
  tab: InspectorTab
  route: string
}

const TABS: Record<string, InspectorTab> = {
  performance: 'performance',
  source: 'source',
  console: 'console'
}
const VIEWS: Record<string, InspectorView> = {
  source: 'source',
  ast: 'ast',
  roc: 'roc',
  html: 'html'
}
const DOCKS: Record<string, DockSide> = { right: 'right', bottom: 'bottom' }

export const DEFAULT_INSPECTOR_PREFS: InspectorPrefs = {
  open: false,
  dock: 'right',
  right: '28rem',
  bottom: '36vh',
  tab: 'performance',
  view: 'source'
}

export function normalizeRoute(value: string): string {
  let route = value || '/'
  try {
    route = decodeURIComponent(route)
  } catch {
    // keep raw
  }
  if (!route.startsWith('/')) {
    route = `/${route}`
  }
  if (route.length > 1 && !route.endsWith('/')) {
    route += '/'
  }
  return route
}

export function routeOf(url: string): string {
  return normalizeRoute(new URL(url).pathname)
}

export function inspectorTuple(inspectorUrl: string, pageUrl: string, tab: InspectorTab): InspectorTuple {
  const base = new URL(inspectorUrl)
  return {
    origin: base.origin,
    path: base.pathname,
    tab,
    route: routeOf(pageUrl)
  }
}

export function tuplesEqual(left: InspectorTuple | undefined, right: InspectorTuple): boolean {
  return Boolean(
    left &&
      left.origin === right.origin &&
      left.path === right.path &&
      left.tab === right.tab &&
      left.route === right.route
  )
}

export function inspectorHref(
  inspectorUrl: string,
  tuple: InspectorTuple,
  includeView: boolean,
  view: InspectorView
): string {
  const url = new URL(inspectorUrl)
  url.searchParams.set('tab', tuple.tab)
  url.searchParams.set('route', tuple.route)
  if (includeView) {
    url.searchParams.set('view', view)
  }
  return url.toString()
}

export function shouldAssignInspectorSrc(
  previous: InspectorTuple | undefined,
  next: InspectorTuple
): boolean {
  return !tuplesEqual(previous, next)
}

export function readInspectorPrefs(raw: unknown): InspectorPrefs {
  if (!raw || typeof raw !== 'object') {
    return { ...DEFAULT_INSPECTOR_PREFS }
  }
  const value = raw as Partial<InspectorPrefs>
  return {
    open: value.open === true,
    dock: DOCKS[value.dock ?? ''] ?? 'right',
    right: typeof value.right === 'string' && value.right ? value.right : '28rem',
    bottom: typeof value.bottom === 'string' && value.bottom ? value.bottom : '36vh',
    tab: TABS[value.tab ?? ''] ?? 'performance',
    view: VIEWS[value.view ?? ''] ?? 'source'
  }
}

export function applyInspectorMessage(
  prefs: InspectorPrefs,
  message: { tab?: unknown; view?: unknown }
): { prefs: InspectorPrefs; viewOnly: boolean } {
  const tab = typeof message.tab === 'string' ? TABS[message.tab] : undefined
  const view = typeof message.view === 'string' ? VIEWS[message.view] : undefined
  const next = { ...prefs }
  let viewOnly = true
  if (tab && tab !== prefs.tab) {
    next.tab = tab
    viewOnly = false
  }
  if (view) {
    next.view = view
  }
  return { prefs: next, viewOnly }
}

export function dockClassNames(prefs: InspectorPrefs, asPage: boolean): string {
  const classes = [`dock-${prefs.dock}`]
  if (prefs.open && !asPage) {
    classes.push('dev-open')
  }
  if (asPage) {
    classes.push('as-page')
  }
  return classes.join(' ')
}
