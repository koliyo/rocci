const deferred = () => {
  let resolve, reject;
  const promise = new Promise((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
};

export class Transactions {
  #active;
  constructor(storage) {
    this.storage = storage;
  }
  get active() {
    return this.#active;
  }
  async begin(mode) {
    if (!this.storage)
      throw new Error("Transactions require object-managed SQLite");
    if (mode !== 0 && mode !== 1)
      throw new Error(
        "Worker SQL supports Deferred and Immediate transaction scopes; Exclusive has no provider equivalent",
      );
    if (this.#active)
      throw new Error(
        "Nested or overlapping transaction scopes are unsupported",
      );
    const opened = deferred(),
      close = deferred();
    const state = { kind: "transaction", close, finished: false };
    this.#active = state;
    state.completion = this.storage
      .transaction(async (transaction) => {
        opened.resolve();
        const commit = await close.promise;
        if (!commit) transaction.rollback();
      })
      .catch((error) => {
        opened.reject(error);
        throw error;
      })
      .finally(() => {
        state.finished = true;
        if (this.#active === state) this.#active = undefined;
      });
    // Attach before publishing the handle so a provider rejection cannot become
    // an unhandled rejection while Roc is suspended inside the begin import.
    state.completion.catch(() => {});
    await opened.promise;
    return state;
  }
  async finish(state, commit) {
    if (state.finished || this.#active !== state)
      throw new Error("Transaction already finished");
    state.close.resolve(commit);
    await state.completion;
  }
  async drop(state) {
    if (!state.finished) {
      state.close.resolve(false);
      await state.completion;
    }
  }
}
