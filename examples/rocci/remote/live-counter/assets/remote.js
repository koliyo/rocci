// Only pending request envelopes live here. Canonical counter/feed are server HTML.
const pending = new Map();
const status = message => { const el = document.querySelector('#command-status'); if (el) el.textContent = message; };
const remember = () => sessionStorage.setItem('rocci-counter-pending', String(pending.size));
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
if (Number(sessionStorage.getItem('rocci-counter-pending')) > 0) {
  status('A command was interrupted. Its outcome is unknown; the live view will resync. It has not been resubmitted.');
  sessionStorage.removeItem('rocci-counter-pending');
}
window.rocciCounter = {
  async command(command, origin = '', tz = '') {
    const id = crypto.randomUUID();
    const envelope = command === 'increment' ? { v: 1, id, command, origin, tz } : { v: 1, id, command };
    pending.set(id, envelope); remember(); status('Sending…');
    let definite = false;
    try {
      for (let attempt = 0; attempt < 5; attempt++) {
        try {
          const response = await fetch(new URL('command', location.href), {
            method: 'POST', credentials: 'same-origin', redirect: 'manual',
            headers: { 'Content-Type': 'application/json', 'Datastar-Request': 'true' }, body: JSON.stringify(envelope),
            signal: AbortSignal.timeout(15000),
          });
          if (response.ok) { definite = true; status(''); return; }
          if (response.status < 500 && response.type !== 'opaqueredirect') {
            definite = true;
            status(`Command failed (${response.status}): ${await response.text()}. Reopen the page if your login expired.`);
            return;
          }
        } catch { /* A lost reply is ambiguous. The next attempt reuses this envelope. */ }
        if (attempt < 4) await pause(250 * 2 ** attempt);
      }
      status('Command outcome unknown. Reopen the page to resync; do not repeat the click until you check the count.');
    } finally {
      if (definite) pending.delete(id);
      remember();
    }
  },
};
