import { api, run, idle, exportCounter, restoreCounter } from './verify.js';
const status = document.querySelector('#status');
const result = document.querySelector('#result');
let busy = false;
function action(id, task) {
  document.getElementById(id).addEventListener('click', async () => {
    if (busy) return;
    busy = true;
    for (const button of document.querySelectorAll('button')) button.disabled = true;
    status.textContent = `Running: ${id}`;
    result.textContent = '';
    try {
      const evidence = await task();
      if (evidence.retryEnvelope) document.querySelector('#retry-envelope').value = JSON.stringify(evidence.retryEnvelope);
      result.textContent = JSON.stringify(evidence, null, 2);
      status.textContent = `Passed: ${id}`;
    } catch (error) {
      result.textContent = error.message;
      status.textContent = `Failed: ${id}`;
    } finally {
      busy = false;
      for (const button of document.querySelectorAll('button')) button.disabled = false;
    }
  });
}
action('run', run);
action('idle', idle);
action('manifest', async () => (await api('manifest')).json());
action('retry', async () => {
  const before = await (await api('manifest')).json();
  const response = await api('command', JSON.parse(document.querySelector('#retry-envelope').value));
  const after = await (await api('manifest')).json();
  if (before.runtime.revision !== after.runtime.revision)
    throw new Error('That envelope was a new operation: revision changed. Retry requires the exact original accepted envelope.');
  return { status: response.status, receiptRevision: response.headers.get('X-Counter-Revision'),
    revisionBefore: before.runtime.revision, revisionAfter: after.runtime.revision, deployment: after.deployment, schema: after.runtime.schema };
});
action('export', exportCounter);
action('unfreeze', async () => { await api('operator/unfreeze', {}); return { frozen: false }; });
action('restore', async () => {
  const file = document.querySelector('#file').files[0];
  if (!file) throw new Error('Choose the saved JSON export first');
  if (file.size > 8 * 1024 * 1024) throw new Error('Export exceeds the 8 MiB import limit');
  return restoreCounter(JSON.parse(await file.text()));
});
