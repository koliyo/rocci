const directions = new Map([
    ['ArrowUp', 'up'],
    ['w', 'up'],
    ['ArrowDown', 'down'],
    ['s', 'down'],
    ['ArrowLeft', 'left'],
    ['a', 'left'],
    ['ArrowRight', 'right'],
    ['d', 'right'],
])

const valid = new Set(['up', 'down', 'left', 'right'])

let sequence = 0

export function sendDirection(direction) {
    if (!valid.has(direction)) {
        return
    }

    void fetch('/api/direction', {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ direction, sequence: ++sequence }),
        keepalive: true,
    })
}

addEventListener('keydown', (event) => {
    if (
        event.target instanceof HTMLInputElement ||
        event.target instanceof HTMLTextAreaElement ||
        event.target?.isContentEditable
    ) {
        return
    }

    const direction = directions.get(event.key) ?? directions.get(event.key.toLowerCase())
    if (!direction || event.repeat) {
        return
    }

    event.preventDefault()
    sendDirection(direction)
})

addEventListener('pointerdown', (event) => {
    const button = event.target instanceof Element ? event.target.closest('[data-steer]') : null
    if (!(button instanceof HTMLElement)) {
        return
    }

    event.preventDefault()
    sendDirection(button.getAttribute('data-steer'))
})
