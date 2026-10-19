# Plain Roc SQL example

`main.roc` demonstrates typed values and transaction boundaries using ordinary
Roc platform APIs. `rocci.toml` declares the object-managed SQLite mapping.
`rocci worker init` and `rocci worker build` supply the JavaScript hosting.

The scripts in `tests/` own the example-specific transaction and failure
assertions; they are acceptance tests rather than application dependencies.
From the repository root:

```sh
ROCCI_WORKER_ROC=/path/to/pinned/roc node test/workers/applications.mjs transactions
```
