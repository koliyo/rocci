const COLS = 10
const ROWS = 20
const COLORS = {
    I: '#6ee7f9',
    J: '#60a5fa',
    L: '#fb923c',
    O: '#facc15',
    S: '#4ade80',
    Z: '#f87171',
    T: '#c084fc',
    G: '#64748b',
    '.': '#0b1f18',
}

const OFFSETS = {
    I: [
        [[0, 1], [1, 1], [2, 1], [3, 1]],
        [[2, 0], [2, 1], [2, 2], [2, 3]],
        [[0, 2], [1, 2], [2, 2], [3, 2]],
        [[1, 0], [1, 1], [1, 2], [1, 3]],
    ],
    O: [
        [[1, 0], [2, 0], [1, 1], [2, 1]],
        [[1, 0], [2, 0], [1, 1], [2, 1]],
        [[1, 0], [2, 0], [1, 1], [2, 1]],
        [[1, 0], [2, 0], [1, 1], [2, 1]],
    ],
    T: [
        [[1, 0], [0, 1], [1, 1], [2, 1]],
        [[1, 0], [1, 1], [2, 1], [1, 2]],
        [[0, 1], [1, 1], [2, 1], [1, 2]],
        [[1, 0], [0, 1], [1, 1], [1, 2]],
    ],
    S: [
        [[1, 0], [2, 0], [0, 1], [1, 1]],
        [[1, 0], [1, 1], [2, 1], [2, 2]],
        [[1, 1], [2, 1], [0, 2], [1, 2]],
        [[0, 0], [0, 1], [1, 1], [1, 2]],
    ],
    Z: [
        [[0, 0], [1, 0], [1, 1], [2, 1]],
        [[2, 0], [1, 1], [2, 1], [1, 2]],
        [[0, 1], [1, 1], [1, 2], [2, 2]],
        [[1, 0], [0, 1], [1, 1], [0, 2]],
    ],
    J: [
        [[0, 0], [0, 1], [1, 1], [2, 1]],
        [[1, 0], [2, 0], [1, 1], [1, 2]],
        [[0, 1], [1, 1], [2, 1], [2, 2]],
        [[1, 0], [1, 1], [0, 2], [1, 2]],
    ],
    L: [
        [[2, 0], [0, 1], [1, 1], [2, 1]],
        [[1, 0], [1, 1], [1, 2], [2, 2]],
        [[0, 1], [1, 1], [2, 1], [0, 2]],
        [[0, 0], [1, 0], [1, 1], [1, 2]],
    ],
}

const KICKS = [
    [0, 0],
    [-1, 0],
    [1, 0],
    [0, 1],
    [-1, 1],
    [1, 1],
    [0, -1],
]

let sequence = 0
let revision = 0
let board = '.'.repeat(200)
let current = { piece: 'T', rot: 0, x: 3, y: 0 }
let locking = false
let eliminated = false
let gravityMs = 800
let gravityTimer = 0
let das = { dir: 0, delay: 0 }
let lastTs = 0
let gamepadPrev = { left: false, right: false, down: false, rot: false, ccw: false, drop: false }

function cellsOf(piece, rot, x, y) {
    const shape = OFFSETS[piece]?.[rot]
    if (!shape) {
        return null
    }
    const cells = shape.map(([dx, dy]) => ({ x: x + dx, y: y + dy }))
    if (cells.some((c) => c.x < 0 || c.x >= COLS || c.y < 0 || c.y >= ROWS)) {
        return null
    }
    return cells
}

function occupied(x, y) {
    if (y < 0) {
        return false
    }
    if (x < 0 || x >= COLS || y >= ROWS) {
        return true
    }
    return board[y * COLS + x] !== '.'
}

function fits(piece, rot, x, y) {
    const cells = cellsOf(piece, rot, x, y)
    return Boolean(cells) && cells.every((c) => !occupied(c.x, c.y))
}

function kick(piece, rot, x, y) {
    for (const [dx, dy] of KICKS) {
        if (fits(piece, rot, x + dx, y + dy)) {
            return { x: x + dx, y: y + dy, rot }
        }
    }
    return null
}

function ghostY() {
    let y = current.y
    while (fits(current.piece, current.rot, current.x, y + 1)) {
        y += 1
    }
    return y
}

function readManifest() {
    const root = document.getElementById('blocks-arena-state')
    const you = root?.querySelector('[data-you="1"]') ?? root?.querySelector('[data-seat]')
    if (!root || !you) {
        return
    }
    revision = Number(root.getAttribute('data-revision') ?? '0')
    board = you.getAttribute('data-board') ?? board
    eliminated = you.getAttribute('data-status') === 'eliminated'
    const piece = you.getAttribute('data-piece')
    if (piece && piece !== current.piece && !locking) {
        current = { piece, rot: 0, x: 3, y: 0 }
    }
}

function applyAck(data) {
    const ok = data.ok === 1 || data.ok === true
    board = data.board ?? board
    revision = data.revision ?? revision
    sequence = data.sequence ?? sequence
    eliminated = data.eliminated === 1 || data.eliminated === true
    current = { piece: data.piece ?? current.piece, rot: 0, x: 3, y: 0 }
    locking = false
    const you = document.querySelector('#blocks-arena-state [data-you="1"]')
    if (you) {
        you.setAttribute('data-board', board)
        you.setAttribute('data-piece', current.piece)
        you.setAttribute('data-status', eliminated ? 'eliminated' : 'alive')
    }
    if (!ok) {
        current = { piece: data.piece ?? current.piece, rot: 0, x: 3, y: 0 }
    }
}

async function postLock() {
    if (locking || eliminated) {
        return
    }
    locking = true
    const body = {
        piece: current.piece,
        rotation: current.rot,
        x: current.x,
        y: current.y,
        board_revision: revision,
        sequence: sequence + 1,
    }
    try {
        const response = await fetch('/command/lock', {
            method: 'POST',
            credentials: 'same-origin',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify(body),
        })
        const data = await response.json()
        applyAck(data)
    } catch {
        locking = false
    }
}

async function resetBoard() {
    const response = await fetch('/command/reset', {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'content-type': 'application/json' },
        body: '{}',
    })
    const data = await response.json()
    sequence = 0
    applyAck(data)
}

function tryMove(dx, dy) {
    if (locking || eliminated) {
        return false
    }
    if (fits(current.piece, current.rot, current.x + dx, current.y + dy)) {
        current = { ...current, x: current.x + dx, y: current.y + dy }
        return true
    }
    return false
}

function tryRotate(dir) {
    if (locking || eliminated) {
        return
    }
    const rot = (current.rot + dir + 4) % 4
    const next = kick(current.piece, rot, current.x, current.y)
    if (next) {
        current = { piece: current.piece, ...next }
    }
}

function softDrop() {
    if (!tryMove(0, 1)) {
        void postLock()
    }
}

function hardDrop() {
    if (locking || eliminated) {
        return
    }
    current = { ...current, y: ghostY() }
    void postLock()
}

function paint() {
    const canvas = document.getElementById('blocks-canvas')
    if (!(canvas instanceof HTMLCanvasElement)) {
        return
    }
    const ctx = canvas.getContext('2d')
    const cw = canvas.width / COLS
    const ch = canvas.height / ROWS
    ctx.fillStyle = '#07110e'
    ctx.fillRect(0, 0, canvas.width, canvas.height)
    for (let y = 0; y < ROWS; y += 1) {
        for (let x = 0; x < COLS; x += 1) {
            const cell = board[y * COLS + x] ?? '.'
            ctx.fillStyle = COLORS[cell] ?? COLORS['.']
            ctx.fillRect(x * cw + 1, y * ch + 1, cw - 2, ch - 2)
        }
    }
    if (!eliminated && OFFSETS[current.piece]) {
        const gy = ghostY()
        ctx.globalAlpha = 0.28
        for (const cell of cellsOf(current.piece, current.rot, current.x, gy) ?? []) {
            ctx.fillStyle = COLORS[current.piece]
            ctx.fillRect(cell.x * cw + 1, cell.y * ch + 1, cw - 2, ch - 2)
        }
        ctx.globalAlpha = 1
        for (const cell of cellsOf(current.piece, current.rot, current.x, current.y) ?? []) {
            ctx.fillStyle = COLORS[current.piece]
            ctx.fillRect(cell.x * cw + 1, cell.y * ch + 1, cw - 2, ch - 2)
        }
    }
}

function handleSteer(action) {
    matchAction(action)
}

function matchAction(action) {
    switch (action) {
        case 'left':
            tryMove(-1, 0)
            break
        case 'right':
            tryMove(1, 0)
            break
        case 'down':
            softDrop()
            break
        case 'rot':
            tryRotate(1)
            break
        case 'ccw':
            tryRotate(-1)
            break
        case 'drop':
            hardDrop()
            break
        default:
            break
    }
}

function keysToAction(key) {
    switch (key) {
        case 'ArrowLeft':
        case 'a':
            return 'left'
        case 'ArrowRight':
        case 'd':
            return 'right'
        case 'ArrowDown':
        case 's':
            return 'down'
        case 'ArrowUp':
        case 'x':
        case 'w':
            return 'rot'
        case 'z':
            return 'ccw'
        case ' ':
            return 'drop'
        default:
            return ''
    }
}

addEventListener('keydown', (event) => {
    if (
        event.target instanceof HTMLInputElement ||
        event.target instanceof HTMLTextAreaElement ||
        event.target?.isContentEditable
    ) {
        return
    }
    const action = keysToAction(event.key) || keysToAction(event.key.toLowerCase())
    if (!action) {
        return
    }
    event.preventDefault()
    if (event.repeat && (action === 'rot' || action === 'ccw' || action === 'drop')) {
        return
    }
    if (action === 'left' || action === 'right') {
        das = { dir: action === 'left' ? -1 : 1, delay: 133 }
        tryMove(das.dir, 0)
        return
    }
    matchAction(action)
})

addEventListener('keyup', (event) => {
    const action = keysToAction(event.key) || keysToAction(event.key.toLowerCase())
    if ((action === 'left' && das.dir === -1) || (action === 'right' && das.dir === 1)) {
        das = { dir: 0, delay: 0 }
    }
})

addEventListener('pointerdown', (event) => {
    const reset = event.target instanceof Element ? event.target.closest('[data-reset]') : null
    if (reset) {
        event.preventDefault()
        void resetBoard()
        return
    }
    const button = event.target instanceof Element ? event.target.closest('[data-steer]') : null
    if (!(button instanceof HTMLElement)) {
        return
    }
    event.preventDefault()
    handleSteer(button.getAttribute('data-steer'))
})

function pollGamepad() {
    const pads = navigator.getGamepads?.() ?? []
    const pad = [...pads].find(Boolean)
    if (!pad) {
        return
    }
    const pressed = {
        left: pad.axes[0] < -0.5 || pad.buttons[14]?.pressed,
        right: pad.axes[0] > 0.5 || pad.buttons[15]?.pressed,
        down: pad.axes[1] > 0.5 || pad.buttons[13]?.pressed,
        rot: pad.buttons[0]?.pressed || pad.buttons[5]?.pressed,
        ccw: pad.buttons[1]?.pressed || pad.buttons[4]?.pressed,
        drop: pad.buttons[2]?.pressed || pad.buttons[3]?.pressed,
    }
    for (const action of ['left', 'right', 'down', 'rot', 'ccw', 'drop']) {
        if (pressed[action] && !gamepadPrev[action]) {
            matchAction(action)
        }
    }
    gamepadPrev = pressed
}

function tick(ts) {
    const dt = lastTs ? ts - lastTs : 0
    lastTs = ts
    if (das.dir !== 0) {
        das.delay -= dt
        if (das.delay <= 0) {
            tryMove(das.dir, 0)
            das.delay = 33
        }
    }
    gravityTimer += dt
    if (gravityTimer >= gravityMs) {
        gravityTimer = 0
        softDrop()
    }
    pollGamepad()
    paint()
    requestAnimationFrame(tick)
}

const root = document.getElementById('blocks-arena-state')
if (root) {
    readManifest()
    new MutationObserver(readManifest).observe(root, { attributes: true, subtree: true, childList: true })
    requestAnimationFrame(tick)
}

export { postLock, resetBoard }
